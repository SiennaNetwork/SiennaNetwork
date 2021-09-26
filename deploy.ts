/// # Sienna Deployment


import settings from '@sienna/settings'
import type { Chain, Agent, ContractUpload } from '@fadroma/ops'
import { Scrt } from '@fadroma/scrt'
import { bold, timestamp, symlinkDir, randomHex } from '@fadroma/tools'
import process from 'process'
import { fileURLToPath } from 'url'
import { getDefaultSchedule, ONE_SIENNA } from './ops/index'


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
    SIENNA, MGMT, RPT,
  } = options

  const EXCHANGE  = new AMMContract({ prefix, admin })
      , AMMTOKEN  = new AMMSNIP20({ prefix, admin })
      , LPTOKEN   = new LPToken({ prefix, admin })
      , IDO       = new IDOContract({ prefix, admin })
      , REWARDS   = new RewardsContract({ prefix, admin })
      , LAUNCHPAD = new LaunchpadContract({ prefix, admin })
      , FACTORY   = new FactoryContract({
          prefix, admin, config: settings[`amm-${chain.chainId}`],
          EXCHANGE, AMMTOKEN, LPTOKEN, IDO, LAUNCHPAD
        })

  await buildAndUpload([EXCHANGE, AMMTOKEN, LPTOKEN, IDO, FACTORY, REWARDS, LAUNCHPAD])

  await FACTORY.instantiate()

  let tokens = {
    SIENNA,
    ...chain.isLocalnet
      ? await deployPlaceholderTokens()
      : getSwapTokens(settings[`swapTokens-${chain.chainId}`])
  }

  await deployRewardPool(SIENNA, SIENNA)

  const rptConfig = []

  for (const name of settings[`swapPairs-${chain.chainId}`]) {
    const [token0, token1] = await deploySwapPair(name)
    const rewardAllocation = settings[`rewardPairs-${chain.chainId}`]
    if (rewardAllocation) {
      const lpToken = await getLPToken(token0, token1)
      const pool = await deployRewardPool(lpToken, SIENNA)
      rptConfig.push([pool.address, String(BigInt(rewardAllocation * ONE_SIENNA))])
    }
  }

  await RPT.configure(rptConfig)


  /// On localnet, placeholder tokens need to be deployed.


  async function deployPlaceholderTokens () {
    const tokens = {}
    for (const token of settings[`placeholderTokens-${chain.chainId}`]) {
      tokens[token.symbol] = new AMMSNIP20({ prefix, ...token })
      await tokens[token.symbol].instantiate(admin)
    }
    return tokens
  }


  /// On testnet and mainnet, interoperate with preexisting token contracts.


  function getSwapTokens (links: Record<string, { address: string, codeHash: string }>) {
    const tokens = {}
    for (const [name, token] of Object.entries(links)) {
      tokens[name] = AMMSNIP20.attach(token)
    }
    return tokens
  }

  async function deploySwapPair (name: string) {
    const [tokenName0, tokenName1] = name.split('-')
    const token0 = tokens[tokenName0]
        , token1 = tokens[tokenName1]
    await FACTORY.createExchange(
      { contract_addr: token0.address, token_code_hash: token0.codeHash },
      { contract_addr: token1.address, token_code_hash: token1.codeHash }
    )
    return [token0, token1]
  }

  async function getLPToken (token0: SNIP20, token1: SNIP20) {
    return LPToken.attach(
      await AMMContract.attach({
        agent:   admin,
        address: (await FACTORY.listExchanges()).list_exchanges.exchanges
          .filter(({pair})=>(
            pair.token_0.custom_token.contract_addr === token0.address &&
            pair.token_1.custom_token.contract_addr === token1.address
          ))[0].address
      }).pairInfo().pair_info.liquidity_token.address,
      LPTOKEN.codeHash,
      admin)
  }

  async function deployRewardPool (lpToken: SNIP20, rewardToken: SNIP20) {
    const rewardPool = new RewardsContract({ prefix, admin, lpToken, rewardToken })
    await rewardPool.instantiate(this.agent)
    return rewardPool
  }

  async function replaceRewardPool () {}
}


/// ## Helper functions

/// ### Build and upload
/// Contracts can be built in parallel, but have to be uploaded in separate blocks.


async function buildAndUpload (contracts: Array<ContractUpload>) {
  await Promise.all(contracts.map(contract=>contract.build()))
  for (const contract of contracts) {
    await contract.upload()
    await contract.uploader.nextBlock
  }
}


/// ### Get the RPT account from the schedule
/// This is a special entry in MGMT's schedule that must be made to point to
/// the RPT contract's address - but that's only possible after deploying
/// the RPT contract. To prevent the circular dependency, the RPT account
/// starts as pointing to the admin's address.


function getRPTAccount (schedule: ScheduleFor_HumanAddr) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='RPT')[0]
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

  return await runCommands(words, {

    reset () {
      if (!chain.node) {
        throw new Error(`${bold(chainName)}: not a localnet`)
      }
      return chain.node.terminate()
    },

    select (id?: string) {
      if (id) {
        return chain.getInstance(id)
      } else {
        return chain.instances.list()
      }
    },

    deploy: {
      all () {
        return deployVesting({prefix, chain, admin}).then(deploySwap)
      },
      vesting () {
        console.log('deploy vesting')
        return deployVesting({prefix, chain, admin})
      },
      swap () {
        console.log('deploy swap')
        const MGMT = getSelectedVesting()
        return deploySwap({prefix, chain, admin, MGMT})
      }
    },

    migrate: {}

  })


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
        , admin = await chain.getAgent()
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
    if (command instanceof Function) {
      const context = await init(chainName)
      chain = context.chain
      admin = context.admin
      return command(words.slice(i + 1))
    } else {
      console.log(`\nAvailable commands:`)
      for (const key of Object.keys(command)) {
        console.log(`  ${bold(key)}`)
      }
    }
  }


  /// Instance picker


  function getActiveInstance () {
    if (chain.activeInstance) {
      console.log(`Active instance: ${bold(chain.activeInstance)}`)
      console.log(`Run ${bold("pnpm deploy select")} to pick another.`)
      return chain.activeInstance
    } else {
      const instances = chain.instances.list()
      console.log(`\nNo target instance selected.`)
      if (instances.length > 0) {
        console.log(
          `Select target instance by running:` +
          `\n  ${bold(`pnpm deploy ${chainName} select INSTANCE`)}` +
          `\nwhere INSTANCE is one of the following:`)
        for (const instance of Object.keys(chain.instances.list()))
          console.log(`  ${bold(instance)}`) }
      console.log(
        `\nDeploy a new instance by running ${bold(`pnpm deploy ${chainName} deploy vesting`)}, ` +
        `which will also set it as the selected instance.\n`)
      process.exit(1)
    }
  }

  function printInstances (chain: Chain) {
    console.log(chain.instances)
  }
}
