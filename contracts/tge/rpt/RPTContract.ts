import { Console, Agent, Scrt_1_2, Snip20Contract_1_2, bold } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
import { MGMTContract } from '@sienna/mgmt'
import { LinearMapAnd_Uint128 as LinearMap, Uint128 } from './schema/init'
import { RPTTransactions } from './RPTTransactions'
import { RPTQueries } from './RPTQueries'

export type RPTRecipient = string
export type RPTAmount    = string
export type RPTConfig    = [RPTRecipient, RPTAmount][]

const console = Console('@sienna/rpt')

import { RPTClient } from './RPTClient'
export class RPTContract extends Scrt_1_2.Contract {

  name = 'RPT'

  source = { workspace, crate: 'sienna-rpt' }

  Client = RPTClient

  /** Command. Print the status of the current deployment's RPT contract. */
  static status = rptStatus

  /** Command. After deploying reward pools, set their addresses
    * in the RPT for them to receive funding from the daily vesting. */
  static adjustConfig = adjustRPTConfig
}

async function rptStatus (context) {

  const {
    deployment, agent,
    RPT = deployment.get('RPT', RPTContract)
  } = context

  const status = await RPT.q().status()
  console.debug(`RPT status of ${bold(agent.address)}`, status)

}

async function adjustRPTConfig (context) {

  const {
    deployment, chain, agent,
    RPT = deployment.get('RPT', RPTContract),
    RPT_CONFIG,
  } = context

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

  console.info(bold(`Configuring RPT`), RPT.address)
  for (const [address, amount] of RPT_CONFIG) {
    console.info(` `, bold(amount), address)
  }

  await RPT.tx(agent).configure(RPT_CONFIG)

  return { RPT_CONFIG }

}
