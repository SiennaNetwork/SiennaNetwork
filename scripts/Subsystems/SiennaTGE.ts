import * as API from '@sienna/api'
import { Console, MigrationContext, buildAndUploadMany, bold, randomHex } from '@hackbg/fadroma'
import getSettings, { ONE_SIENNA } from '@sienna/settings'
import { versions, contracts, sources } from '../Build'
import { linkTuple } from '../misc'

const console = Console('Sienna TGE')

type VestingKind = 'tge' | 'vesting'

export interface Schedule {
  total: string,
  pools: Array<{
    name:     string
    total:    string
    partial:  boolean
    accounts: Array<{
      name:         string
      amount:       string
      address:      string
      start_at:     number
      interval:     number
      duration:     number
      cliff:        string
      portion_size: string
      remainder:    string
    }>
  }>
}

export interface VestingDeployOptions extends MigrationContext {
  /** Which kind of vesting to deploy **/
  version: VestingKind
  /** Address of the admin. */
  admin:   string
  /** The schedule for the new MGMT.
    * Defaults to production schedule. */
  settings?: { schedule?: Schedule }
}

export interface TGEDeployResult {
  /** The deployed MGMT contract. */
  MGMT:   API.MGMTClient
  /** The deployed RPT contract. */
  RPT:    API.RPTClient
  /** The deployed SIENNA SNIP20 token contract. */
  SIENNA: API.Snip20Client
}

export async function deployTGE (context: VestingDeployOptions): Promise<TGEDeployResult> {
  const {
    deployment,
    prefix,
    agent,
    admin = agent.address,
    settings: { schedule } = getSettings(agent.chain.mode)
    // 1. Build and upload the three TGE contracts:
    ref       = versions.TGE.legacy,
    srcs      = sources(ref, contracts.TGE),
    builder,
    uploader,
    templates: [
      tokenTemplate,
      mgmtTemplate,
      rptTemplate
    ] = await buildAndUploadMany(builder, uploader, srcs),
  } = context
  // 2. Instantiate the main token
  const tokenInitMsg = {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    config:    { public_total_supply: true },
    prng_seed: randomHex(36)
  }
  const tokenInstance = await deployment.init(agent, tokenTemplate, 'SIENNA', tokenInitMsg)
  // 3. Mutate the vesting schedule to use
  // the admin address as a temporary RPT address
  const tokenLink     = linkTuple(tokenInstance)
  const rptAccount    = Object.assign(getRPTAccount(schedule), { address: admin })
  const portion       = rptAccount.portion_size
  // 4. Instantiate the vesting contract (MGMT)
  const mgmtInitMsg   = { admin: admin, token: tokenLink, schedule}
  console.debug(mgmtInitMsg)
  const mgmtInstance  = await deployment.init(agent, mgmtTemplate, 'MGMT', mgmtInitMsg)
  const mgmtLink      = linkTuple(mgmtInstance)
  // 5. Instantiate the RPT contract
  const rptInstance   = await deployment.init(agent, rptTemplate, 'RPT', {
    portion,
    config: [[admin, portion]],
    token:  tokenLink,
    mgmt:   mgmtLink,
  })
  // 6. Set the RPT contract's account in schedule
  rptAccount.address = rptInstance.address
  const { isTestnet, isDevnet } = agent.chain
  await agent.bundle().wrap(async bundle=>{
    // 7. In non-production modes, mint some test tokens
    //    for the admin and other pre-defined accounts
    const token = new API.SiennaSnip20Client({...deployment.get('SIENNA'), agent: bundle})
    if (isTestnet||isDevnet) {
      console.warn(
        'Minting some test tokens '         +
        'for the admin and other testers. ' +
        '(Only for testnet and devnet!)'
      )
      await token.setMinters([admin])
      for (const addr of [ admin, ...testers ]) {
        const amount = "5000000000000000000000"
        console.warn(bold('Minting'), amount, bold('SIENNA'), 'to', bold(addr))
        await token.mint(amount, admin)
      }
    }
    // 8. MGMT becomes admin and sole minter of token,
    //    takes the final vesting config, and launches
    const mgmt = new API.MGMTClient['legacy']({ ...deployment.get('MGMT'), agent: bundle })
    await mgmt.acquire(token)
    await mgmt.configure(schedule)
    await mgmt.launch()
  })
  // 9. Return interfaces to the three contracts.
  //    This will add them to the context for
  //    subsequent steps. (Retrieves them through
  //    the Deployment to make sure the receipts
  //    were saved.)
  return {
    SIENNA: deployment.getClient(agent, API.SiennaSnip20Client,   'SIENNA'),
    MGMT:   deployment.getClient(agent, API.MGMTClient['legacy'], 'MGMT'),
    RPT:    deployment.getClient(agent, API.RPTClient['legacy'],  'RPT'),
  }
}

/** The **RPT account** (Remaining Pool Tokens) is a special entry 
  * in MGMT's vesting schedule; its funds are vested to **the RPT contract's address**,
  * and the RPT contract uses them to fund the Reward pools.
  * However, the RPT address is only available after deploying the RPT contract,
  * which in turn nees MGMT's address, therefore establishing a
  * circular dependency. To resolve it, the RPT account in the schedule
  * is briefly mutated to point to the deployer's address (before any funds are vested). */
export function getRPTAccount (schedule: Schedule) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='RPT')[0]
}

/** The **LPF account** (Liquidity Provision Fund) is an entry in MGMT's vesting schedule
  * which is vested immediately in full. On devnet and testnet, this can be used
  * to provide funding for tester accounts. In practice, testers are funded with an extra
  * mint operation in `deployTGE`. */
export function getLPFAccount (schedule: Schedule) {
  return schedule.pools
    .filter((x:any)=>x.name==='MintingPool')[0].accounts
    .filter((x:any)=>x.name==='LPF')[0]
}

export const testers = [
  //"secret1vdf2hz5f2ygy0z7mesntmje8em5u7vxknyeygy",
  "secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y",
  "secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty",
]

async function fundTesters ({ deployment, agent, cmdArgs }) {
  const [ vk = 'q1Y3S7Vq8tjdWXCL9dkh' ] = cmdArgs
  const sienna  = new API.SiennaSnip20Client({ ...deployment.get('SIENNA'), agent })
  const balanceBefore = await sienna.getBalance(agent.address, vk)
  console.info(`SIENNA balance of ${bold(agent.address)}: ${balanceBefore}`)
  const amount  = balanceBefore.slice(0, balanceBefore.length - 1)
  await sienna.transfer(amount, 'secret13nkfwfp8y9n226l9sy0dfs0sls8dy8f0zquz0y')
  await sienna.transfer(amount, 'secret1xcywp5smmmdxudc7xgnrezt6fnzzvmxqf7ldty')
  const balanceAfter = await sienna.getBalance(agent.address, vk)
  console.info(`SIENNA balance of ${bold(agent.address)}: ${balanceAfter}`)
}
