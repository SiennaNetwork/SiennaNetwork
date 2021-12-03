import type { IChain, IAgent } from '@fadroma/ops'
import { bold } from '@fadroma/tools'
import { CHAINS } from '@fadroma/scrt'

export default async function init (chainName: string) {

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
