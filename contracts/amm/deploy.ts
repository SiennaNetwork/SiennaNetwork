import { MigrationContext, bold, IAgent, randomHex, Console } from '@hackbg/fadroma'
import getSettings, { workspace, SIENNA_DECIMALS, ONE_SIENNA } from '@sienna/settings'
import { SNIP20Contract } from '@fadroma/snip20'
import {
  FactoryContract,
  AMMContract,
  AMMSNIP20Contract,
  IDOContract,
  LPTokenContract,
  LaunchpadContract,
  RPTContract,
  RewardsContract,
  SiennaSNIP20Contract,
} from '@sienna/api'

const console = Console('@sienna/amm/deploy')

export async function deployAMM ({ run }: MigrationContext) {
  const { FACTORY } = await run(deployAMMFactory)
  const { TOKENS, EXCHANGES, LP_TOKENS } = await run(deployAMMExchanges, { FACTORY })
}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory ({
  prefix, admin, chain, deployment
}: MigrationContext): Promise<{
  FACTORY: FactoryContract
}> {
  const options = { workspace, prefix, admin }
  const FACTORY = new FactoryContract({ ...options })
  const [_, EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD] = await chain.buildAndUpload([
    // only this one will be deployed
    FACTORY,
    // however all of them must be built, so that
    // the factory can be given their code ids/hashes
    new AMMContract({       ...options }),
    new AMMSNIP20Contract({ ...options }),
    new LPTokenContract({   ...options }),
    new IDOContract({       ...options }),
    new LaunchpadContract({ ...options }),
  ])
  // extract id and code_hash from each uploaded contract
  const template = contract => ({ id: contract.codeId, code_hash: contract.codeHash })
  // configure factory: set supported contracts
  FACTORY.setContracts({
    snip20_contract:    template(AMMTOKEN),
    pair_contract:      template(EXCHANGE),
    lp_token_contract:  template(LPTOKEN),
    ido_contract:       template(IDO),
    launchpad_contract: template(LAUNCHPAD),
  })
  // configure factory: set fees etc
  const { amm: { exchange_settings } } = getSettings(chain.chainId)
  FACTORY.initMsg.exchange_settings = exchange_settings
  // deploy the factory
  const receipt = deployment.contracts['SiennaAMMFactory']
  await FACTORY.instantiateOrExisting(receipt)
  return { FACTORY }
}

/** Deploy a Factory with the same settings as
  * the one that exists in the deployment, but
  * with code from the `main` branch. */
export async function deployAMMFactoryLegacy ({
  deployment, prefix, timestamp, chain, admin
}: MigrationContext): Promise<{
  LEGACY_FACTORY: FactoryContract
}> {
  const FACTORY = deployment.getContract(FactoryContract, 'SiennaAMMFactory', admin)
  const LEGACY_FACTORY = new FactoryContract({
    ref:    `main`,
    prefix,
    suffix: `@v1+${timestamp}`,
    admin,
    exchange_settings: getSettings(chain.chainId).amm.exchange_settings,
    contracts:         await FACTORY.getContracts(),
  })
  await chain.buildAndUpload([LEGACY_FACTORY])
  await LEGACY_FACTORY.instantiate()
  return { LEGACY_FACTORY }
}

export async function deployAMMExchanges ({
  chain, run,
  SIENNA, FACTORY
}: MigrationContext & {
  SIENNA:  SiennaSNIP20Contract
  FACTORY: FactoryContract
}): Promise<{
  TOKENS:    Record<string, SNIP20Contract>,
  EXCHANGES: AMMContract[],
  LP_TOKENS: LPTokenContract[],
}> {
  const { swapTokens, swapPairs, rewardPairs } = getSettings(chain.chainId)
  // Collect referenced tokens, and created exchanges/LPs
  const TOKENS:    Record<string, SNIP20Contract>  = { SIENNA }
  const EXCHANGES: AMMContract[]     = []
  const LP_TOKENS: LPTokenContract[] = []
  if (chain.isLocalnet) {
    // On localnet, deploy some placeholder tokens corresponding to the config.
    console.info(`Running on ${bold('localnet')}, deploying placeholder tokens...`)
    Object.assign(TOKENS, await run(deployPlaceholderTokens))
  } else {
    // On testnet and mainnet, talk to preexisting token contracts from the config.
    console.info(`Not running on localnet, using tokens from config:`)
    Object.assign(TOKENS, getSwapTokens(swapTokens))
    console.debug(bold('Tokens:'), TOKENS)
  }
  // If there are any initial swap pairs defined in the config
  if (swapPairs.length > 0) {
    // Call the factory to deploy an EXCHANGE for each
    for (const name of swapPairs) {
      const EXCHANGE = await run(deployAMMExchange, { FACTORY, TOKENS, name })
      EXCHANGES.push(EXCHANGE)
      LP_TOKENS.push(EXCHANGE.lp_token)
    }
  }
  return { TOKENS, LP_TOKENS, EXCHANGES }
}

export async function deployAMMExchange ({
  admin, deployment,
  FACTORY, TOKENS, name
}: MigrationContext & {
  FACTORY: FactoryContract
  TOKENS:  Record<string, SNIP20Contract>
  name:    string
}) {
  console.info(`Deploying liquidity pool ${bold(name)}...`)
  const [tokenName0, tokenName1] = name.split('-')
  const token0 = TOKENS[tokenName0].asCustomToken
  const token1 = TOKENS[tokenName1].asCustomToken
  console.info(`- Token 0: ${bold(JSON.stringify(token0))}...`)
  console.info(`- Token 1: ${bold(JSON.stringify(token1))}...`)
  try {
    const EXCHANGE = await FACTORY.getExchange(token0, token1, admin)
    console.info(`${bold(name)}: Already exists.`)
    return { EXCHANGE }
  } catch (e) {
    if (e.message.includes("Address doesn't exist in storage")) {
      const EXCHANGE = await FACTORY.createExchange(token0, token1)
      deployment.save(EXCHANGE, `SiennaSwap_${name}`)
      console.info(
        `Deployed liquidity pool ${EXCHANGE.exchange.address} `+
        ` and LP token ${EXCHANGE.lp_token.address}`
      )
      return EXCHANGE
    } else {
      throw new Error(`${bold(`Factory::GetExchange(${name})`)}: not found (${e.message})`)
    }
  }
}

export async function deployPlaceholderTokens (
  { deployment, chain, admin, prefix, timestamp }: MigrationContext
): Promise<{
  PLACEHOLDER_TOKENS: Record<string, SNIP20Contract>
}> {
  const AMMTOKEN = new AMMSNIP20Contract({ workspace, prefix, chain, admin })
  // this can later be used to check if the deployed contracts have
  // gone out of date (by codehash) and offer to redeploy them
  await chain.buildAndUpload([AMMTOKEN])
  const PLACEHOLDER_TOKENS = {}
  const { placeholderTokens } = getSettings(chain.chainId)
  type TokenConfig = { label: string, initMsg: any }
  const placeholders: Record<string, TokenConfig> = placeholderTokens
  for (const [symbol, {label: suffix, initMsg}] of Object.entries(placeholders)) {
    const TOKEN = PLACEHOLDER_TOKENS[symbol] = new AMMSNIP20Contract({
      chain,
      prefix,
      admin,
      instantiator: admin,
      codeId:       AMMTOKEN.codeId,
      codeHash:     AMMTOKEN.codeHash,
      name:         'AMMSNIP20',
      suffix:       `_${suffix}+${timestamp}`,
      initMsg: {
        ...initMsg,
        prng_seed: randomHex(36)
      }
    })
    // the instantiateOrExisting mechanic needs work -
    // chiefly, to decide in which subsystem it lives.
    // probably move that into `deployment` as well.
    // or, Deployment's child - Migration proper,
    // represented by a single JSON file containing
    // all the inputs and outputs of one of these.
    const existing = deployment.contracts[`AMMSNIP20_${suffix}`]
    await TOKEN.instantiateOrExisting(existing)
    // newly deployed placeholder tokens give the admin a large balance.
    // these are only intended for localnet so when you run out of it
    // it's a good time to redeploy the localnet to see if all is in order anyway.
    if (!existing) {
      await TOKEN.tx(admin).setMinters([admin.address])
      await TOKEN.tx(admin).mint("100000000000000000000000", admin.address)
    }
  }
  return { PLACEHOLDER_TOKENS }
}

export function getSwapTokens (
  links:  Record<string, { address: string, codeHash: string }>,
  admin?: IAgent
): Record<string, SNIP20Contract> {
  const tokens = {}
  for (const [name, {address, codeHash}] of Object.entries(links)) {
    tokens[name] = new AMMSNIP20Contract({address, codeHash, admin})
    console.log('getSwapToken', name, address, codeHash)
  }
  return tokens
}

/** Deploy SIENNA/SIENNA SINGLE-SIDED STAKING,
  * (SSSSS or SSSSSS depending on whether you count the SLASH)
  * a Sienna Rewards pool where you stake SIENNA to earn SIENNA. */
export async function deploySSSSS ({
  run, chain,
  SIENNA
}: MigrationContext & {
  SIENNA: SiennaSNIP20Contract
}): Promise<{
  RPT_CONFIG_PARTIAL: [string, string][]
}> {

  const singleSidedStaking = await run(deployRewardPool, {
    lpToken:     SIENNA,
    rewardToken: SIENNA,
  })

  return {
    RPT_CONFIG_PARTIAL: [
      [
        singleSidedStaking.address,
        String(BigInt(getSettings(chain.chainId).rewardPairs.SIENNA) * ONE_SIENNA)
      ]
    ]
  }

}

export type RPTRecipient = string

export type RPTAmount    = string

export type RPTConfig    = [RPTRecipient, RPTAmount][]

export type RewardsAPIVersion = 'v2'|'v3'

/** Deploy the rest of the reward pools,
  * where you stake a LP token to earn SIENNA. */
export async function deployRewards ({
  chain, admin, deployment, prefix, run,
  suffix     = '',
  apiVersion = 'v3',
  split      = 1.0,
  ref        = 'HEAD',
  SIENNA     = deployment.getContract(SiennaSNIP20Contract, 'SiennaSNIP20',     admin),
  FACTORY    = deployment.getContract(FactoryContract,      'SiennaAMMFactory', admin),
  TOKENS     = { SIENNA }
}: MigrationContext & {
  apiVersion?: RewardsAPIVersion
  suffix?:     string
  split?:      number
  ref?:        string
  SIENNA?:     SiennaSNIP20Contract
  FACTORY?:    FactoryContract,
  TOKENS?:     Record<string, SNIP20Contract>
}): Promise<{
  REWARD_POOLS: RewardsContract[]
  RPT_CONFIG:   RPTConfig
}> {
  const { swapTokens, swapPairs, rewardPairs, } = getSettings(chain.chainId)
  const REWARDS = new RewardsContract({ prefix, admin, ref })
  await chain.buildAndUpload([REWARDS])
  Object.assign(TOKENS,
    chain.isLocalnet
      ? await run(deployPlaceholderTokens)
      : getSwapTokens(swapTokens, admin))
  const REWARD_POOLS = []
  const RPT_CONFIG   = []
  const reward       = BigInt(rewardPairs.SIENNA) / BigInt(1 / split)
  const pool         = await run(deployRewardPool, { suffix, lpToken: SIENNA, rewardToken: SIENNA })
  REWARD_POOLS.push(pool)
  RPT_CONFIG.push([pool.address, String(reward * ONE_SIENNA)])
  if (swapPairs.length > 0) {
    const rewards = rewardPairs
    for (const name of swapPairs) {
      if (rewards && rewards[name]) {
        const exchangeName = `SiennaSwap_${name}`
        const exchange = deployment.contracts[exchangeName]
        if (!exchange) {
          console.log(`${exchangeName} doesn't exist`)
          process.exit(1)
        }
        const { lp_token } = exchange
        console.debug(`Deploying rewards for ${name}...`, { lp_token })
        const lpToken = new LPTokenContract({
          address:  exchange.lp_token.address,
          codeHash: exchange.lp_token.code_hash,
          admin
        })
        const reward = BigInt(rewards[name]) / BigInt(1 / split)
        const pool   = await run(deployRewardPool, { suffix, lpToken, rewardToken: SIENNA })
        REWARD_POOLS.push(pool)
        RPT_CONFIG.push([pool.address, String(reward * ONE_SIENNA)])
      }
    }
  }
  console.debug('Resulting RPT config:', RPT_CONFIG)
  return { REWARD_POOLS, RPT_CONFIG }
}

import { timestamp } from '@hackbg/fadroma'
export async function deployRewardPool ({
  admin, deployment, prefix,
  lpToken, rewardToken, apiVersion
}: MigrationContext & {
  apiVersion?:  'v2'|'v3'
  suffix?:      string
  lpToken?:     SNIP20Contract
  rewardToken?: SNIP20Contract
}) {
  const tokenInfo = await lpToken.q(admin).tokenInfo()
  const suffix    = `_${tokenInfo.symbol}_${apiVersion}+${timestamp()}`
  const contract  = new RewardsContract({
    workspace, prefix, suffix, lpToken, rewardToken,
    instantiator: admin, name: 'SiennaRewards',
  })
  await contract.buildInDocker()
  await contract.uploadAs(admin)
  if (apiVersion === 'v2') {
    // override init msg for legacy api
    const initMsg = {
      admin:        admin.address,
      lp_token:     lpToken.link,
      reward_token: rewardToken.link,
      viewing_key:  "",
      ratio:        ["1", "1"],
      threshold:    15940,
      cooldown:     15940,
    }
    // use Object.assign to avoid type check
    Object.assign(contract, { initMsg })
  }
  const receipt = deployment.contracts[contract.label]
  await contract.instantiateOrExisting(receipt)
  return contract
}

export async function deployRewardsSideBySide ({
  timestamp, run, chain, admin, prefix, deployment
}: MigrationContext) {
  const v2Suffix = `@v2+${timestamp}`
  const v3Suffix = `@v3+${timestamp}`
  const options = { chain, admin, prefix }
  const [v2, v3] = await Promise.all([
    run(deployRewards, {
      ...options, apiVersion: 'v2', suffix: v2Suffix, split: 0.5, ref: 'rewards-2.1.2'
    }),
    run(deployRewards, {
      ...options, apiVersion: 'v3', suffix: v2Suffix, split: 0.5, ref: 'HEAD'
    }),
  ])
  const RPT_CONFIG = [
    ...v2.RPT_CONFIG,
    ...v3.RPT_CONFIG
  ]
  const RPT = deployment.getContract(RPTContract, 'SiennaRPT', admin)
  await RPT.tx(admin).configure(RPT_CONFIG)
  console.log({RPT_CONFIG})
  console.table([
    ...v2.REWARD_POOLS,
    ...v3.REWARD_POOLS
  ].reduce((table, contract)=>{
    table[contract.init.label] = {
      address:  contract.init.address,
      codeId:   contract.blob.codeId,
      codeHash: contract.blob.codeHash
    }
    return table
  }, {}))
}
