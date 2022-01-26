import { timestamp, Migration, waitUntilNextBlock, Deployment, Chain, Agent, bold, Console } from '@hackbg/fadroma'

const console = Console('@sienna/tge/deploy')

import type { ScheduleFor_HumanAddr } from '@sienna/mgmt/schema/handle.d'
import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract
} from '@sienna/api'

import settings, { workspace } from '@sienna/settings'

export type Inputs = Migration & {

  /** Input: The schedule for the new MGMT.
    * Defaults to production schedule. */
  schedule?: ScheduleFor_HumanAddr

}

export type Outputs = Migration & {

  /** Output: The deployed SIENNA SNIP20 token contract. */
  SIENNA: SiennaSNIP20Contract

  /** Output: The deployed MGMT contract. */
  MGMT:   MGMTContract

  /** Output: The deployed RPT contract. */
  RPT:    RPTContract

  /** Output: The newly created deployment. */
  deployment: Deployment

}

export async function deployTGE (inputs: Inputs): Promise<Outputs> {

  const {
    chain,
    admin,
    args = [],

    schedule = settings.schedule
  } = inputs

  console.info(bold('Admin balance:'), await admin.balance)

  // ignore deployment/prefix from the inputs;
  // always start new deployment
  const prefix = args[0] /* let user name it */ || timestamp() /* or default */
  await chain.deployments.create(prefix)
  await chain.deployments.select(prefix)
  const deployment = chain.deployments.active

  const RPTAccount = getRPTAccount(schedule)
  const portion    = RPTAccount.portion_size
  const options    = { uploader: admin, instantiator: admin, admin, workspace, chain, prefix }

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
  RPTAccount.address = RPT.address
  await MGMT.tx().configure(schedule)

  await MGMT.tx().launch()
  await RPT.tx().vest()

  return {
    ...inputs,
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
