import { randomBytes } from 'crypto'
import Ensemble from '@fadroma/scrt-ops/ensemble.js'
import { render, Console } from '@fadroma/utilities'

import getDefaultSchedule from './getDefaultSchedule.js'
import { abs } from './root.js'
import { combine, args } from './args.js'
import { genConfig } from './gen.js'

import { SNIP20Contract, MGMTContract, RPTContract } from '@sienna/api'

const { log, warn, info, table } = Console(import.meta.url)

export default class TGEContracts extends Ensemble {

  workspace = abs()

  prefix = `${new Date().toISOString()} `

  contracts = {

    TOKEN: {
      crate:   'snip20-sienna',
      schema:  'schema',
      label:   `${this.prefix}SIENNA_SNIP20`,
      initMsg: {
        prng_seed: randomBytes(36).toString('hex'),
        name:     "Sienna",
        symbol:   "SIENNA",
        decimals: 18,
        config: { public_total_supply: true }
      }
    },

    MGMT: {
      crate:   'sienna-mgmt',
      schema:  'mgmt_schema',
      label:   `${this.prefix}SIENNA_MGMT`,
      initMsg: {}
    },

    RPT: {
      crate:   'sienna-rpt',
      schema:  'rpt_schema',
      label:   `${this.prefix}SIENNA_RPT`,
      initMsg: {}
    }

  }
 
  async initialize (options = {}) {
    // idempotency support
    // passing existing `contracts` to this makes it a no-op
    const { contracts = {} } = options
    if (Object.keys(contracts)>0) return contracts

    // unwrap mutable options
    let { agent
        , network  = agent ? {network: agent.network} : await SecretNetwork.localnet({stateBase})
        , schedule = getDefaultSchedule
        } = options

    // accepts schedule as string or struct
    if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))
    //log(render(schedule))

    // if `network` is just the connection type, replace it with a real connection
    if (typeof network === 'string') {
      network = conformChainIdToNetwork(network)
      network = await SecretNetwork[network]({stateBase})
    }

    // if there's no agent, use the default one from the connection
    if (!agent) agent = network.agent

    // unwrap remaining options
    const { task                = taskmaster()
          , receipts            = await upload({agent, network, task})
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

    return contracts
  }

  async launch (options = {}) {
    let { network
        , address
        } = options
    if (typeof network === 'string') {
      network = conformChainIdToNetwork(network)
      network = (await SecretNetwork[network]({stateBase}))
    }
    const MGMT = network.network.getContract(MGMTContract, address, network.agent)
    info(`⏳ launching contract ${address}...`)
    try {
      await MGMT.launch()
      info(`🟢 launch reported success`)
    } catch (e) {
      warn(e)
      info(`🔴 launch reported a failure`)
    }
    info(`⏳ querying status...`)
    log(render(await MGMT.status))
  }

  async reallocate () { throw new Error('not implemented') }

  async addAccount () { throw new Error('not implemented') }

  async claim (options = {}) {
    const { claimant = await pickKey()
          } = options
    let { network = 'localnet'
        } = options
    if (typeof network === 'string') {
      network = conformChainIdToNetwork(network)
      network = await SecretNetwork[network]({stateBase})
    }
    log({network, claimant})
  }

  commands (yargs) {
    return yargs
      .command('build',
        '👷 Compile contracts from working tree',
        args.Sequential, () => this.build())
      .command('deploy-tge [network] [schedule]',
        '🚀 Build, init, and deploy the TGE',
        combine(args.Network, args.Schedule),
        x => this.deploy(x).then(info))
      .command('upload <network>',
        '📦 Upload compiled contracts to network',
        args.Network,
        () => this.upload())
      .command('init <network> [<schedule>]',
        '🚀 Just instantiate uploaded contracts',
        combine(args.Network, args.Schedule),
        x => this.initialize(x).then(info))
      .command('launch <network> <address>',
        '🚀 Launch deployed vesting contract',
        combine(args.Network, args.Address),
        () => this.launch())
      .command('transfer <network> <address>',
        '⚡ Transfer ownership to another address',
        combine(args.Network, args.Address),
        () => this.transfer())
      .command('configure <deployment> <schedule>',
        '⚡ Upload a JSON config to an initialized contract',
        combine(args.Deployment, args.Schedule),
        () => this.configure())
      .command('reallocate <deployment> <allocations>',
        '⚡ Update the allocations of the RPT tokens',
        combine(args.Deployment, args.Allocations),
        () => this.reallocate())
      .command('add-account <deployment> <account>',
        '⚡ Add a new account to a partial vesting pool',
        combine(args.Deployment, args.Account),
        () => this.addAccount())
      .command('claim <network> <contract> [<claimant>]',
        '⚡ Claim funds from a deployed contract',
        combine(args.Network, args.Contract, args.Claimant),
        () => this.claim())
      .command('config [<spreadsheet>]',
        '📅 Convert a spreadsheet into a JSON schedule',
        args.Spreadsheet, genConfig)
  }

}
