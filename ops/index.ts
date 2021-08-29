#!/usr/bin/env node
import { Chain, Scrt, prefund,
         CommandName, Commands, runCommand, printUsage, REPL, open, prompts,
         on, resetLocalnet, openFaucet } from '@hackbg/fadroma'
import { env, stderr, existsSync, readFileSync, writeFileSync,
         resolve, basename, extname, dirname, fileURLToPath, cargo } from '@hackbg/fadroma'
import { SNIP20Contract,
         MGMTContract, RPTContract,
         RewardsContract,
         AMMContract, FactoryContract,
         IDOContract } from '@sienna/api'
import { scheduleFromSpreadsheet } from '@sienna/schedule'
import { SiennaTGE, SiennaSwap, SiennaRewards, SiennaLend } from './ensembles'
import { CLIHelp as Help } from './help'

export const projectRoot = resolve(dirname(fileURLToPath(import.meta.url)), '..')
           , abs         = (...args: Array<string>) => resolve(projectRoot, ...args)
           , stateBase   = abs('artifacts')

export default async function main (command: CommandName, ...args: any) {

  const tge     = new SiennaTGE()
      , rewards = new SiennaRewards()
      , amm     = new SiennaSwap()
      , lend    = new SiennaLend()

  function remoteCommands (chain: Chain): Commands {
    return [
      ["status",  Help.STATUS,  () => chain.printStatusTables()],
      null,
      ["tge",     Help.TGE,     null, new SiennaTGE({chain}).remoteCommands()],
      ["amm",     Help.AMM,     null, new SiennaSwap({chain}).remoteCommands()],
      ["rewards", Help.REWARDS, null, new SiennaRewards({chain}).remoteCommands()],
      ["lend",    Help.LEND,    null, new SiennaLend({chain}).remoteCommands()]] }

  const commands: Commands = [
    [["help", "--help", "-h"], Help.USAGE, () => printUsage({}, commands)],

    null,

    ["docs",     Help.DOCS,     genDocs],
    ["test",     Help.TEST,     runTests],
    ["coverage", Help.COVERAGE, genCoverage],
    ["schema",   Help.SCHEMA,   genSchema],
    ["build",    Help.BUILD, null, [
      ["all",     Help.BUILD_ALL,     () => cargo('build')],
      ["tge",     Help.BUILD_TGE,     () => tge.build()],
      ["rewards", Help.BUILD_REWARDS, () => rewards.build()],
      ["amm",     Help.BUILD_AMM,     () => amm.build()],
      ["lend",    Help.BUILD_LEND,    () => lend.build()]]],

    null,

    ["tge",     Help.TGE,     null,
      [...tge.localCommands(),     null, ...Scrt.chainSelector(SiennaTGE)    ] as Commands],
    ["amm",     Help.AMM,     null,
      [...amm.localCommands(),     null, ...Scrt.chainSelector(SiennaSwap)   ] as Commands],
    ["rewards", Help.REWARDS, null,
      [...rewards.localCommands(), null, ...Scrt.chainSelector(SiennaRewards)] as Commands],
    ["lend",    Help.LEND,    null,
      [...lend.localCommands(),    null, ...Scrt.chainSelector(SiennaLend)   ] as Commands],

    null,

    ["mainnet",  Help.MAINNET,  on.mainnet,  [
      ["shell",  Help.SHELL,    runShell],
      ...remoteCommands(Scrt.mainnet())]],
    ["testnet",  Help.TESTNET,  on.testnet,  [
      ["shell",  Help.SHELL,    runShell],
      ["faucet", Help.FAUCET,   openFaucet],
      ["fund",   Help.FUND,     prefund],
      ...remoteCommands(Scrt.testnet())]],
    ["localnet", Help.LOCALNET, on.localnet, [
      ["shell",  Help.SHELL,    runShell],
      ["reset",  Help.FAUCET,   resetLocalnet],
      ["fund",   Help.FUND,     prefund],
      ...remoteCommands(Scrt.localnet())]]]

  return await runCommand({ command: [ command ] }, commands, command, ...args) }

export async function runShell ({
  chain, agent, builder,
}: Record<string, any>) {
  return await new REPL({
    workspace: abs(),
    chain,
    agent,
    builder,
    Contracts: {
      AMM:     AMMContract,
      Factory: FactoryContract,
      IDO:     IDOContract,
      MGMT:    MGMTContract,
      RPT:     RPTContract,
      Rewards: RewardsContract,
      SNIP20:  SNIP20Contract },
    Ensembles: {
      TGE:     SiennaTGE,
      Rewards: SiennaRewards,
      Swap:    SiennaSwap,
      Lend:    SiennaLend } }).run() }

export function genCoverage () {
  // fixed by https://github.com/rust-lang/cargo/issues/9220
  cargo('tarpaulin', '--out=Html', `--output-dir=${abs()}`, '--locked', '--frozen') }

export function genSchema () {
  cargo('run', '--bin', 'schema') }

export function genDocs (_:any, crate = '', dontOpen = false) {
  const entryPoint = crate
    ? abs('target', 'doc', crate, 'index.html')
    : abs('target', 'doc')
  try {
    process.stderr.write(`⏳ Building documentation...\n\n`)
    cargo('doc') }
  catch (e) {
    process.stderr.write('\n🤔 Building documentation failed.')
    if (!dontOpen) {
      if (existsSync(entryPoint)) {
        process.stderr.write(`\n⏳ Opening what exists at ${entryPoint}...`) }
      else {
        process.stderr.write(`\n⏳ ${entryPoint} does not exist, opening nothing.`)
        return } } }
  if (!dontOpen) {
    open(`file://${entryPoint}`) } }

export function getDefaultSchedule () {
  const path = resolve(projectRoot, 'settings', 'schedule.json')
  try {
    return JSON.parse(readFileSync(path, 'utf8')) }
  catch (e) {
    console.warn(`${path} does not exist - "./sienna tge config" should create it`)
    return null } }

export function genConfig (
  { file = abs('settings', 'schedule.ods') } = {}
) {
  stderr.write(`\n⏳ Importing configuration from ${file}...\n\n`)
  const name     = basename(file, extname(file)) // path without extension
      , schedule = scheduleFromSpreadsheet({ file })
      , output   = resolve(dirname(file), `${name}.json`)
  stderr.write(`⏳ Saving configuration to ${output}...\n\n`)
  writeFileSync(output, stringify(schedule), 'utf8')
  stderr.write(`🟢 Configuration saved to ${output}\n`) }

function stringify (data: any) {
  const indent = 2
  const withBigInts = (_:any, v:any) => typeof v === 'bigint' ? v.toString() : v
  return JSON.stringify(data, withBigInts, indent) }

export const runTests = () => {
  clear()
  stderr.write(`⏳ Running tests...\n\n`)
  try {
    run('sh', '-c',
      'cargo test --color=always --no-fail-fast -- --nocapture --test-threads=1 2>&1'+
      ' | less -R +F')
    stderr.write('\n🟢 Tests ran successfully.\n') }
  catch (e) {
    stderr.write('\n🔴 Tests failed.\n') } }

export const fmtDecimals = (d: number|string) => (x: number|string) => {
  const a = (BigInt(x) / BigInt(d)).toString()
  const b = (BigInt(x) % BigInt(d)).toString()
  return `${a}.${b.padEnd(18, '0')}` }

export const
  SCRT_DECIMALS = 6,
  ONE_SCRT = BigInt(`1${[...Array(SCRT_DECIMALS)].map(()=>`0`).join('')}`),
  fmtSCRT  = fmtDecimals(ONE_SCRT.toString())

export const
  SIENNA_DECIMALS = 18,
  ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>`0`).join('')}`),
  fmtSIENNA  = fmtDecimals(ONE_SIENNA.toString())
