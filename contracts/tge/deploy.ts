import {
  MigrationContext, timestamp, printContracts, Deployment, Chain, Agent, bold, Console } from '@hackbg/fadroma'

const console = Console('@sienna/amm/upgrade')

import type { ScheduleFor_HumanAddr } from '@sienna/mgmt/schema/handle.d'
import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract
} from '@sienna/api'

import settings, { workspace } from '@sienna/settings'

export async function deployTGE ({
  chain, admin, deployment, prefix,
  schedule = settings.schedule
}: MigrationContext & {
  /** Input: The schedule for the new MGMT.
    * Defaults to production schedule. */
  schedule?: typeof settings.schedule
}): Promise<{
  workspace:  string
  timestamp:  string
  /** Output: The newly created deployment. */
  deployment: Deployment
  prefix:     string
  /** Output: The deployed SIENNA SNIP20 token contract. */
  SIENNA:     SiennaSNIP20Contract
  /** Output: The deployed MGMT contract. */
  MGMT:       MGMTContract
  /** Output: The deployed RPT contract. */
  RPT:        RPTContract
}> {
  console.info(bold('Admin balance:'), await admin.balance)
  const RPTAccount = getRPTAccount(schedule)
  const portion    = RPTAccount.portion_size
  const options    = { uploader: admin, creator: admin, admin, workspace, chain, prefix }
  const SIENNA     = new SiennaSNIP20Contract({ ...options })
  const MGMT       = new MGMTContract({ ...options, schedule, SIENNA })
  const RPT        = new RPTContract({ ...options, MGMT, SIENNA, portion })
  await chain.buildAndUpload([SIENNA, MGMT, RPT])
  await SIENNA.instantiate()
  if (chain.isTestnet) {
    await SIENNA.tx(admin).setMinters([admin.address])
    await SIENNA.tx(admin).mint("5000000000000000000000", admin.address)
  }
  RPTAccount.address = admin.address
  await MGMT.instantiate()
  await MGMT.tx().acquire(SIENNA)
  await RPT.instantiate()
  console.info(bold('Deployed TGE contracts:'))
  printContracts([SIENNA, MGMT, RPT])
  console.info(bold('Setting TGE schedule'))
  RPTAccount.address = RPT.address
  await MGMT.tx().configure(schedule)
  console.info(bold('Launching the TGE'))
  await MGMT.tx().launch()
  console.info(bold('Vesting RPT'))
  await RPT.tx().vest()
  return {
    workspace,
    deployment,
    timestamp,
    prefix,
    SIENNA,
    MGMT,
    RPT
  }
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
