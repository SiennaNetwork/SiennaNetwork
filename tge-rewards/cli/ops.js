import { randomBytes } from 'crypto'
import { stderr } from 'process'
import { readdirSync, readFileSync, existsSync } from 'fs'
import assert from 'assert'

import bignum from 'bignumber.js'
import { table } from 'table'
import { render } from 'prettyjson'

import { SNIP20Contract, MGMTContract, RPTContract, RewardsContract } from '../api/index.js'

import { taskmaster, SecretNetwork } from '@hackbg/fadroma'
import ContractEnsemble from '@hackbg/fadroma/js/SecretNetwork/Ensemble.js'
import ensureWallets from '@hackbg/fadroma/js/SecretNetwork/ensureWallets.js'
import { pull } from '@hackbg/fadroma/js/net.js'
import { fileURLToPath, resolve, basename, extname, dirname
       , readFile, writeFile } from '@hackbg/fadroma/js/sys.js'

import { conformChainIdToNetwork, conformNetworkToChainId
       , pickNetwork, pickInstance, pickKey } from './pick.js'
import { projectRoot, abs } from './root.js'

export const stateBase = abs('artifacts')

// decimals
export const fmtDecimals = d => x =>
  `${bignum(x).div(d).toString()}.${bignum(x).mod(d).toString().padEnd(18, '0')}`

export const SIENNA_DECIMALS = 18
export const ONE_SIENNA = bignum(`1${[...Array(SIENNA_DECIMALS)].map(()=>`0`).join('')}`)
export const fmtSIENNA = fmtDecimals(ONE_SIENNA)

export const SCRT_DECIMALS = 6
export const ONE_SCRT = bignum(`1${[...Array(SCRT_DECIMALS)].map(()=>`0`).join('')}`)
export const fmtSCRT = fmtDecimals(ONE_SCRT)

export { ensureWallets }

export const getDefaultSchedule = () => {
  const path = resolve(projectRoot, 'settings', 'schedule.json')
  try {
    JSON.parse(readFileSync(path, 'utf8'))
  } catch (e) {
    console.warn(`${path} does not exist - "./sienna.js config" should create it`)
    return null
  }
}

const prefix = new Date().toISOString().replace(/[-:\.]/g, '-').replace(/[TZ]/g, '_')

const prng_seed = randomBytes(36).toString('hex')

export class TGEContracts extends ContractEnsemble {

  static workspace = abs()

  static contracts = {
    TOKEN:
      { crate:   'snip20-reference-impl'
      , schema:  'schema'
      , label:   `${prefix}SIENNA_SNIP20`
      , initMsg:
        { prng_seed
        , name:     "Sienna"
        , symbol:   "SIENNA"
        , decimals: 18
        , config:   { public_total_supply: true } } },
    MGMT:
      { crate:   'sienna-mgmt'
      , schema:  'mgmt_schema'
      , label:   `${prefix}SIENNA_MGMT`
      , initMsg: {} },
    RPT:
      { crate:   'sienna-rpt'
      , schema:  'rpt_schema'
      , label:   `${prefix}SIENNA_RPT`
      , initMsg: {} } }
 
  static async initialize (options = {}) {
    // idempotency support
    // passing existing `contracts` to this makes it a no-op
    const { contracts = {} } = options
    if (Object.keys(contracts)>0) return contracts

    // unwrap mutable options
    let { agent
        , network = agent ? {network: agent.network} : await SecretNetwork.localnet({stateBase})
        , schedule
        } = options

    // accepts schedule as string or struct
    if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))
    console.log(render(schedule))

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

    // too many steps - mgmt could automatically instantiate token and rpt
    await task('initialize token', async report => {
      const {codeId} = receipts.TOKEN, {label, initMsg} = inits.TOKEN
      initMsg.admin = agent.address
      contracts.TOKEN = await SNIP20Contract.init({agent, codeId, label, initMsg})
      report(contracts.TOKEN.transactionHash) })
    await task('initialize mgmt', async report => {
      const {codeId} = receipts.MGMT, {label, initMsg} = inits.MGMT
      initMsg.token    = [contracts.TOKEN.address, contracts.TOKEN.codeHash]
      initMsg.schedule = schedule
      schedule.pools.filter(x=>x.name==='MintingPool')[0]
              .accounts.filter(x=>x.name==='RPT')[0]
              .address = agent.address
      contracts.MGMT = await MGMTContract.init({agent, codeId, label, initMsg})
      report(contracts.MGMT.transactionHash) })
    await task('make mgmt owner of token', async report => {
      const {MGMT, TOKEN} = contracts, [tx1, tx2] = await MGMT.acquire(TOKEN)
      report(tx1.transactionHash)
      report(tx2.transactionHash) })
    await task('initialize rpt', async report => {
      const {codeId} = receipts.RPT, {label, initMsg} = inits.RPT, {MGMT, TOKEN} = contracts
      initMsg.token   = [TOKEN.address, TOKEN.codeHash]
      initMsg.mgmt    = [MGMT.address,  MGMT.codeHash ]
      initMsg.portion = "2500000000000000000000" // TODO get this from schedule!!!
      initMsg.config  = [[initialRPTRecipient, initMsg.portion]]
      contracts.RPT = await RPTContract.init({ agent, codeId, label, initMsg })
      report(contracts.RPT.transactionHash) })
    await task('point rpt account in mgmt schedule to rpt contract', async report => {
      const {MGMT, RPT} = contracts
      schedule.pools.filter(x=>x.name==='MintingPool')[0]
              .accounts.filter(x=>x.name==='RPT')[0]
              .address = RPT.address
      const {transactionHash} = await MGMT.configure(schedule)
      report(transactionHash) })
    console.log(table([
      ['Contract\nDescription',      'Address\nCode hash'],
      ['TOKEN\nSienna SNIP20 token', `${contracts.TOKEN.address}\n${contracts.TOKEN.codeHash}`],
      ['MGMT\nVesting',              `${contracts.MGMT.address}\n${contracts.MGMT.codeHash}`],
      ['RPT\nRemaining pool tokens', `${contracts.RPT.address}\n${contracts.RPT.codeHash}`]
    ]))
    return contracts
  }

  static async launch (options = {}) {
    let { network
        , address
        } = options
    if (typeof network === 'string') {
      network = conformChainIdToNetwork(network)
      network = (await SecretNetwork[network]({stateBase}))
    }
    const MGMT = network.network.getContract(MGMTContract, address, network.agent)
    console.info(`⏳ launching contract ${address}...`)
    try {
      await MGMT.launch()
      console.info(`🟢 launch reported success`)
    } catch (e) {
      console.warn(e)
      console.info(`🔴 launch reported a failure`)
    }
    console.info(`⏳ querying status...`)
    console.log(render(await MGMT.status))
  }

  static async reallocate () { throw new Error('not implemented') }

  static async addAccount () { throw new Error('not implemented') }

  static async claim (options = {}) {
    const { claimant = await pickKey()
          } = options
    let { network = 'localnet'
        } = options
    if (typeof network === 'string') {
      network = conformChainIdToNetwork(network)
      network = await SecretNetwork[network]({stateBase})
    }
    console.log({network, claimant})
  }

}

export class RewardsContracts extends ContractEnsemble {

  static workspace = abs()

  static build = (...args) => super.build(...args)

  static contracts = {
    TOKEN:
      { crate:   'snip20-reference-impl'
      , schema:  'schema'
      , label:   `${prefix}SIENNA_SNIP20`
      , initMsg:
        { prng_seed
        , name:     "Sienna"
        , symbol:   "SIENNA"
        , decimals: 18
        , config:   { public_total_supply: true } } },
    REWARDS:
      { crate: 'sienna-rewards'
      , label: `${prefix}SIENNA_REWARDS`
      , initMsg: JSON.parse(readFileSync(abs('settings/rewards.json'), 'utf8')) } }

  static async initialize ({ receipts, agent }) {
    const instances = {}
    const task = taskmaster()

    await task('initialize token', async report => {
      const {codeId} = receipts.TOKEN
      const {label, initMsg} = this.contracts.TOKEN
      Object.assign(initMsg, {
        admin: agent.address
      })
      instances.TOKEN = await SNIP20Contract.init({agent, codeId, label, initMsg})
      report(instances.TOKEN.transactionHash)
    })

    await task('initialize rewards', async report => {
      const {codeId} = receipts.REWARDS
      const {label, initMsg} = this.contracts.REWARDS
      console.log(agent.address)
      Object.assign(initMsg, {
        admin:     agent.address,
        entropy:   randomBytes(36).toString('base64'),
        prng_seed: randomBytes(36).toString('base64'),
        reward_token: {
          address:   instances.TOKEN.contractAddress,
          code_hash: instances.TOKEN.codeHash
        },
      })
      instances.REWARDS = await RewardsContract.init({agent, codeId, label, initMsg})
      report(instances.REWARDS.transactionHash)
    })

    await task('mint reward token', async report => {
      const result = await instances.TOKEN.mint(agent, '390000000000000000000000', instances.REWARDS.contractAddress)
      report(result)
    })

    console.log(instances)

    console.log(table([
      ['Contract\nDescription',      'Address\nCode hash'],
      ['TOKEN\nSienna SNIP20 token', `${instances.TOKEN.address}\n${instances.TOKEN.codeHash}`],
      ['Rewards\n',                  `${instances.REWARDS.address}\n${instances.REWARDS.codeHash}`],
    ]))

    return instances
  }

}
