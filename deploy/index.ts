import type { IChain, IAgent } from '@fadroma/ops'
import { CHAINS } from '@fadroma/scrt'
import { bold, timestamp, runCommands, entrypoint } from '@fadroma/tools'
import process from 'process'
import { fileURLToPath } from 'url'

import deployVesting from './deployVesting'
import deploySwap from './deploySwap'
import replaceRewardPool, { printRewardsContracts } from './replaceRewardPool'

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

  async all () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const prefix = timestamp()
    const vesting = await deployVesting({prefix, chain, admin})
    await chain.instances.select(vesting.prefix)
    await deploySwap(vesting)
    chain.printActiveInstance()
  },

  async vesting () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    const prefix = timestamp()
    const vesting = await deployVesting({prefix, chain, admin})
    await chain.instances.select(vesting.prefix)
    chain.printActiveInstance()
  },

  async swap () {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    if (!chain.instances.active) await commands.deploy.vesting()
    const { name: prefix } = chain.instances.active
    await deploySwap({ chain, admin, prefix })
    chain.printActiveInstance()
  }

}

commands['upgrade'] = {

  async rewards (id?: string) {
    const {chain, admin} = await init(process.env.CHAIN_NAME)
    if (id) {
      await replaceRewardPool(chain, admin, id)
    } else {
      printRewardsContracts(chain)
    }
  }

}

export default async function main (
  [chainName, ...words]: Array<string>
) {
  process.env.CHAIN_NAME = chainName
  return await runCommands(
    commands,
    words,
    async (command: any) => {
      const { chain } = await init(chainName)
      chain.printActiveInstance()
      console.log(`\nAvailable commands:`)
      for (const key of Object.keys(command)) {
        console.log(`  ${bold(key)}`)
      }
    }
  )
}


async function init (chainName: string) {

  let chain: IChain
  let admin: IAgent

  if (!chainName || !Object.keys(CHAINS).includes(chainName)) {
    console.log(`Select target chain:`)
    for (const chain of Object.keys(CHAINS)) console.log(`  ${bold(chain)}`)
    process.exit(1)
  }

  chain = await CHAINS[chainName]().ready

  try {
    admin = await chain.getAgent()
    console.info(`Operating on ${bold(chainName)} as ${bold(admin.address)}`)
    const initialBalance = await admin.balance
    console.info(`Balance: ${bold(initialBalance)}uscrt`)
    process.on('beforeExit', async () => {
      const finalBalance = await admin.balance
      console.log(`\nInitial balance: ${bold(initialBalance)}uscrt`)
      console.log(`\nFinal balance: ${bold(finalBalance)}uscrt`)
      console.log(`\nConsumed gas: ${bold(String(initialBalance - finalBalance))}uscrt`)
    })
  } catch (e) {
    console.warn(`Could not get an agent for ${chainName}: ${e.message}`)
  }

  return { chain, admin }
}

entrypoint(import.meta.url, main)
