import { Console, bold } from '@hackbg/fadroma'
const console = Console('@sienna/mgmt')

import { Agent, Scrt_1_2 } from '@hackbg/fadroma'
import { workspace, schedule } from '@sienna/settings'
import type { Init, Schedule } from './schema/init'
import { MGMTClient } from './MGMTClient'
export { MGMTClient }
export class MGMTContract extends Scrt_1_2.Contract<MGMTClient> {
  name = 'MGMT'
  source = { workspace, crate: 'sienna-mgmt' }
  Client = MGMTClient
  initMsg?: Init

  /** Command. Print the current status of a deployed MGMT contract. */
  static status = mgmtStatus

  /** Command. Print an agent's status in the deployed MGMT contract. */
  static progress = mgmtProgress

}

async function mgmtStatus (context) {
  const { agent, deployment, MGMT = deployment.get('MGMT', MGMTContract) } = context
  try {
    const status = await MGMT.q().status()
    console.debug(`${bold(`MGMT status`)} of ${bold(MGMT.address)}`, status)
  } catch (e) {
    console.error(e.message)
  }
}

async function mgmtProgress (context) {
  const { agent, address = agent.address, deployment, MGMT = deployment.get('MGMT', MGMTContract) } = context
  try {
    const progress = await MGMT.q().progress(address)
    console.info(`${bold(`MGMT progress`)} of ${bold(address)} in ${MGMT.address}`)
    for (const [k,v] of Object.entries(progress)) console.info(' ', bold(k), v)
  } catch (e) {
    console.error(e.message)
  }
}
