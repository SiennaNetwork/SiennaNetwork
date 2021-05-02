import { stderr } from 'process'
import { readdirSync, readFileSync, existsSync } from 'fs'
import assert from 'assert'

import bignum from 'bignumber.js'
import prompts from 'prompts'
import { table } from 'table'
import { render } from 'prettyjson'

import { SNIP20Contract, MGMTContract, RPTContract } from '../api/index.js'

import { taskmaster, SecretNetwork } from '@hackbg/fadroma'
import { pull } from '@hackbg/fadroma/js/net.js'
import { fileURLToPath, resolve, basename, extname, dirname
       , readFile, writeFile } from '@hackbg/fadroma/js/sys.js'

import { conformChainIdToNetwork, pickNetwork, pickInstance } from './pick.js'
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

// contract list
const prefix = new Date().toISOString().replace(/[-:\.]/g, '-').replace(/[TZ]/g, '_')
const prng_seed = 'insecure'
export const CONTRACTS = {
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

export const getDefaultSchedule = () => {
  const path = resolve(projectRoot, 'settings', 'schedule.json')
  try {
    JSON.parse(readFileSync(path, 'utf8'))
  } catch (e) {
    console.warn(`${path} does not exist - "./sienna.js config" should create it`)
    return null
  }
}

// build and upload
export async function build (options = {}) {
  const { task      = taskmaster()
        , builder   = new SecretNetwork.Builder()
        , workspace = projectRoot
        , outputDir = resolve(workspace, 'artifacts') } = options

  // pull build container
  await pull('enigmampc/secret-contract-optimizer:latest')

  // build all contracts
  const binaries = {}
  for (const [name, {crate}] of Object.entries(CONTRACTS)) {
    await task(`build ${name}`, async report => {
      const buildOutput = resolve(outputDir, `${crate}@HEAD.wasm`)
      if (existsSync(buildOutput)) {
        console.info(`${buildOutput} exists. Delete it to rebuild that contract.`)
        binaries[name] = buildOutput
      } else {
        binaries[name] = await builder.build({outputDir, workspace, crate})
      }
    })
  }
  return binaries
}

export async function upload (options = {}) {
  const { task     = taskmaster()
        , binaries = await build() // if binaries are not passed, build 'em
        } = options

  let { builder
      , network = builder ? null : await SecretNetwork.localnet({stateBase}) } = options
  if (typeof network === 'string') network = await SecretNetwork[network]({stateBase})
  if (!builder) builder = network.builder

  const receipts = {}
  for (let contract of Object.keys(CONTRACTS)) {
    await task(`upload ${contract}`, async report => {
      const receipt = receipts[contract] = await builder.uploadCached(binaries[contract])
      console.log(`⚖️  compressed size ${receipt.compressedSize} bytes`)
      report(receipt.transactionHash) }) }

  return receipts
}

export async function initialize (options = {}) {

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
        , inits               = CONTRACTS
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

export async function deploy (options = {}) {
  const { task     = taskmaster()
        , initMsgs = {}
        , schedule = getDefaultSchedule()
        } = options

  let { agent
      , builder = agent ? agent.getBuilder() : undefined
      , network = builder ? builder.network : await pickNetwork()
      } = options

  if (typeof network === 'string') {
    assert(['localnet','testnet','mainnet'].indexOf(network) > -1)
    const conn = await SecretNetwork[network]()
    network = conn.network
    agent   = conn.agent
    builder = conn.builder
  }

  return await task('build, upload, and initialize contracts', async () => {
    const binaries  = await build({ task, builder })
    const receipts  = await upload({ task, builder, binaries })
    const contracts = await initialize({ task, receipts, agent, schedule })
  })
}

export async function transfer (options = {}) {
  throw new Error('not implemented')
  const { address
        , network
        , instance = await pickInstance(network) } = options
}

export async function configure ({
  file = abs('settings', 'schedule.json')
}) {
  throw new Error('not implemented')
  const { address
        , network
        , instance = await pickInstance(network) } = options
}

export async function launch (options = {}) {
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

export async function reallocate ({
  file = abs('settings', 'schedule.json')
}) {
  throw new Error('not implemented')
  const { address
        , network
        , instance = await pickInstance(network) } = options
}

export async function addAccount ({
  file = abs('settings', 'schedule.json')
}) {
  throw new Error('not implemented')
  const { address
        , network
        , instance = await pickInstance(network) } = options
}

export async function ensureWallets (options = {}) {

  let { recipientGasBudget = bignum("5000000")
      , connection         = 'testnet' } = options

  // allow passing strings:
  recipientGasBudget = bignum(recipientGasBudget)
  if (typeof connection === 'string') {
    assert(['localnet','testnet','mainnet'].indexOf(connection) > -1)
    connection = await SecretNetwork[connection]({stateBase})
  }

  const { task  = taskmaster()
        , n     = 16 // give or take
        // connection defaults to testnet because localnet
        // wallets are not worth keeping (they don't even
        // transfer between localnet instances)
        , agent      = connection.agent
        // {address:{agent,address}}
        , recipients = await getDefaultRecipients()
        // [[address,budget]]
        , wallets    = await recipientsToWallets(recipients)
        } = options

  // check that admin has enough balance to create the wallets
  const {balance, recipientBalances} = await fetchAdminAndRecipientBalances()
  const fee = bignum(agent.fees.send)
  const preseedTotal = fee.plus(bignum(wallets.length).times(recipientGasBudget))
  if (preseedTotal.gt(balance)) {
    const message =
      `admin wallet does not have enough balance to preseed test wallets ` +
     `(${balance.toString()} < ${preseedTotal.toString()}); can't proceed.\n\n` +
      `on localnet, it's easiest to clear the state and redo the genesis.\n` +
      `on testnet, use the faucet at https://faucet.secrettestnet.io/ twice\n` +
      `with ${agent.address} to get 200 testnet SCRT`
    console.error(message)
    process.exit(1) }
  await task(`ensure ${wallets.length} test accounts have balance`, async report => {
    const tx = await agent.sendMany(wallets, 'create recipient accounts')
    report(tx.transactionHash)})

  await fetchAdminAndRecipientBalances()

  async function getDefaultRecipients () {
    const recipients = {}
    const wallets = readdirSync(agent.network.wallets)
      .filter(x=>x.endsWith('.json'))
      .map(x=>readFileSync(resolve(agent.network.wallets, x), 'utf8'))
      .map(JSON.parse)
    for (const {address, mnemonic} of wallets) {
      const agent = await agent.network.getAgent({mnemonic})
      assert(address === agent.address)
      recipients[address] = { agent, address }
    }
    return recipients
  }
  async function recipientsToWallets (recipients) {
    return Promise.all(Object.values(recipients).map(({address, agent})=>{
      return agent.balance.then(balance=>[address, recipientGasBudget, bignum(balance) ])
    }))
  }
  async function fetchAdminAndRecipientBalances () {
    const balance = bignum(await agent.getBalance())
    console.info('Admin balance:', balance.toString())
    const withBalance = async ({agent}) => [agent.name, bignum(await agent.balance)]
    const recipientBalances = []
    console.info('\nRecipient balances:')
    for (const {agent} of Object.values(recipients)) {
      recipientBalances.push([agent.name, bignum(await agent.balance)])
      console.info(agent.name.padEnd(10), fmtSCRT(balance))
    }
    return {balance, recipientBalances}
  }
}
