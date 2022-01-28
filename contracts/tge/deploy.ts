import {
  MigrationContext, printContracts, Deployment, Chain, Agent,
  bold, Console, randomHex, timestamp
} from '@hackbg/fadroma'

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
  /** Output: Root directory for building the contracts. */
  workspace:  string
  /** Output: The newly created deployment. */
  deployment: Deployment
  /** Output: The identifier of the deployment on- and off-chain. */
  prefix:     string
  /** Output: The deployed SIENNA SNIP20 token contract. */
  SIENNA:     SiennaSNIP20Contract
  /** Output: The deployed MGMT contract. */
  MGMT:       MGMTContract
  /** Output: The deployed RPT contract. */
  RPT:        RPTContract
}> {

  console.info(bold('Admin balance:'), await admin.balance)

  const [SIENNA, MGMT, RPT] = await chain.buildAndUpload(admin, [
    new SiennaSNIP20Contract({ workspace }),
    new MGMTContract({         workspace }),
    new RPTContract({          workspace })
  ])

  await deployment.createContract(admin, SIENNA, {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    config:    { public_total_supply: true },
    prng_seed: randomHex(36)
  })

  if (chain.isTestnet) {
    await SIENNA.tx(admin).setMinters([admin.address])
    await SIENNA.tx(admin).mint("5000000000000000000000", admin.address)
  }

  const RPTAccount = getRPTAccount(schedule)
  RPTAccount.address = admin.address // mutate schedule
  const portion    = RPTAccount.portion_size

  await deployment.createContract(admin, MGMT, {
    admin: admin.address,
    token: [SIENNA.address, SIENNA.codeHash],
    schedule
  })

  await MGMT.tx().acquire(SIENNA)

  await deployment.createContract(admin, RPT, {
    token:   [SIENNA.address, SIENNA.codeHash],
    mgmt:    [MGMT.address, MGMT.codeHash],
    portion: RPTAccount.portion_size,
    config:  [[admin.address, RPTAccount.portion_size]]
  })

  console.log()
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
