import { IChain, IAgent, timestamp } from '@hackbg/fadroma'

import type { ScheduleFor_HumanAddr } from '@sienna/mgmt/schema/handle.d'
import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract
} from '@sienna/api'

import settings, { abs } from '@sienna/settings'

import type { SwapOptions } from './deploySwap'

export type VestingOptions = {
  workspace?: string
  prefix?:    string
  chain?:     IChain
  admin?:     IAgent
  schedule?:  ScheduleFor_HumanAddr
}

export async function deployVesting (
  options: VestingOptions = {}
): Promise<SwapOptions> {

  const {
    workspace = abs(),
    prefix    = timestamp(),
    chain,
    admin     = await chain.getAgent(),
    schedule  = settings.schedule
  } = options

  const RPTAccount = getRPTAccount(schedule)
  const portion    = RPTAccount.portion_size

  const contractOptions = { uploader: admin, instantiator: admin, admin, workspace, chain, prefix }
  const SIENNA = new SiennaSNIP20Contract({ ...contractOptions })
  const MGMT   = new MGMTContract({ ...contractOptions, schedule, SIENNA })
  const RPT    = new RPTContract({ ...options, MGMT, SIENNA, portion })

  SIENNA.uploader = MGMT.uploader = RPT.uploader = admin
  await chain.buildAndUpload([SIENNA, MGMT, RPT])

  SIENNA.instantiator = MGMT.instantiator = RPT.instantiator = admin
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

  return { workspace, prefix, chain, admin, SIENNA, MGMT, RPT }

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
