import { Agent, Scrt_1_2, Console, bold } from '@hackbg/fadroma'
import { workspace, schedule } from '@sienna/settings'
import type { Init, Schedule } from './schema/init'
import { MGMTTransactions } from './MGMTTransactions'
import { MGMTQueries } from './MGMTQueries'

const console = Console('@sienna/mgmt')

export class MGMTContract extends Scrt_1_2.Contract<
  MGMTTransactions,
  MGMTQueries
> {
  workspace = workspace
  crate     = 'sienna-mgmt'
  name      = 'MGMT'
  initMsg?: Init
  Transactions = MGMTTransactions
  Queries      = MGMTQueries

  /** query current schedule */
  get schedule (): Promise<Schedule> {
    if (this.address) {
      return this.q().schedule()
    } else {
      return Promise.resolve(this.initMsg.schedule)
    }
  }

  set schedule (schedule: Schedule|Promise<Schedule>) {
    if (this.address) {
      throw new Error('Use the configure method to set the schedule of a deployed contract.')
    } else {
      Promise.resolve(schedule).then(schedule=>this.initMsg.schedule = schedule)
    }
  }

  /** Command. Print the current status of a deployed MGMT contract. */
  static status = async function mgmtStatus ({
    deployment, agent,
    MGMT = deployment.getThe('MGMT', new MGMTContract({ agent }))
  }) {
    try {
      const status = await MGMT.q().status()
      console.debug(`${bold(`MGMT status`)} of ${bold(MGMT.address)}`, status)
    } catch (e) {
      console.error(e.message)
    }
  }

  /** Command. Print an agent's status in the deployed MGMT contract. */
  static progress = async function mgmtProgress ({
    deployment, agent, address = agent.address,
    MGMT = deployment.getThe('MGMT', new MGMTContract({ agent }))
  }) {
    try {
      const progress = await MGMT.q().progress(address)
      console.debug(`${bold(`MGMT progress`)} of ${bold(address)} in ${MGMT.address}`, progress)
    } catch (e) {
      console.error(e.message)
    }
  }

}
