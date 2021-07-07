import { SecretNetwork } from '@fadroma/scrt-agent'
import { ScrtEnsemble } from '@fadroma/scrt-ops'
import {
  Console, render,
  readFileSync, randomBytes,
  resolve, basename, extname, dirname,
  stderr
} from '@fadroma/utilities'
import { SNIP20Contract, MGMTContract, RPTContract } from '@sienna/api'
import { scheduleFromSpreadsheet } from '@sienna/schedule'
import { projectRoot, abs, runDemo } from './lib/index.js'

const { debug, log, warn, error, info, table } = Console(import.meta.url)

export default class TGEContracts extends ScrtEnsemble {

  workspace = abs()

  prefix = `${new Date().toISOString()} `

  contracts = {
    TOKEN: {
      crate:   'snip20-sienna',
      label:   `${this.prefix}SIENNA_SNIP20`,
      initMsg: {
        prng_seed: randomBytes(36).toString('hex'),
        name:     "Sienna",
        symbol:   "SIENNA",
        decimals: 18,
        config: { public_total_supply: true }
      }
    },
    MGMT: { crate: 'sienna-mgmt', label: `${this.prefix}SIENNA_MGMT`, initMsg: {} },
    RPT:  { crate: 'sienna-rpt',  label: `${this.prefix}SIENNA_RPT`,  initMsg: {} }
  }

  get localCommands () {
    return [
      ["build",       '👷 Compile contracts from working tree',
        (_, sequential) => this.build(sequential)],
      ['config',      '📅 Convert a spreadsheet into a JSON schedule',
        (_, spreadsheet) => genConfig(spreadsheet)]
    ]
  }

  get remoteCommands () {
    return [
      ["deploy",       '🚀 Build, init, and deploy the TGE',
        (context, schedule) => this.deploy({...context, schedule}).then(process.exit)],
      ["demo",         '🐒 Run the TGE demo (long-running integration test)',
        runDemo],
      ["upload",       '📦 Upload compiled contracts to network',
        (context)           => this.upload(context)],
      ["init",         '🚀 Init new instances of already uploaded contracts',
        (context, schedule) => this.initialize({...context, schedule})],
      ["launch",       '🚀 Launch deployed vesting contract',
        (context, address)  => this.launch({...context, address})],
      ["transfer",     '⚡ Transfer ownership of contracts to another address',
        (context, address)  => this.transfer({...context, address})],
      ['claim',        '⚡ Claim funds from a deployed contract',
        (context, address, claimant) => this.claim({...context, address, claimant})],
      ['status',       '👀 Print the status and schedule of a contract.',
        (context, address) => this.getStatus({...context, address})],
      //
      // not implemented:
      //
      //["configure",   '⚡ Upload a new JSON config to an already initialized contract',
        //(context, deployment, schedule) => this.configure(deployment, schedule)],
      //['reallocate',  '⚡ Update the allocations of the RPT tokens',
        //(context, [deployment, allocations]) => this.reallocate(deployment, allocations)],
      //['add-account', '⚡ Add a new account to a partial vesting pool',
        //(context, [deployment, account]) => this.addAccount(deployment, account)],
    ]
  }
 
  async initialize (options = {}) {
    const network = SecretNetwork.hydrate(options.network || this.network)
    const agent = options.agent || this.agent || await network.getAgent()
    // idempotency support
    // passing existing `contracts` to this makes it a no-op
    const { contracts = {} } = options
    if (Object.keys(contracts)>0) return contracts

    // accepts schedule as string or struct
    let { schedule = getDefaultSchedule() } = options
    if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))
    //log(render(schedule))

    // unwrap remaining options
    const { task                = taskmaster()
          , receipts            = await this.upload({agent, network, task})
          , inits               = this.contracts
          , initialRPTRecipient = agent.address
          } = options

    // too many steps - mgmt could automatically instantiate token and rpt if it supported callbacks
    await task('initialize token', async report => {
      const {codeId} = receipts.TOKEN, {label, initMsg} = inits.TOKEN
      initMsg.admin = agent.address
      contracts.TOKEN = await agent.instantiate(new SNIP20Contract({codeId, label, initMsg}))
      report(contracts.TOKEN.initTx.transactionHash)
    })

    await task('initialize mgmt', async report => {
      const {codeId} = receipts.MGMT, {label, initMsg} = inits.MGMT
      initMsg.token    = [contracts.TOKEN.address, contracts.TOKEN.codeHash]
      initMsg.schedule = schedule
      console.log({schedule})
      schedule.pools.filter(x=>x.name==='MintingPool')[0]
              .accounts.filter(x=>x.name==='RPT')[0]
              .address = agent.address
      contracts.MGMT = await agent.instantiate(new MGMTContract({codeId, label, initMsg}))
      report(contracts.MGMT.initTx.transactionHash)
    })

    await task('make mgmt owner of token', async report => {
      const {MGMT, TOKEN} = contracts
          , [tx1, tx2] = await MGMT.acquire(TOKEN)
      report(tx1.transactionHash)
      report(tx2.transactionHash)
    })

    await task('initialize rpt', async report => {
      const {codeId} = receipts.RPT, {label, initMsg} = inits.RPT, {MGMT, TOKEN} = contracts
      initMsg.token   = [TOKEN.address, TOKEN.codeHash]
      initMsg.mgmt    = [MGMT.address,  MGMT.codeHash ]
      initMsg.portion = "2500000000000000000000" // TODO get this from schedule!!!
      initMsg.config  = [[initialRPTRecipient, initMsg.portion]]
      contracts.RPT = await agent.instantiate(new RPTContract({ codeId, label, initMsg }))
      report(contracts.RPT.initTx.transactionHash)
    })

    await task('point rpt account in mgmt schedule to rpt contract', async report => {
      const {MGMT, RPT} = contracts
      schedule.pools.filter(x=>x.name==='MintingPool')[0]
              .accounts.filter(x=>x.name==='RPT')[0]
              .address = RPT.address
      const {transactionHash} = await MGMT.configure(schedule)
      report(transactionHash)
    })

    table([
      ['Contract\nDescription',      'Address\nCode hash'],
      ['TOKEN\nSienna SNIP20 token', `${contracts.TOKEN.address}\n${contracts.TOKEN.codeHash}`],
      ['MGMT\nVesting',              `${contracts.MGMT.address}\n${contracts.MGMT.codeHash}`],
      ['RPT\nRemaining pool tokens', `${contracts.RPT.address}\n${contracts.RPT.codeHash}`]
    ])

    return {network, agent, contracts}
  }

  async launch (options = {}) {
    const address = options.address
    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      process.exit(1)
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
    }

    info(`⏳ launching vesting MGMT contract at ${address}...`)
    const network = SecretNetwork.hydrate(options.network || this.network)
    const { agent } = await network.connect()
    const MGMT = network.getContract(MGMTContract, address, agent)

    try {
      await MGMT.launch()
      info(`🟢 launch reported success`)
      info(`⏳ querying status...`)
      debug(await MGMT.status)
    } catch (e) {
      warn(e)
      info(`🔴 launch reported a failure`)
    }
  }

  async getStatus (options = {}) {
    const address = options.address
    if (!address) {
      warn('TGE launch: needs address of deployed MGMT contract')
      process.exit(1)
      // TODO add `error.user = true` flag to errors
      // to be able to discern between bugs and incorrect inputs
    }
    info(`⏳ querying MGMT contract at ${address}...`)
    const network = SecretNetwork.hydrate(options.network || this.network)
    const { agent } = await network.connect()
    const MGMT = network.getContract(MGMTContract, address, agent)
    const [schedule, status] = await Promise.all([MGMT.schedule, MGMT.status])
    console.log('\n'+render(schedule))
    console.log('\n'+render(status))
  }

  async reallocate () { throw new Error('not implemented') }

  async addAccount () { throw new Error('not implemented') }

  async claim (options = {}) { throw new Error('not implemented') }

}

export function getDefaultSchedule () {
  const path = resolve(projectRoot, 'settings', 'schedule.json')
  try {
    return JSON.parse(readFileSync(path, 'utf8'))
  } catch (e) {
    console.warn(`${path} does not exist - "./sienna.js config" should create it`)
    return null
  }
}

export function genConfig (options = {}) {
  const { file = abs('settings', 'schedule.ods')
        } = options

  stderr.write(`\n⏳ Importing configuration from ${file}...\n\n`)
  const name       = basename(file, extname(file)) // path without extension
  const schedule   = scheduleFromSpreadsheet({ file })
  const serialized = stringify(schedule)
  const output     = resolve(dirname(file), `${name}.json`)
  stderr.write(`⏳ Saving configuration to ${output}...\n\n`)

  writeFileSync(output, stringify(schedule), 'utf8')
  stderr.write(`🟢 Configuration saved to ${output}\n`)
}

function stringify (data) {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent)
}
