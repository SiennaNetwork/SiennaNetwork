import { stderr } from 'process'
import { writeFileSync, readdirSync, readFileSync } from 'fs'
import assert from 'assert'

import bignum from 'bignum'

import { scheduleFromSpreadsheet } from '@hackbg/schedule'
import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

import { taskmaster, SecretNetwork } from '@hackbg/fadroma'
import { pull } from '@hackbg/fadroma/js/net.js'
import { fileURLToPath, resolve, basename, extname, dirname
       , readFile, writeFile } from '@hackbg/fadroma/js/sys.js'

// resolve path relative to this file's parent directory
export const __dirname = dirname(fileURLToPath(import.meta.url))
export const abs = (...args) => resolve(__dirname, ...args)
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

export async function build (options = {}) {
  const { task      = taskmaster()
        , builder   = new SecretNetwork.Builder()
        , workspace = __dirname
        , outputDir = resolve(workspace, 'artifacts') } = options

  // pull build container
  await pull('enigmampc/secret-contract-optimizer:latest')

  // build all contracts
  const binaries = {}
  await task.parallel('build project',
    ...Object.entries(CONTRACTS).map(([name, {crate}])=>
      task(`build ${name}`, async report => {
        binaries[name] = await builder.build({outputDir, workspace, crate})
      })))

  return binaries
}

export async function upload (options = {}) {
  const { task     = taskmaster()
        , binaries = await build() // if binaries are not passed, build 'em
        } = options

  let { builder
      , conn = builder ? null : await SecretNetwork.localnet({stateBase}) } = options
  if (typeof conn === 'string') conn = await SecretNetwork[conn]({stateBase})
  if (!builder) builder = conn.builder

  const receipts = {}
  for (let contract of Object.keys(CONTRACTS)) {
    await task(`upload ${contract}`, async report => {
      const receipt = receipts[contract] = await builder.uploadCached(binaries[contract])
      console.log(`⚖️  compressed size ${receipt.compressedSize} bytes`)
      report(receipt.transactionHash) }) }

  return receipts
}

export function prepareConfig (options = {}) {
  const { file = abs('settings', 'schedule.ods')
        } = options

  file = resolve(file) // ???

  stderr.write(`\n⏳ Importing configuration from ${file}...\n\n`)
  const name       = basename(file, extname(file)) // path without extension
  const schedule   = scheduleFromSpreadsheet({ file })
  const serialized = stringify(schedule)
  const output     = resolve(dirname(file), `${name}.json`)
  stderr.write(`⏳ Saving configuration to ${output}...\n\n`)

  writeFileSync(output, stringify(schedule), 'utf8')
  stderr.write(`🟢 Configuration saved to ${output}\n`)
}

export async function initialize (options = {}) {

  // idempotency support
  // passing existing `contracts` to this makes it a no-op
  const { contracts = {} } = options
  if (Object.keys(contracts)>0) return contracts

  // unwrap mutable options
  let { agent
      , conn = agent ? {network: agent.network}
                     : await SecretNetwork.localnet({stateBase})
      , schedule
      } = options

  // accepts schedule as string or struct
  if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))

  // if `conn` is just the connection type, replace it with a real connection
  if (typeof conn === 'string') conn = await SecretNetwork[conn]({stateBase})

  // if there's no agent, use the default one from the connection
  if (!agent) agent = conn.agent

  // unwrap remaining options
  const { task                = taskmaster()
        , receipts            = await upload({agent, conn, task})
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
  return contracts
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
  const preseedTotal = fee.add(bignum(wallets.length).mul(recipientGasBudget))
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
      console.info(name.padEnd(10), fmtSCRT(balance))
    }
    return {balance, recipientBalances}
  }
}

export async function configure ({
  file = abs('settings', 'schedule.json')
}) {
  throw new Error('not implemented')
}

export default async function deploy ({
  task     = taskmaster(),
  builder  = new SecretNetwork.Builder(),
  initMsgs
}) {
  builder = await Promise.resolve(builder)
  return await task('build, upload, and initialize contracts', async () => {
    const binaries  = await build({ task, builder })
    const receipts  = await upload({ task, builder, binaries })
    const contracts = await initialize({ task, builder, initMsgs })
  })
}

export async function launch () {
  throw new Error('not implemented')
}

export async function reallocate ({
  file = abs('settings', 'schedule.json')
}) {
  throw new Error('not implemented')
}

export async function addAccount ({
  file = abs('settings', 'schedule.json')
}) {
  throw new Error('not implemented')
}

export function generateCoverage () {
  // fixed by https://github.com/rust-lang/cargo/issues/9220
  let output = abs('docs', 'coverage')
  cargo('tarpaulin', '--out=Html', `--output-dir=${output}`)
}

export function generateSchema () {
  const cwd = process.cwd()
  try {
    for (const [name, {schema}] of Object.entries(CONTRACTS)) {
      const contractDir = abs('contracts', name)
      stderr.write(`Generating schema in ${contractDir}...`)
      process.chdir(contractDir)
      cargo('run', '--example', schema)
    }
  } finally {
    process.chdir(cwd)
  }
}

export function generateDocs () {
  const target = abs('target', 'doc', crate, 'index.html')
  try {
    stderr.write(`⏳ Building documentation...\n\n`)
    cargo('doc')
  } catch (e) {
    stderr.write('\n🤔 Building documentation failed.')
    if (existsSync(target)) {
      stderr.write(`\n⏳ Opening what exists at ${target}...`)
    } else {
      return
    }
  }
  open(`file:///${target}`)
}

const stringify = data => {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent)
}
