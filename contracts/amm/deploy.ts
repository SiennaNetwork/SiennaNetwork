import {
  MigrationContext, bold, Agent, randomHex, Console, timestamp, printContracts
} from '@hackbg/fadroma'
import getSettings, { workspace, SIENNA_DECIMALS, ONE_SIENNA } from '@sienna/settings'
import { SNIP20Contract } from '@fadroma/snip20'
import {
  FactoryContract,
  AMMContract,
  AMMSNIP20Contract,
  IDOContract,
  LPTokenContract,
  LaunchpadContract,
  RPTContract, RPTConfig,
  RewardsContract, RewardsAPIVersion,
  SiennaSNIP20Contract,
} from '@sienna/api'

const console = Object.assign(Console('@sienna/amm/upgrade'), { table: global.console.table })

/** Taking a TGE deployment, add the AMM to it,
  * creating the pre-configured liquidity and reward pools. */
export async function deployAMM ({
  deployment, admin, run,
  SIENNA = deployment.getContract(admin, SiennaSNIP20Contract, 'SiennaSNIP20')
}: MigrationContext & {
  /* The deployment's SIENNA token. */
  SIENNA: SiennaSNIP20Contract
}): Promise<{
  /* The newly created factory contract. */
  FACTORY:      FactoryContract
  /* Collection of tokens supported by the AMM. */
  TOKENS:       Record<string, SNIP20Contract>
  /* List of exchanges created. */
  EXCHANGES:    AMMContract[]
  /* List of LP tokens created. */
  LP_TOKENS:    LPTokenContract[]
  /* List of reward pools created. */
  REWARD_POOLS: RewardsContract[]
  /* RPT config that was set. */
  RPT_CONFIG:   RPTConfig
}> {
  const { FACTORY } = 
    await run(deployAMMFactory)
  const { TOKENS, EXCHANGES, LP_TOKENS } =
    await run(deployAMMExchanges, { SIENNA, FACTORY })
  const { SSSSS_POOL, RPT_CONFIG_SSSSS } =
    await run(deploySSSSS, { SIENNA })
  const { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS } =
    await run(deployRewards, { apiVersion: 'v3', SIENNA, FACTORY, TOKENS })
  const { RPT_CONFIG } =
    await run(adjustRPTConfig, { RPT_CONFIG_SSSSS, RPT_CONFIG_SWAP_REWARDS })

  console.log()
  console.info(bold('Deployed AMM contracts:'))
  printContracts([FACTORY,...EXCHANGES,...LP_TOKENS,...REWARD_POOLS])
  console.log()

  return {
    FACTORY,
    TOKENS,
    EXCHANGES,
    LP_TOKENS,
    REWARD_POOLS,
    RPT_CONFIG
  }
}

/** After deploying the SSSSS and the other reward pools,
  * set their addresses in the deployment's RPT contract. */
export async function adjustRPTConfig ({
  deployment, chain, admin,
  RPT = deployment.getContract(admin, RPTContract, 'SiennaRPT'),
  RPT_CONFIG_SSSSS,
  RPT_CONFIG_SWAP_REWARDS
}: MigrationContext & {
  /** The RPT contract to be configured.*/
  RPT:                     RPTContract,
  /** The config section for SSSSS (normally 1 entry). */
  RPT_CONFIG_SSSSS:        RPTConfig,
  /** The config section for Sienna Swap Rewards. */
  RPT_CONFIG_SWAP_REWARDS: RPTConfig
}): Promise<{
  /* The final config that was set in the RPT contract. */
  RPT_CONFIG: RPTConfig
}> {
  const RPT_CONFIG = [
    ...RPT_CONFIG_SSSSS,
    ...RPT_CONFIG_SWAP_REWARDS
  ]
  // on mainnet we use a multisig
  // so we can't run the transaction from here
  if (chain.isMainnet) {
    deployment.save({config: RPT_CONFIG}, 'RPTConfig.json')
    console.info(
      `\n\nWrote RPT config to deployment ${deployment.prefix}. `+
      `You should use this file as the basis of a multisig transaction.`
    )
    return
  }
  console.info(
    bold(`Configuring RPT`), RPT.address
  )
  for (const [address, amount] of RPT_CONFIG) {
    console.info(`- ${address} ${amount}`)
  }
  await RPT.tx(admin).configure(RPT_CONFIG)
  return { RPT_CONFIG }
}

/** Deploy the Factory contract which is the hub of the AMM.
  * It needs to be passed code ids and code hashes for
  * the different kinds of contracts that it can instantiate.
  * So build and upload versions of those contracts too. */
export async function deployAMMFactory ({
  prefix, admin, chain, deployment
}: MigrationContext): Promise<{
  /* This deployment's Factory context. */
  FACTORY: FactoryContract
}> {
  const options = { workspace, prefix, admin }
  const FACTORY = new FactoryContract({ ...options })
  const [_, EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD] = await chain.buildAndUpload(admin, [
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
  // configure factory: set fees etc
  // deploy the factory
  await deployment.getOrCreateContract(admin, FACTORY, 'SiennaAMMFactory', {
    admin:              admin.address,
    prng_seed:          randomHex(36),
    exchange_settings:  getSettings(chain.chainId).amm.exchange_settings,
    snip20_contract:    template(AMMTOKEN),
    pair_contract:      template(EXCHANGE),
    lp_token_contract:  template(LPTOKEN),
    ido_contract:       template(IDO),
    launchpad_contract: template(LAUNCHPAD),
  })
  return { FACTORY }
}

/** Deploy a Factory with the same settings as
  * the one that exists in the deployment, but
  * with code from the `main` branch. */
export async function deployAMMFactoryLegacy ({
  deployment, prefix, timestamp, chain, admin,
  FACTORY = deployment.getContract(admin, FactoryContract, 'SiennaAMMFactory')
}: MigrationContext & {
  /* The current factory whose settings will be copied to the legacy factory. */
  FACTORY: FactoryContract
}): Promise<{
  /* The newly deployed legacy factory */
  LEGACY_FACTORY: FactoryContract
}> {
  const LEGACY_FACTORY = new FactoryContract({
    workspace, ref: `main`, suffix: `@v1+${timestamp}`
  })
  await chain.buildAndUpload(admin, [LEGACY_FACTORY])
  await deployment.createContract(admin, LEGACY_FACTORY, {
    admin:             admin.address,
    prng_seed:         randomHex(36),
    exchange_settings: getSettings(chain.chainId).amm.exchange_settings,
    ...await FACTORY.getContracts()
  })
  return { LEGACY_FACTORY }
}

export async function deployAMMExchanges ({
  chain, run,
  SIENNA,
  FACTORY,
  settings: { swapTokens, swapPairs } = getSettings(chain.chainId),
}: MigrationContext & {
  /* The SIENNA token. */
  SIENNA:  SiennaSNIP20Contract
  /* The FACTORY contract that will create the exchanges. */
  FACTORY: FactoryContract,
  /* Lists of tokens to know and exchanges to create, from project settings. */
  settings: { swapTokens: Record<string, any>, swapPairs: Array<any> }
}): Promise<{
  /* A collection of the tokens used by the exchanges. */
  TOKENS:    Record<string, SNIP20Contract>,
  /* The created AMM exchanges. */
  EXCHANGES: AMMContract[],
  /* The LP tokens of the created exchanges. */
  LP_TOKENS: LPTokenContract[],
}> {
  // Collect referenced tokens, and created exchanges/LPs
  const TOKENS:    Record<string, SNIP20Contract> = { SIENNA }
  const EXCHANGES: AMMContract[]     = []
  const LP_TOKENS: LPTokenContract[] = []
  if (chain.isLocalnet) {
    // On localnet, deploy some placeholder tokens corresponding to the config.
    const { PLACEHOLDERS } = await run(deployPlaceholders)
    Object.assign(TOKENS, PLACEHOLDERS)
  } else {
    // On testnet and mainnet, talk to preexisting token contracts from the config.
    console.info(`Not running on localnet, using tokens from config:`)
    Object.assign(TOKENS, getSwapTokens(swapTokens))
    console.debug(bold('Tokens:'), TOKENS)
  }
  // If there are any initial swap pairs defined in the config
  if (swapPairs.length > 0) {
    for (const name of swapPairs) {
      // Call the factory to deploy an EXCHANGE for each
      const { EXCHANGE, LP_TOKEN } = await run(deployAMMExchange, {
        FACTORY, TOKENS, name
      })
      // And collect the results
      EXCHANGES.push(EXCHANGE)
      LP_TOKENS.push(LP_TOKEN)
    }
  }
  return { TOKENS, LP_TOKENS, EXCHANGES }
}

export async function deployAMMExchange ({
  admin, deployment,
  FACTORY, TOKENS, name
}: MigrationContext & {
  /* The factory that will be commanded to deploy the exchange. */
  FACTORY: FactoryContract
  /* A collection of known tokens, between two of which the exchange will be created. */
  TOKENS:  Record<string, SNIP20Contract>
  /* The name of the exchange, in the form TOKEN0-TOKEN1 */
  name:    string
}): Promise<{
  /* The created exchange. */
  EXCHANGE: AMMContract
  /* The LP token created for the exchange. */
  LP_TOKEN: LPTokenContract
}> {
  console.info(
    bold(`Deploying AMM exchange`), name
  )
  const [tokenName0, tokenName1] = name.split('-')
  const token0 = TOKENS[tokenName0].asCustomToken
  const token1 = TOKENS[tokenName1].asCustomToken
  //console.info(`- Token 0: ${bold(JSON.stringify(token0))}...`)
  //console.info(`- Token 1: ${bold(JSON.stringify(token1))}...`)
  try {
    const { EXCHANGE, LP_TOKEN } = await FACTORY.getExchange(token0, token1, admin)
    console.info(`${bold(name)}: Already exists.`)
    return { EXCHANGE, LP_TOKEN }
  } catch (e) {
    if (e.message.includes("Address doesn't exist in storage")) {
      const {
        pair_contract:     { id: ammId, code_hash: ammHash },
        lp_token_contract: { id: lpId }
      } = await FACTORY.getContracts()
      const { EXCHANGE, LP_TOKEN, raw } = await FACTORY.createExchange(token0, token1)
      console.info(bold(`Deployed AMM exchange`), EXCHANGE.address)
      deployment.save({
        ...raw,
        codeId:   ammId,
        codeHash: ammHash,
        initTx:   { contractAddress: raw.exchange.address }
      }, `SiennaSwap_${name}`)
      console.info(bold(`Deployed LP token`), LP_TOKEN.address)
      deployment.save({
        ...raw,
        codeId:   lpId,
        codeHash: raw.lp_token.code_hash,
        initTx:   { contractAddress: raw.lp_token.address }
      }, `SiennaSwap_LP-${name}`)
      return { EXCHANGE, LP_TOKEN }
    } else {
      console.error(e)
      throw new Error(`${bold(`Factory::GetExchange(${name})`)}: not found (${e.message})`)
    }
  }
}

export async function deployPlaceholders (
  { deployment, chain, admin, prefix }: MigrationContext
): Promise<{
  PLACEHOLDERS: Record<string, SNIP20Contract>
}> {
  // this can later be used to check if the deployed contracts have
  // gone out of date (by codehash) and offer to redeploy them
  const PLACEHOLDERS = {}
  const { placeholderTokens } = getSettings(chain.chainId)
  console.info(
    bold(`Deploying placeholder tokens`), Object.keys(placeholderTokens).join(' ')
  )
  type TokenConfig = { label: string, initMsg: any }
  const placeholders: Record<string, TokenConfig> = placeholderTokens
  for (const [symbol, {label, initMsg}] of Object.entries(placeholders)) {
    const name = `Placeholder_${label}` 
    try {
      PLACEHOLDERS[symbol] = deployment.getContract(admin, AMMSNIP20Contract, name)
      console.info(bold('Found, not redeploying:'), name)
    } catch (e) {
      if (e.message.startsWith('@fadroma/ops: no contract')) {
        console.info(bold('Not found, deploying:'), name)
        const TOKEN = PLACEHOLDERS[symbol] = new AMMSNIP20Contract({
          workspace, prefix, name, suffix: `+${timestamp()}`,
        })
        await chain.buildAndUpload(admin, [TOKEN])
        await deployment.createContract(admin, TOKEN, {
          ...initMsg, name, symbol: symbol.toUpperCase()
        })
        await TOKEN.tx().setMinters([admin.address])
        await TOKEN.tx().mint("100000000000000000000000", admin.address)
      } else {
        console.error(e)
        throw new Error(
          `@sienna/amm/deploy: error when deploying placeholder tokens: ${e.message}`
        )
      }
    }
  }
  return { PLACEHOLDERS }
}

export function getSwapTokens (
  links:  Record<string, { address: string, codeHash: string }>,
  admin?: Agent
): Record<string, SNIP20Contract> {
  const tokens = {}
  for (const [name, {address, codeHash}] of Object.entries(links)) {
    tokens[name] = new AMMSNIP20Contract({address, codeHash, admin})
  }
  return tokens
}

/** Deploy SIENNA/SIENNA SINGLE-SIDED STAKING,
  * (5- or 6-S depending on whether you count the SLASH)
  * a Sienna Rewards pool where you stake SIENNA to earn SIENNA. */
export async function deploySSSSS ({
  run, chain, deployment,
  SIENNA
}: MigrationContext & {
  SIENNA: SiennaSNIP20Contract
}): Promise<{
  SSSSS_POOL:       RewardsContract
  RPT_CONFIG_SSSSS: RPTConfig
}> {
  const { REWARDS: SSSSS_POOL } = await run(deployRewardPool, {
    name:        'SIENNA_SIENNA',
    lpToken:     SIENNA,
    rewardToken: SIENNA,
  })
  return {
    SSSSS_POOL,
    RPT_CONFIG_SSSSS: [
      [
        SSSSS_POOL.address,
        String(BigInt(getSettings(chain.chainId).rewardPairs.SIENNA) * ONE_SIENNA)
      ]
    ]
  }

}

/** Deploy the rest of the reward pools,
  * where you stake a LP token to earn SIENNA. */
export async function deployRewards ({
  chain, admin, deployment, prefix, run,
  apiVersion = 'v3',
  suffix     = `_${apiVersion}+${timestamp()}`,
  split      = 1.0,
  ref        = 'HEAD',
  SIENNA     = deployment.getContract(admin, SiennaSNIP20Contract, 'SiennaSNIP20'),
  FACTORY    = deployment.getContract(admin, FactoryContract,      'SiennaAMMFactory'),
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
  REWARD_POOLS:       RewardsContract[]
  RPT_CONFIG_SWAP_REWARDS: RPTConfig
}> {
  const { swapPairs, rewardPairs } = getSettings(chain.chainId)
  const REWARDS = new RewardsContract({ workspace, prefix, admin, ref })
  await chain.buildAndUpload(admin, [REWARDS])
  const REWARD_POOLS            = []
  const RPT_CONFIG_SWAP_REWARDS = []
  if (swapPairs.length > 0) {
    const rewards = rewardPairs
    for (const name of swapPairs) {
      if (rewards && rewards[name]) {
        const exchangeName = `SiennaSwap_${name}`
        const exchange = deployment.receipts[exchangeName]
        if (!exchange) {
          console.error(bold(`Contract does not exist in deployment`), exchangeName)
          console.error(bold(`Contracts in deployment:`), Object.keys(deployment.receipts).join(' '))
          process.exit(1)
        }
        const { lp_token } = exchange
        console.info(
          bold(`Deploying rewards for ${name}`),
          JSON.stringify({ lp_token })
        )
        const { REWARDS } = await run(deployRewardPool, {
          name: `${name}_SIENNA`,
          suffix,
          lpToken: new LPTokenContract({
            address:  exchange.lp_token.address,
            codeHash: exchange.lp_token.code_hash,
            admin
          }),
          rewardToken: SIENNA
        })
        REWARD_POOLS.push(REWARDS)
        const reward = BigInt(rewards[name]) / BigInt(1 / split)
        RPT_CONFIG_SWAP_REWARDS.push(
          [REWARDS.address, String(reward * ONE_SIENNA)]
        )
      }
    }
  }
  return { REWARD_POOLS, RPT_CONFIG_SWAP_REWARDS }
}

export async function deployRewardPool ({
  admin, chain, deployment, prefix,
  name        = 'UNTITLED',
  lpToken,
  rewardToken = new SiennaSNIP20Contract().from(deployment),
  apiVersion  = 'v3',
  suffix      = `_${apiVersion}+${timestamp()}`,
}: MigrationContext & {
  name:        string
  lpToken:     SNIP20Contract
  rewardToken: SNIP20Contract
  apiVersion:  'v2'|'v3'
  suffix:      string
}): Promise<{
  REWARDS:     RewardsContract
}> {
  const REWARDS = new RewardsContract({ workspace, name: `SiennaRewards_${name}` })
  await chain.buildAndUpload(admin, [REWARDS])
  let initMsg
  switch (apiVersion) {
    case 'v3':
      initMsg = {
        admin:  admin.address,
        config: {
          reward_vk:    randomHex(36),
          bonding:      86400,
          timekeeper:   admin.address,
          lp_token:     lpToken.link,
          reward_token: rewardToken.link,
        }
      }
      break
    case 'v2':
      initMsg = {
        admin:        admin.address,
        lp_token:     lpToken.link,
        reward_token: rewardToken.link,
        viewing_key:  "",
        ratio:        ["1", "1"],
        threshold:    15940,
        cooldown:     15940,
      }
      break
    default:
      throw new Error
  }
  await deployment.getOrCreateContract(
    admin, REWARDS, REWARDS.label, initMsg
  )
  return { REWARDS }
}

export async function deployRewardsSideBySide ({
  timestamp, run, chain, admin, prefix, deployment
}: MigrationContext) {
  const options = { chain, admin, prefix }
  const [v2, v3] = await Promise.all([
    run(deployRewards, { ...options, apiVersion: 'v2', split: 0.5, ref: 'rewards-2.1.2' }),
    run(deployRewards, { ...options, apiVersion: 'v3', split: 0.5, ref: 'HEAD' }),
  ])
  const RPT_CONFIG = [
    ...v2.RPT_CONFIG,
    ...v3.RPT_CONFIG
  ]
  const RPT = deployment.getContract(admin, RPTContract, 'SiennaRPT')
  await RPT.tx(admin).configure(RPT_CONFIG)
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
