#!/usr/bin/env node
// core
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from 'fs'
import { resolve, basename, extname, dirname } from 'path'
import { env, argv, stdout, stderr, exit } from 'process'
import { execFileSync } from 'child_process'
import { fileURLToPath } from 'url'

// 3rd party
import open from 'open'
import yargs from 'yargs'

// custom
import { SecretNetwork } from '@hackbg/fadroma'
import { scheduleFromSpreadsheet } from '@hackbg/schedule'
import { CONTRACTS, abs, stateBase
       , deploy, build, upload, initialize, launch
       , genConfig, configure, reallocate, addAccount
       , ensureWallets } from './ops.js'
import { genCoverage, genSchema, genDocs } from './artifacts/index.js'
import demo from './demo.js'

const main = () => yargs(process.argv.slice(2))
  .wrap(yargs().terminalWidth())
  .demandCommand(1, '')

  // validation:
  .command('test',
    '⚗️  Run test suites for all the individual components.',
    runTests)
  .command('ensure-wallets',
    '⚗️  Ensure there are testnet wallets for the demo.',
    ensureWallets)
  .command('demo [--testnet]',
    '⚗️  Run integration test/demo.',
    args.IsTestnet, runDemo)

  // artifacts:
  .command('build',
    '👷 Compile contracts from working tree',
    build)
  .command('schema',
    `🤙 Regenerate JSON schema for each contract's API.`,
    genSchema)
  .command('docs [crate]',
    '📖 Build the documentation and open it in a browser.',
    args.Crate, genDocs)
  .command('coverage',
    '⚗️  Generate test coverage and open it in a browser.',
    genCoverage)
  .command('config [<spreadsheet>]',
    '📅 Convert a spreadsheet into a JSON schedule',
    args.Spreadsheet, genConfig)

  // prepare contract binaries:
  .command('deploy',
    '🚀 Build, init, and deploy (step by step with prompts)',
    combine(args.Network, args.Schedule), x => deploy(x).then(console.info))
  .command('upload <network>',
    '📦 Upload compiled contracts to network',
    args.Network, upload)
  .command('init <network> [<network>]',
    '🚀 Just instantiate uploaded contracts',
    combine(args.Network, args.Schedule), x => initialize(x).then(console.info))
  .command('launch <deployment>',
    '🚀 Just launch initialized contracts',
    launch)

  // post-launch config
  .command('configure <deployment> <schedule>',
    '⚡ Upload a JSON config to an initialized contract',
    combine(args.Deployment, args.Schedule), configure)
  .command('reallocate <deployment> <allocations>',
    '⚡ Update the allocations of the RPT tokens',
    combine(args.Deployment, args.Allocations), reallocate)
  .command('add-account <deployment> <account>',
    '⚡ Add a new account to a partial vesting pool',
    combine(args.Deployment, args.Account), addAccount)

  .argv

const combine = (...args) =>
  yargs => args.reduce((yargs, argfn)=>argfn(yargs), yargs)
const args =
  { IsTestnet:   yargs => yargs.option(
      'testnet',
      { describe: 'run on holodeck-2 instead of a local container' })
  , Network:     yargs => yargs.positional(
      'network',
      { describe: 'the network to connect to'
      , default:  'localnet'
      , choices:  ['localnet', 'testnet', 'mainnet'] })
  , Spreadsheet: yargs => yargs.positional(
      'spreadsheet',
      { describe: 'path to input spreadsheet'
      , default:  abs('settings', 'schedule.ods') })
  , Schedule:    yargs => yargs.positional(
      'schedule',
      { describe: 'the schedule to use'
      , default:  abs('settings', 'schedule.json') })
  , Crate:       yargs => yargs.positional(
      'crate',
      { describe: 'crate to open'
      , default:  'sienna_schedule' })
  , Account:     yargs => yargs.positional(
      'account',
      { describe: 'description of account to add' })
  , Allocations: yargs => yargs.positional(
      'allocations',
      { describe: 'new allocation of Remaining Pool Tokens' }) }

export const clear = () =>
  env.TMUX && run('sh', '-c', 'clear && tmux clear-history')

export const cargo = (...args) =>
  run('cargo', '--color=always', ...args)

export const run = (cmd, ...args) => {
  stderr.write(`\n🏃 running:\n${cmd} ${args.join(' ')}\n\n`)
  execFileSync(cmd, [...args], {stdio:'inherit'})
}

const runTests = () => {
  clear()
  stderr.write(`⏳ Running tests...\n\n`)
  try {
    run('sh', '-c',
      'cargo test --color=always --no-fail-fast -- --nocapture --test-threads=1 2>&1'+
      ' | less -R')
    stderr.write('\n🟢 Tests ran successfully.\n')
  } catch (e) {
    stderr.write('\n👹 Tests failed.\n')
  }
}

const runDemo = async ({testnet}) => {
  clear()
  //script = abs('integration', script)
  try {
    let environment
    if (testnet) {
      console.info(`⏳ running demo on testnet...`)
      environment = await SecretNetwork.testnet({stateBase})
    } else {
      console.info(`⏳ running demo on localnet...`)
      environment = await SecretNetwork.localnet({stateBase})
    }
    await demo(environment)
    console.info('\n🟢 Demo executed successfully.\n')
  } catch (e) {
    console.error(e)
    console.info('\n👹 Demo failed.\n')
  }
}

main()
