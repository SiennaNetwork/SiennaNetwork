import { stderr } from 'process'
import { writeFileSync } from 'fs'

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

export async function build ({
  task      = taskmaster(),
  builder   = new SecretNetwork.Builder(),
  workspace = __dirname,
  outputDir = resolve(workspace, 'artifacts'),
} = {}) {
  await pull('enigmampc/secret-contract-optimizer:latest')
  const binaries = {}
  await task.parallel('build project',
    ...Object.entries(CONTRACTS).map(([name, {crate}])=>
      task(`build ${name}`, async report => {
        binaries[name] = await builder.build({outputDir, workspace, crate})
      })))
  return binaries
}

export async function upload (options = {}) {
  const {
    task     = taskmaster(),
    binaries = await build()
  } = options

  let { builder
      , conn = builder ? null : await SecretNetwork.localnet({stateBase}) } = options
  if (typeof conn === 'string') conn = await SecretNetwork[conn]({stateBase})
  if (!builder) builder = conn.builder

  const receipts = {}
  for (let contract of Object.keys(CONTRACTS)) {
    await task(`upload ${contract}`, async () => {
      const receipt = receipts[contract] = await builder.uploadCached(binaries[contract])
      console.log(`⚖️  compressed size ${receipt.compressedSize} bytes`)
    })
  }
  return receipts
}

export function prepareConfig ({
  file = abs('settings', 'schedule.ods')
}) {
  file = resolve(file)
  stderr.write(`\n⏳ Importing configuration from ${file}...\n\n`)

  const name       = basename(file, extname(file)) // path without extension
  const schedule   = scheduleFromSpreadsheet({ file })
  const serialized = stringify(schedule)
  //stderr.write(render(JSON.parse(serialized))) // or `BigInt`s don't show

  const output     = resolve(dirname(file), `${name}.json`)
  stderr.write(`⏳ Saving configuration to ${output}...\n\n`)

  writeFileSync(output, stringify(schedule), 'utf8')
  stderr.write(`🟢 Configuration saved to ${output}\n`)
}

export async function initialize (options = {}) {

  let { agent
      , conn = agent ? {network: agent.network}
                     : await SecretNetwork.localnet({stateBase}) } = options
  if (typeof conn === 'string') conn = await SecretNetwork[conn]({stateBase})
  if (!agent) agent = conn.agent

  const { task = taskmaster()
        , receipts = await upload({agent, conn, task})
        , inits = CONTRACTS
        , initialRPTRecipient = agent.address } = options

  let { schedule } = options
  if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))

  const contracts = {}

  await task('initialize token', async report => {
    const {codeId} = receipts.TOKEN, {label, initMsg} = inits.TOKEN
    initMsg.admin = agent.address
    contracts.TOKEN = await SNIP20Contract.init({agent, codeId, label, initMsg})
    report(contracts.TOKEN.transactionHash)
  })

  await task('initialize mgmt', async report => {
    const {codeId} = receipts.MGMT, {label, initMsg} = inits.MGMT
    initMsg.token    = [contracts.TOKEN.address, contracts.TOKEN.codeHash]
    initMsg.schedule = schedule
    contracts.MGMT = await MGMTContract.init({agent, codeId, label, initMsg})
    report(contracts.MGMT.transactionHash)
  })

  await task('make mgmt owner of token', async report => {
    const {MGMT, TOKEN} = contracts, [tx1, tx2] = await MGMT.acquire(TOKEN)
    report(tx1.transactionHash)
    report(tx2.transactionHash)
  })

  await task('initialize rpt', async report => {
    const {codeId} = receipts.RPT, {label, initMsg} = inits.RPT, {MGMT, TOKEN} = contracts
    initMsg.token   = [TOKEN.address, TOKEN.codeHash]
    initMsg.mgmt    = [MGMT.address,  MGMT.codeHash ]
    initMsg.portion = "2500000000000000000000"
    initMsg.config  = [[initialRPTRecipient, initMsg.portion]]
    contracts.RPT = await RPTContract.init({ agent, codeId, label, initMsg })
    report(contracts.RPT.transactionHash)
  })

  await task('point rpt account in mgmt schedule to rpt contract', async report => {
    const {MGMT, RPT} = contracts
    schedule.pools.filter(x=>x.name==='MintingPool')[0]
            .accounts.filter(x=>x.name==='RPT')[0]
            .address = RPT.address
    const {transactionHash} = await MGMT.configure(schedule)
    report(transactionHash)
  })

  return contracts
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

export async function setConfig ({
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

export async function makeWallets (options = {}) {
  const { n     = 20
        , conn  = await SecretNetwork.testnet({stateBase})
        , agent = conn.agent } = options
  console.info(`make ${n} wallets...`)
  const agents = await Promise.all([...Array(n)].map(()=>conn.conn.getAgent()))
  for (const {address, mnemonic} of agents) {
    await agent.send(address, 5000000)
    console.info()
    console.info(address)
    console.info(mnemonic)
    const file = resolve(conn.network.wallets, `${address}.json`)
    await writeFile(file, JSON.stringify({address, mnemonic}), 'utf8')
  }
}

export async function launch () {}

const stringify = data => {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent)
}
