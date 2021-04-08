import { stderr } from 'process'
import { writeFileSync } from 'fs'

import { scheduleFromSpreadsheet } from '@hackbg/schedule'
import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

import { taskmaster, SecretNetwork } from '@hackbg/fadroma'
import { pull } from '@hackbg/fadroma/js/net.js'
import { fileURLToPath, resolve, basename, extname, dirname
       , readFile } from '@hackbg/fadroma/js/sys.js'

// resolve path relative to this file's parent directory
export const abs = (...args) => resolve(dirname(fileURLToPath(import.meta.url)), ...args)
export const stateBase = abs('.fadroma')

const timestamp = new Date().toISOString()

// contract list
export const CONTRACTS = {

  TOKEN: {
    crate:     'snip20-reference-impl',
    schemaGen: 'schema',
    label:     `[${timestamp}] snip20`,
    initMsg:   { name:      "Sienna"
               , symbol:    "SIENNA"
               , decimals:  18
               , prng_seed: "insecure"
               , config:    { public_total_supply: true } } },

  MGMT: {
    crate:     'sienna-mgmt',
    schemaGen: 'mgmt_schema',
    label:     `[${timestamp}] mgmt`,
    initMsg:   {} },

  RPT: {
    crate:     'sienna-rpt',
    schemaGen: 'rpt_schema',
    label:     `[${timestamp}] rpt`,
    initMsg:   {} }
}

export async function build ({
  task      = taskmaster(),
  workspace = abs('.'),
  outputDir = resolve(workspace, 'artifacts'),
  builder   = new SecretNetwork.Builder(),
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
  let {
    task     = taskmaster(),
    network  = await SecretNetwork.localnet({stateBase}),
    binaries = await build()
  } = options

  if (typeof network === 'string') network = await SecretNetwork[network]({stateBase})
  const {builder} = network

  const receipts = {}
  for (let contract of Object.keys(CONTRACTS)) {
    await task(`upload ${contract}`, async () => {
      const receipt = receipts[contract] = await builder.uploadCached(binaries[contract])
      console.log(`⚖️  compressed size ${receipt.compressedSize} bytes`)
    })
  }
  return receipts
}

export async function initialize (options = {}) {

  let { network = await SecretNetwork.localnet({stateBase}) } = options
  if (typeof network === 'string') network = await SecretNetwork[network]({stateBase})
  const {agent} = network

  let { schedule } = options
  if (typeof schedule === 'string') schedule = JSON.parse(await readFile(schedule, 'utf8'))

  const {
    task     = taskmaster(),
    receipts = await upload({network}),
    inits    = CONTRACTS,
  } = options

  const contracts = {}
  const initTXs = {}
  await task('initialize token', async report => {
    const {codeId}      = receipts.TOKEN
        , {label, initMsg} = inits.TOKEN

    initMsg.admin = agent.address

    contracts.TOKEN = new SNIP20Contract({agent, codeId})
    report(await contracts.TOKEN.init({label, initMsg}))
  })

  await task('initialize mgmt', async report => {
    const {codeId}      = receipts.MGMT
        , {label, initMsg} = inits.MGMT

    initMsg.token    = [contracts.TOKEN.address, contracts.TOKEN.codeHash]
    initMsg.schedule = schedule

    contracts.MGMT = new MGMTContract({agent, codeId})
    report(await contracts.MGMT.init({label, initMsg}))
  })

  await task('initialize rpt', async report => {
    const {codeId}      = receipts.RPT
        , {label, initMsg} = inits.RPT

    initMsg.token = [contracts.TOKEN.address, contracts.TOKEN.codeHash]
    initMsg.mgmt  = [contracts.MGMT.address, contracts.MGMT.codeHash]

    contracts.RPT = new RPTContract({ agent, codeId })
    report(await contracts.RPT.init({label, initMsg}))
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

const stringify = data => {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent)
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
    for (const [name, {schemaGen}] of Object.entries(CONTRACTS)) {
      const contractDir = abs('contracts', name)
      stderr.write(`Generating schema in ${contractDir}...`)
      process.chdir(contractDir)
      cargo('run', '--example', schemaGen)
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

export async function makeTestnetWallets (options = {}) {
  const { n       = 20
        , network = await SecretNetwork.testnet({stateBase})
        , agent   = network.agent } = options
  const wallets = await Promise.all([...Array(n)].map(()=>network.network.getAgent()))
  for (const {address, mnemonic} of wallets) {
    await agent.send(address, 5000000)
    console.info()
    console.info(address)
    console.info(mnemonic)
  }
}

export async function launch () {}
