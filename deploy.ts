/// # Sienna Deployment


import settings from '@sienna/settings'
import type { Chain, Agent, ContractUpload } from '@fadroma/ops'
import { Scrt } from '@fadroma/scrt'
import { bold, colors, timestamp, symlinkDir, randomHex } from '@fadroma/tools'
import process from 'process'
import { fileURLToPath } from 'url'
import { getDefaultSchedule, ONE_SIENNA } from './ops/index'


/// ## Build and upload
/// Contracts can be built in parallel, but have to be uploaded in separate blocks.


async function buildAndUpload (contracts: Array<ContractUpload>) {
  await Promise.all(contracts.map(contract=>contract.build()))
  for (const contract of contracts) {
    await contract.upload()
  }
}


/// ## Sienna TGE
/// This contains the Sienna SNIP20 token and the vesting contracts (MGMT and RPT).


import type { ScheduleFor_HumanAddr } from '@sienna/api/mgmt/handle'
import type { SNIP20Contract as SNIP20 } from '@sienna/api'
import { SiennaSNIP20, MGMTContract, RPTContract } from '@sienna/api'

export type VestingOptions = {
  prefix?:   string
  chain?:    Chain,
  admin?:    Agent,
  schedule?: ScheduleFor_HumanAddr
}

export async function deployVesting (options: VestingOptions = {}): Promise<SwapOptions> {
  const { prefix   = timestamp(),
          chain    = await new Scrt().ready,
          admin    = await chain.getAgent(),
          schedule = getDefaultSchedule() } = options

  const RPTAccount = getRPTAccount(schedule)
      , portion    = RPTAccount.portion_size

  const SIENNA = new SiennaSNIP20({ prefix, admin })
      , MGMT   = new MGMTContract({ prefix, admin, schedule, SIENNA })
      , RPT    = new RPTContract({ prefix, admin, MGMT, SIENNA, portion })

  await buildAndUpload([SIENNA, MGMT, RPT])

  await SIENNA.instantiate()

  RPTAccount.address = admin.address
  await MGMT.instantiate()
  await MGMT.acquire(SIENNA)

  await RPT.instantiate()
  RPTAccount.address = RPT.address
  await MGMT.configure(schedule)

  await MGMT.launch()
  await RPT.vest()

  return { prefix, chain, admin, SIENNA, MGMT, RPT }

  /// ### Get the RPT account from the schedule
  /// This is a special entry in MGMT's schedule that must be made to point to
  /// the RPT contract's address - but that's only possible after deploying
  /// the RPT contract. To prevent the circular dependency, the RPT account
  /// starts as pointing to the admin's address.

  function getRPTAccount (schedule: ScheduleFor_HumanAddr) {
    return schedule.pools
      .filter((x:any)=>x.name==='MintingPool')[0].accounts
      .filter((x:any)=>x.name==='RPT')[0] }

}


/// ## Sienna Swap


import {
  FactoryContract, AMMContract, AMMSNIP20, LPToken, RewardsContract, IDOContract, LaunchpadContract
} from '@sienna/api'
export type SwapOptions = {
  prefix:  string,
  chain?:  Chain,
  admin?:  Agent,
  SIENNA?: SiennaSNIP20
  MGMT?:   MGMTContract
  RPT?:    RPTContract
}
export async function deploySwap (options: SwapOptions) {
  const {
    prefix,
    chain  = await new Scrt().ready,
    admin  = await chain.getAgent(),
    MGMT,
  } = options

  const SIENNA = chain.instances.active.getContract(SiennaSNIP20, 'SiennaSNIP20', admin)
      , RPT = chain.instances.active.getContract(RPTContract, 'SiennaRPT', admin)
      , EXCHANGE = new AMMContract({ prefix, admin })
      , AMMTOKEN = new AMMSNIP20({ prefix, admin })
      , LPTOKEN = new LPToken({ prefix, admin })
      , IDO = new IDOContract({ prefix, admin })
      , REWARDS = new RewardsContract({ prefix, admin })
      , LAUNCHPAD = new LaunchpadContract({ prefix, admin })
      , FACTORY = new FactoryContract({
          prefix, admin, config: settings[`amm-${chain.chainId}`],
          EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD
        })

  await buildAndUpload([EXCHANGE, AMMTOKEN, LPTOKEN, IDO, FACTORY, REWARDS, LAUNCHPAD])

  await FACTORY.instantiateOrExisting(chain.instances.active.contracts['SiennaAMMFactory'])

  const tokens = { SIENNA }
  if (chain.isLocalnet) {
    Object.assign(tokens, await deployPlaceholderTokens())
  } else {
    Object.assign(getSwapTokens(settings[`swapTokens-${chain.chainId}`]))
  }

  const rptConfig = []

  const siennaStakingRewards = await deployRewardPool('SIENNA', SIENNA, SIENNA)
  rptConfig.push([
    siennaStakingRewards.address,
    String(BigInt(settings[`rewardPairs-${chain.chainId}`].SIENNA) * ONE_SIENNA)
  ])

  const swapPairs = settings[`swapPairs-${chain.chainId}`]

  if (swapPairs.length > 0) {
    const existingExchanges = await FACTORY.listExchanges()

    for (const name of swapPairs) {
      const [token0, token1] = await deploySwapPair(name, existingExchanges)

      const rewards = settings[`rewardPairs-${chain.chainId}`]
      if (rewards) {
        const lpToken = await getLPToken(token0, token1)
        const reward  = BigInt(rewards[name])
        const pool    = await deployRewardPool(name, lpToken, SIENNA)
        rptConfig.push([pool.address, String(reward * ONE_SIENNA)])
      }
    }
  }

  await RPT.configure(rptConfig)

  /// On localnet, placeholder tokens need to be deployed.

  async function deployPlaceholderTokens () {
    const tokens = {}
    for (
      const [symbol, {label, initMsg}]
      of Object.entries(settings[`placeholderTokens-${chain.chainId}`])
    ) {
      const token = tokens[symbol] = new AMMSNIP20({ admin })
      Object.assign(token.blob, { codeId: AMMTOKEN.codeId, codeHash: AMMTOKEN.codeHash })
      Object.assign(token.init, { prefix, label, msg: initMsg })
      Object.assign(token.init.msg, { prng_seed: randomHex(36) })
      const existing = chain.instances.active.contracts[label]
      await tokens[symbol].instantiateOrExisting(existing)
    }
    return tokens
  }


  /// On testnet and mainnet, interoperate with preexisting token contracts.


  function getSwapTokens (links: Record<string, { address: string, codeHash: string }>) {
    const tokens = {}
    for (const [name, {address, codeHash}] of Object.entries(links)) {
      tokens[name] = AMMSNIP20.attach(address, codeHash, admin)
    }
    return tokens
  }

  async function deploySwapPair (name: string, existingExchanges: any[]) {
    const [tokenName0, tokenName1] = name.split('-')
    const token0 = tokens[tokenName0]
        , token1 = tokens[tokenName1]
    for (const {pair} of existingExchanges) {
      if (
        (pair.token_0.custom_token.contract_addr === token0.address &&
        pair.token_1.custom_token.contract_addr === token1.address) ||
        (pair.token_0.custom_token.contract_addr === token1.address &&
        pair.token_1.custom_token.contract_addr === token0.address)
      ) {
        console.info(`Exchange exists: ${token0.init.label}/${token1.init.label}`)
        return [token0, token1]
      }
    }
    await FACTORY.createExchange(
      { custom_token: { contract_addr: token0.address, token_code_hash: token0.codeHash } },
      { custom_token: { contract_addr: token1.address, token_code_hash: token1.codeHash } }
    )
    return [token0, token1]
  }

  async function getLPToken (token0: SNIP20, token1: SNIP20) {
    const exchanges = await FACTORY.listExchanges()
    const {address: pairAddress} = exchanges.filter(({pair})=>(
      pair.token_0.custom_token.contract_addr === token0.address &&
      pair.token_1.custom_token.contract_addr === token1.address
    ))[0]
    const {pair_info} = await AMMContract.attach(pairAddress, EXCHANGE.codeHash, admin).pairInfo()
    const {address, code_hash} = pair_info.liquidity_token
    return LPToken.attach(address, code_hash, admin)
  }

  async function deployRewardPool (name: string, lpToken: SNIP20, rewardToken: SNIP20) {
    const {codeId, codeHash} = REWARDS
        , rewardPool = new RewardsContract({
            codeId, codeHash, prefix, name, admin, lpToken, rewardToken,
          })
    const receipt = chain.instances.active.contracts[rewardPool.init.label]
    await rewardPool.instantiateOrExisting(receipt)
    return rewardPool
  }

}


/// ## Upgrading reward pools


export async function replaceRewardPool (
  chain: Chain,
  admin: Agent,
  label: string
) {

  const { name: prefix, getContract } = chain.instances.active

  console.log(
    `Upgrading reward pool ${bold(label)}` +
    `\nin deployment ${bold(prefix)}` +
    `\non ${bold(chain.chainId)}` +
    `\nas ${bold(admin.address)}\n`
  )

  const OLD_POOL = getContract(RewardsContract, label, admin)
      , RPT      = getContract(RPTContract, 'SiennaRPT', admin)

  const {config} = await RPT.status
  let found: number = NaN
  for (let i = 0; i < config.length; i++) {
    console.log(config[i])
    if (config[i][0] === OLD_POOL.address) {
      if (!isNaN(found)) {
        console.log(`Address ${bold(OLD_POOL.address)} found in RPT config twice.`)
        process.exit(1)
      }
      found = i
    }
  }

  if (isNaN(found)) {
    console.log(`Reward pool ${bold(OLD_POOL.address)} not found in RPT ${bold(RPT.address)}`)
    process.exit(1)
  }

  console.log(`Replacing reward pool ${OLD_POOL.address}`)

  const [LP_TOKEN, REWARD_TOKEN] = await Promise.all([
    OLD_POOL.getLPToken(admin),
    OLD_POOL.getRewardToken(admin)
  ])

  const NEW_POOL = new RewardsContract({
    prefix, label: `${label}@${timestamp()}`, admin,
    lpToken:     LP_TOKEN,
    rewardToken: REWARD_TOKEN
  })
  await buildAndUpload([NEW_POOL])
  await NEW_POOL.instantiate()

  config[found][0] = NEW_POOL.address
  await RPT.configure(config)

  await OLD_POOL.close(`Moved to ${NEW_POOL.address}`)
}


/// ## Entry point


if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(()=>process.exit(0))
}

export default async function main ([chainName, ...words]: Array<string>) {

  let chain: Chain
  let admin: Agent


  /// Prefix is used to identify the instance.


  let prefix = timestamp()
  const commands = {

    reset () {
      if (!chain.node) {
        throw new Error(`${bold(chainName)}: not a localnet`)
      }
      return chain.node.terminate()
    },

    async select (id?: string) {
      const list = await chain.instances.list()
      if (list.length < 1) {
        console.log('\nNo deployed instances.')
      }
      if (id) {
        await chain.instances.select(id)
      } else if (list.length > 0) {
        console.log(`\nKnown instances:`)
        for (let instance of await chain.instances.list()) {
          if (instance === chain.instances.active.name) instance = bold(instance)
          console.log(`  ${instance}`)
        }
      }
      printActiveInstance()
    },

    deploy: {

      async all () {
        const vesting = await deployVesting({prefix, chain, admin})
        await chain.instances.select(vesting.prefix)
        await deploySwap(vesting)
        printActiveInstance()
      },

      async vesting () {
        const vesting = await deployVesting({prefix, chain, admin})
        await chain.instances.select(vesting.prefix)
        printActiveInstance()
      },

      async swap () {
        if (!chain.instances.active) await commands.deploy.vesting()
        const { name: prefix, getContract } = chain.instances.active
        const MGMT = getContract(MGMTContract, 'SiennaMGMT', admin)
        await deploySwap({ chain, admin, prefix, MGMT })
        printActiveInstance()
      }

    },

    upgrade: {
      async rewards (id?: string) {
        if (id) {
          await replaceRewardPool(chain, admin, id)
        } else {
          printRewardsContracts()
        }
      }
    }

  }

  return await runCommands(words, commands)


  /// Get an interface to the chain, and a deployment agent


  async function init (chainName: string) {
    const chains: Record<string, Function> = {
      'localnet-1.0': Scrt.localnet_1_0,
      'localnet-1.2': Scrt.localnet_1_2,
      'holodeck-2':   Scrt.holodeck_2,
      'supernova-1':  Scrt.supernova_1,
      'secret-2':     Scrt.secret_2,
      'secret-3':     Scrt.secret_3
    }

    if (!chainName) {
      console.log(`Select target chain:`)
      for (const chain of Object.keys(chains)) console.log(`  ${bold(chain)}`)
      process.exit(1)
    }

    const chain = await chains[chainName]().ready
    const admin = await chain.getAgent()
    console.log(`\nOperating on ${bold(chainName)} as ${bold(admin.address)}`)

    return { chain, admin }
  }


  /// Command parser


  async function runCommands (words: Array<string>, commands: Record<string, any>) {
    let command = commands
    let i: number
    for (i = 0; i < words.length; i++) {
      const word = words[i]
      if (typeof command === 'object' && command[word]) command = command[word]
      if (command instanceof Function) break
    }
    const context = await init(chainName)
    chain = context.chain
    admin = context.admin
    if (command instanceof Function) {
      return await Promise.resolve(command(...words.slice(i + 1)))
    } else {
      printActiveInstance()
      console.log(`\nAvailable commands:`)
      for (const key of Object.keys(command)) {
        console.log(`  ${bold(key)}`)
      }
    }
  }


  /// Select an instance or a reward contract


  function printActiveInstance () {
    if (chain && chain.instances.active) {
      console.log(`\nActive instance:`)
      console.log(`  ${bold(chain.instances.active.name)}`)
      for (const contract of Object.keys(chain.instances.active.contracts)) {
        console.log(`    ${colors.green('✓')}  ${contract}`)
      }
    }
  }

  function printRewardsContracts () {
    if (chain && chain.instances.active) {
      const {name, contracts} = chain.instances.active
      const isRewardPool = (x: string) => x.startsWith('SiennaRewards_')
      const rewardsContracts = Object.keys(contracts).filter(isRewardPool)
      if (rewardsContracts.length > 0) {
        console.log(`\nRewards contracts in ${bold(name)}:`)
        for (const name of rewardsContracts) {
          console.log(`  ${colors.green('✓')}  ${name}`)
        }
      } else {
        console.log(`\nNo rewards contracts.`)
      }
    } else {
      console.log(`\nSelect an instance to pick a reward contract.`)
    }
  }

}
