import {
  MigrationContext, printContracts, Deployment, Chain, Agent,
  bold, Console, randomHex, timestamp
} from '@hackbg/fadroma'

const console = Console('@sienna/tge/deploy')

import type { Schedule } from '@sienna/mgmt/schema/handle.d'
import {
  SiennaSnip20Contract, Snip20Client,
  MGMTContract,         MGMTClient,
  RPTContract,          RPTClient
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

export async function deployTGE (context: MigrationContext & {
  /** Input: The schedule for the new MGMT.
    * Defaults to production schedule. */
  schedule?: typeof settings.schedule,
}): Promise<{
  /** Output: The deployed SIENNA Snip20 token contract. */
  SIENNA:     Snip20Client
  /** Output: The deployed MGMT contract. */
  MGMT:       MGMTClient
  /** Output: The deployed RPT contract. */
  RPT:        RPTClient
}> {

  const {
    agent, deployment, prefix,
    schedule = settings.schedule,
  } = context

  const SIENNA = new SiennaSnip20Contract()
  const MGMT   = new MGMTContract()
  const RPT    = new RPTContract()

  await agent.buildAndUpload([SIENNA, MGMT, RPT])

  const admin = agent.address

  const siennaInitMsg = {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    config:    { public_total_supply: true },
    prng_seed: randomHex(36)
  }
  await deployment.instantiate(agent, [SIENNA, siennaInitMsg])
  const siennaLink = [SIENNA.instance.address, SIENNA.instance.codeHash] 

  const RPTAccount = getRPTAccount(schedule)
  RPTAccount.address = admin // mutate schedule
  const portion = RPTAccount.portion_size

  const mgmtInitMsg = { admin: admin, token: siennaLink, schedule }
  await deployment.instantiate(agent, [MGMT, mgmtInitMsg, 'MGMT'])
  const mgmtLink = [MGMT.instance.address, MGMT.instance.codeHash]

  const rptInitMsg = { token: siennaLink, mgmt: mgmtLink, portion, config: [[admin, portion]] } 
  await deployment.instantiate(agent, [RPT, rptInitMsg, 'RPT'])

  RPTAccount.address = RPT.instance.address

  const { isTestnet, isLocalnet } = agent.chain
  await agent.bundle().wrap(async bundle=>{
    const sienna = SIENNA.client(bundle)
    const mgmt   = MGMT.client(bundle)
    if (isTestnet||isLocalnet) {
      console.warn('Minting some test tokens for the admin and other testers. '+
                   'Only for testnet and localnet.')
      await sienna.setMinters([admin])
      for (const addr of [ admin, ...testers ]) {
        const amount = "5000000000000000000000"
        console.warn(bold('Minting'), amount, bold('SIENNA'), 'to', bold(addr))
        await sienna.mint(amount, admin)
      }
    }
    await mgmt.acquire(sienna)
    await mgmt.configure(schedule)
    await mgmt.launch()
  })

  return {
    SIENNA: SIENNA.client(agent),
    MGMT:   MGMT.client(agent),
    RPT:    RPT.client(agent)
  }

}

const testers = [
  "secret1vdf2hz5f2ygy0z7mesntmje8em5u7vxknyeygy",
  "secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y",
  "secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty",
]
