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
export const abs = (...args) => resolve(dirname(fileURLToPath(import.meta.url)), ...args)
export const stateBase = abs('artifacts')

const timestamp = new Date().toISOString()
  .replace(/[-:\.]/g, '-')
  .replace(/[TZ]/g, '_')
const prng_seed = "insecure"

// contract list
export const CONTRACTS = {
  TOKEN: {
    crate:   'snip20-reference-impl',
    schema:  'schema',
    label:   `${timestamp}snip20`,
    initMsg: { prng_seed, name: "Sienna", symbol: "SIENNA", decimals:  18
             , config: { public_total_supply: true } } },
  MGMT: {
    crate:   'sienna-mgmt',
    schema:  'mgmt_schema',
    label:   `${timestamp}mgmt`,
    initMsg: {} },
  RPT: {
    crate:   'sienna-rpt',
    schema:  'rpt_schema',
    label:   `${timestamp}rpt`,
    initMsg: {} }
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
    initMsg.config  = [[agent.address, initMsg.portion]]
    contracts.RPT = await RPTContract.init({ agent, codeId, label, initMsg })
    report(contracts.RPT)
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
  const { n       = 20
        , network = await SecretNetwork.testnet({stateBase})
        , agent   = network.agent } = options
  console.info(`make ${n} wallets...`)
  const agents = await Promise.all([...Array(n)].map(()=>network.network.getAgent()))
  for (const {address, mnemonic} of agents) {
    await agent.send(address, 5000000)
    console.info()
    console.info(address)
    console.info(mnemonic)
    const file = resolve(network.network.wallets, `${address}.json`)
    await writeFile(file, JSON.stringify({address, mnemonic}), 'utf8')
  }
}

export async function launch () {}

const stringify = data => {
  const indent = 2
  const withBigInts = (k, v) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent)
}
