import { Console, Agent, Scrt_1_2, SNIP20Contract_1_2, bold } from '@hackbg/fadroma'
import { workspace } from '@sienna/settings'
import { MGMTContract } from '@sienna/mgmt'
import { LinearMapAnd_Uint128 as LinearMap, Uint128 } from './schema/init'
import { RPTTransactions } from './RPTTransactions'
import { RPTQueries } from './RPTQueries'

export type RPTRecipient = string
export type RPTAmount    = string
export type RPTConfig    = [RPTRecipient, RPTAmount][]

const console = Console('@sienna/rpt')

export class RPTContract extends Scrt_1_2.Contract<RPTTransactions, RPTQueries> {
  workspace = workspace
  crate = 'sienna-rpt'
  name  = 'RPT'

  Transactions = RPTTransactions
  Queries      = RPTQueries

  /** Query contract status */
  get status () {
    return this.q().status().then(({status})=>status)
  }

  /** Command. Print the status of the current deployment's RPT contract. */
  static status = async function rptStatus ({
    deployment, agent,
    RPT = deployment.getThe('RPT', new RPTContract({ agent }))
  }) {
    const status = await RPT.q().status()
    console.debug(`RPT status of ${bold(agent.address)}`, status)
  }

  /** Command. After deploying reward pools, set their addresses
    * in the RPT for them to receive funding from the daily vesting. */
  static adjustConfig = async function adjustRPTConfig ({
    deployment, chain, agent,
    RPT = deployment.getThe('RPT', new RPTContract({ agent })),
    RPT_CONFIG,
  }) {
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
      console.info(` `, bold(amount), address)
    }
    await RPT.tx(agent).configure(RPT_CONFIG)
    return { RPT_CONFIG }
  }
}
