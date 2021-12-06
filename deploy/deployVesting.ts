import type { IChain, IAgent } from '@fadroma/ops'
import type { ScheduleFor_HumanAddr } from '@sienna/api/mgmt/handle'
import type { SwapOptions } from './deploySwap'
import { Scrt } from '@fadroma/scrt'
import { SiennaSNIP20, MGMTContract, RPTContract, AMMSNIP20 } from '@sienna/api'
import { getDefaultSchedule } from '../ops/index'
import { timestamp } from '@fadroma/tools'
import buildAndUpload from './buildAndUpload'

export type VestingOptions = {
  prefix?:   string
  chain?:    IChain,
  admin?:    IAgent,
  schedule?: ScheduleFor_HumanAddr
}

export default async function deployVesting (
  options: VestingOptions = {}
): Promise<SwapOptions> {

  const {
    prefix   = timestamp(),
    chain    = await new Scrt().ready,
    admin    = await chain.getAgent(),
    schedule = getDefaultSchedule()
  } = options

  const RPTAccount = getRPTAccount(schedule)
  const portion    = RPTAccount.portion_size

  const SIENNA = new SiennaSNIP20({ prefix, admin })
  const MGMT   = new MGMTContract({ prefix, admin, schedule, SIENNA })
  const RPT    = new RPTContract({ prefix, admin, MGMT, SIENNA, portion })

  await buildAndUpload([SIENNA, MGMT, RPT])

  await SIENNA.instantiate()

  if (chain.isTestnet) {
    await SIENNA.setMinters([admin.address], admin)
    await SIENNA.mint("5000000000000000000000", admin)
  }

  RPTAccount.address = admin.address
  await MGMT.instantiate()
  await MGMT.acquire(SIENNA)

  await RPT.instantiate()
  RPTAccount.address = RPT.address
  await MGMT.configure(schedule)

  await MGMT.launch()
  await RPT.vest()

  return { prefix, chain, admin, SIENNA, MGMT, RPT }

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