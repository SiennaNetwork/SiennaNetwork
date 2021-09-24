import type { Chain, Agent } from '@fadroma/ops'
import { Scrt } from '@fadroma/scrt'
import { bold, symlinkDir } from '@fadroma/tools'
import process from 'process'
import { fileURLToPath } from 'url'

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  main(process.argv.slice(2)).then(()=>process.exit(0))
}

/// ------------------------------------------------------------------------------------------------

import type { ScheduleFor_HumanAddr } from '@sienna/api/mgmt/handle'
import { SiennaSNIP20, MGMTContract, RPTContract } from '@sienna/api'
export type VestingOptions = { chain?: Chain, admin?: Agent, schedule?: ScheduleFor_HumanAddr }
export async function deployVesting (options: VestingOptions = {}): Promise<SwapOptions> {
  const { chain = await new Scrt().ready,
          admin = await chain.getAgent(),
          schedule } = options
  const SIENNA = new SiennaSNIP20({ admin })
      , MGMT   = new MGMTContract({ admin, schedule })
      , RPT    = new RPTContract({ admin, MGMT })
      , RPTAccount = getRPTAccount(schedule)
  await Promise.all([SIENNA, MGMT, RPT].map(contract=>contract.build()))
  await Promise.all([SIENNA, MGMT, RPT].map(contract=>contract.upload()))
  await SIENNA.instantiate()
  RPTAccount.address = admin.address
  await MGMT.instantiate()
  await RPT.instantiate()
  RPTAccount.address = RPT.address
  await MGMT.configure(schedule)
  await MGMT.launch()
  await RPT.vest()
  return { chain, admin, MGMT }
}

function getRPTAccount (schedule: ScheduleFor_HumanAddr) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='RPT')[0]
}

export function getSelectedVesting (chain: Chain) {}
export function selectVesting (chain: Chain, id: string) {}
export function printVestingInstances (chain: Chain) {}

/// ------------------------------------------------------------------------------------------------

import { FactoryContract, AMMContract, AMMSNIP20, LPToken, RewardsContract, IDOContract } from '@sienna/api'
export type SwapOptions = { chain?: Chain, admin?: Agent, MGMT?: MGMTContract, }
export async function deploySwap (options: SwapOptions = {}) {
  const { chain = await new Scrt().ready,
          admin = await chain.getAgent(),
          MGMT,
          swapConfig = loadSwapConfig() } = options
  const SIENNA   = SiennaSNIP20.attach((await MGMT.status()).token)
      , RPT      = RPTContract.attach(getRPTAccount(await MGMT.schedule()))
      , EXCHANGE = new AMMContract({ admin })
      , AMMTOKEN = new AMMSNIP20({ admin })
      , LPTOKEN  = new LPToken({ admin })
      , IDO      = new IDOContract({ admin })
      , FACTORY  = new FactoryContract({ admin, swapConfig, EXCHANGE, AMMTOKEN, LPTOKEN, IDO })
      , REWARDS  = new RewardsContract({ admin })
  await Promise.all([EXCHANGE, AMMTOKEN, LPTOKEN, IDO, FACTORY, REWARDS].map(contract=>contract.build()))
  await Promise.all([EXCHANGE, AMMTOKEN, LPTOKEN, IDO, FACTORY, REWARDS].map(contract=>contract.upload()))
  await FACTORY.instantiate()
}

export async function loadSwapConfig () {}

export async function addRewardPool () {}

export async function replaceRewardPool () {}

export function getSelectedSwap (chain: Chain) {}
export function selectSwap (chain: Chain, id: string) {}
export function printSwapInstances (chain: Chain) {}

/// ------------------------------------------------------------------------------------------------

export default async function main ([chainName, ...words]: Array<string>) {

  const { chain, admin } = await init(chainName)

  if (chain.activeInstance) {
    console.log(`Active instance: ${bold(chain.activeInstance)}`)
    console.log(`Run ${bold("pnpm deploy select")} to pick another.`)
  } else {
    console.log(`Select target instance by running ${bold("pnpm deploy select INSTANCE")}`)
    console.log(`where INSTANCE is one of the following:`)
    for (const instance of chain.instances) console.log(`  ${bold(instance)}`)
    process.exit(1)
  }

  return await runCommands(words, {
    select (id?: string) {
      if (id) {
        return chain.getInstance(id)
      } else {
        return chain.instances.list()
      }
    },
    deploy: {
      all () {
        console.log('deploy all')
      },
      vesting () {
        console.log('deploy vesting')
        return deployVesting({ chain, admin })
      },
      swap () {
        console.log('deploy swap')
        const MGMT = getSelectedVesting()
        return deploySwap({ chain, admin, MGMT })
      }
    },
    migrate: {}
  })

  async function init (chainName: string) {
    const chains = ['localnet-1.2', 'localnet-1.0', 'holodeck-2', 'supernova-1', 'secret-2', 'secret-3']
    console.log({chainName})
    if (!chainName) {
      console.log(`Select target chain:`)
      for (const chain of chains) console.log(`  ${bold(chain)}`)
      process.exit(1)
    }
    const chain = await new Scrt[chainName]().ready
        , admin = await chain.getAgent()
    console.log(`Operating on ${bold(chainName)} as ${bold(admin.address)}`)
    return { chain, admin }
  }

  function runCommands (words: Array<string>, commands: Record<string, any>) {
    let command = commands
    let fragment: string|undefined = words.shift()
    while (fragment) {
      if (command[fragment] instanceof Function) break
      try {
        command = command[fragment]
      } catch (e) {
        throw new Error(`invalid command: ${fragment}`)
      }
    }
    if (command[fragment] instanceof Function) {
      return command[fragment](...words)
    } else {
      throw new Error(`invalid command: ${fragment}`)
    }
  }

  function printInstances (chain: Chain) {
    console.log(chain.instances)
  }
}
