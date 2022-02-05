import {
  MigrationContext, printContracts, Deployment, Chain, Agent,
  bold, Console, randomHex, timestamp
} from '@hackbg/fadroma'

const console = Console('@sienna/tge/deploy')

import type { Schedule } from '@sienna/mgmt/schema/handle.d'
import {
  SiennaSNIP20Contract,
  MGMTContract,
  RPTContract
} from '@sienna/api'

import * as settings from '@sienna/settings'

// This is a special entry in MGMT's schedule that must be made to point to
// the RPT contract's address - but that's only possible after deploying
// the RPT contract. To prevent the circular dependency, the RPT account
// starts as pointing to the admin's address.
export function getRPTAccount (schedule: Schedule) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='RPT')[0] }

// This is an entry in the schedule which is vested immediately.
// On localnet and testnet, it's split between several addresses.
export function getLPFAccount (schedule: Schedule) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='LPF')[0] }

export async function deployTGE ({
  chain, agent, deployment, prefix,
  schedule = settings.schedule
}: MigrationContext & {
  /** Input: The schedule for the new MGMT.
    * Defaults to production schedule. */
  schedule?: typeof settings.schedule
}): Promise<{
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
  const [SIENNA, MGMT, RPT] = await chain.buildAndUpload(agent, [
    new SiennaSNIP20Contract(),
    new MGMTContract(),
    new RPTContract()
  ])
  const admin = agent.address
  await deployment.init(agent, SIENNA, {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    config:    { public_total_supply: true },
    prng_seed: randomHex(36)
  })
  const RPTAccount = getRPTAccount(schedule)
  RPTAccount.address = admin // mutate schedule
  const portion = RPTAccount.portion_size
  await deployment.init(agent, MGMT, {
    admin: admin,
    token: [SIENNA.address, SIENNA.codeHash],
    schedule
  })
  await deployment.init(agent, RPT, {
    token:   [SIENNA.address, SIENNA.codeHash],
    mgmt:    [MGMT.address, MGMT.codeHash],
    portion,
    config:  [[admin, portion]]
  })
  RPTAccount.address = RPT.address
  await agent.bundle(async agent=>{ // once async always async
    const sienna = SIENNA.tx(agent)
    if (chain.isTestnet || chain.isLocalnet) {
      console.warn(
        'Minting some test tokens for the admin and other testers. Only for testnet and localnet.'
      )
      await sienna.setMinters([admin])
      for (const addr of [
        admin,
        "secret1vdf2hz5f2ygy0z7mesntmje8em5u7vxknyeygy",
        "secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y",
        "secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty",
      ]) {
        const amount = "5000000000000000000000"
        console.warn(bold('Minting'), amount, 'to', bold(addr))
        await sienna.mint(amount, admin)
      }
    }
    const mgmt = MGMT.tx(agent)
    await mgmt.acquire(SIENNA)
    await mgmt.configure(schedule)
    await mgmt.launch()
  })
  // not needed
  //console.info(bold('Vesting RPT'))
  //await RPT.tx().vest()

  return {
    deployment,
    prefix,
    SIENNA,
    MGMT,
    RPT
  }
}
