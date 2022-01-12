Error.stackTraceLimit = Infinity
import process from 'process'

import { bold, timestamp, runCommands, entrypoint } from '@fadroma/tools'

import { RPTContract } from '@sienna/api'

import init from './init'
import deployVesting from './deployVesting'
import deploySwap from './deploySwap'
import deployRewards from './deployRewards'
import replaceRewardPool, { printRewardsContracts } from './replaceRewardPool'
import rewardsAudit from './rewardsAudit'

const commands: Record<string, any> = {}

commands['reset'] = async function reset () {

  const {chain} = await init(process.env.CHAIN_NAME)
  if (!chain.node) {
    throw new Error(`${bold(process.env.CHAIN_NAME)}: not a localnet`)
  }

  return chain.node.terminate()

}

commands['select'] = async function select (id?: string) {

  const {chain} = await init(process.env.CHAIN_NAME)

  const list = chain.instances.list()
  if (list.length < 1) {
    console.log('\nNo deployed instances.')
  }

  if (id) {
    await chain.instances.select(id)
  } else if (list.length > 0) {
    console.log(`\nKnown instances:`)
    for (let instance of chain.instances.list()) {
      if (instance === chain.instances.active.name) instance = bold(instance)
      console.log(`  ${instance}`)
    }
  }

  chain.printActiveInstance()

}

commands['deploy'] = {

  async ['all'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const prefix = timestamp()
    const vesting = await deployVesting({prefix, chain, admin})
    await chain.instances.select(vesting.prefix)
    await deploySwap(vesting)
    chain.printActiveInstance()
  },

  async ['vesting'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const prefix = timestamp()
    const vesting = await deployVesting({prefix, chain, admin})
    await chain.instances.select(vesting.prefix)
    chain.printActiveInstance()
  },

  async ['swap'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    if (!chain.instances.active) await commands.deploy.vesting()
    const { name: prefix } = chain.instances.active
    await deploySwap({ chain, admin, prefix })
    chain.printActiveInstance()
  },

  async ['rewards-v2-and-v3'] () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    if (chain.isMainnet) {
      console.log('This command is not intended for mainnet.')
      process.exit(1)
    }
    if (!chain.instances.active) {
      console.log('Need to select an active instance for this command.')
      process.exit(1)
    }
    chain.printActiveInstance()
    const { name: prefix } = chain.instances.active
    const options = { chain, admin, prefix }
    const v2Suffix = `@v2+${timestamp()}`
    const v3Suffix = `@v3+${timestamp()}`
    const rptConfig = [
      ...await deployRewards({ ...options, suffix: v2Suffix, split: 0.5, ref: 'rewards-2.1.2' }),
      ...await deployRewards({ ...options, suffix: v3Suffix, split: 0.5, ref: 'HEAD' })
    ]
    console.log({rptConfig})
    const RPT = chain.instances.active.getContract(RPTContract, 'SiennaRPT', admin)
    await RPT.configure(rptConfig)
  }

}

commands['upgrade'] = {

  async ['rewards'] (id?: string) {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    if (id) {
      await replaceRewardPool(chain, admin, id)
    } else {
      printRewardsContracts(chain)
    }
  }

}

commands['audit'] = {

  rewards: rewardsAudit

}

export default async function main (
  [chainName, ...words]: Array<string>
) {

  // FIXME: a better way to pass the chain name
  // (reintroduce context object, minimally)
  process.env.CHAIN_NAME = chainName

  return await runCommands(
    commands,
    words,
    async (command: any) => {
      const { chain } = await init(chainName)
      chain.printIdentities()
      chain.printActiveInstance()
      console.log(`\nAvailable commands:`)
      for (const key of Object.keys(command)) {
        console.log(`  ${bold(key)}`)
      }
    }
  )
}

entrypoint(import.meta.url, main)
