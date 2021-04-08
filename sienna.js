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
       , build, upload, initialize, launch
       , prepareConfig, setConfig
       , generateCoverage, generateSchema, generateDocs
       , makeTestnetWallets } from './ops.js'
import demo from './demo.js'

const main = () => yargs(process.argv.slice(2))
  .wrap(yargs().terminalWidth())
  .demandCommand(1, '')

  // main deploy flow:

  .command('build',
    '👷 Compile contracts from working tree',
    build)

  .command('upload <network>',
    '📦 Upload compiled contracts to network',
    withNetwork,
    upload)

  .command('prepare-config [<spreadsheet>]',
    '📅 Convert a spreadsheet into a JSON schedule',
    withSpreadsheet,
    prepareConfig)

  .command('init <network> [<schedule>]',
    '💡 Instantiate uploaded contracts',
    yargs => withSchedule(withNetwork(yargs)),
    initialize)

  .command('launch <initReceiptOrContractAddr>',
    '📦 Launch initialized contracts',
    launch)

  // configuration:

  .command('set-config <initReceiptOrContractAddr> <schedule>',
    '⚡ Upload a JSON config to an initialized contract',
    yargs => yargs.positional('file', {
      describe: 'path to input JSON',
      default: abs('settings', 'schedule.json') }),
    setConfig)

  // appendices:

  .command('coverage',
    '🗺️  Generate test coverage and open it in a browser.',
    generateCoverage)

  .command('schema',
    `🤙 Regenerate JSON schema for each contract's API.`,
    generateSchema)

  .command('docs [crate]',
    '📖 Build the documentation and open it in a browser.',
    yargs => yargs.positional('crate', {
      describe: 'crate to open',
      default: 'sienna_schedule' }),
    generateDocs)

  .command('test',
    '⚗️  Run test suites for all the individual components.',
    function runTests () {
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
    })

  .command('demo [--testnet]',
    '📜 Run integration test/demo.',
    yargs =>
      yargs.option('testnet', { describe: 'run on holodeck-2 instead of a local container' }),
    async function runDemo ({testnet}) {
      clear()
      //script = abs('integration', script)
      try {
        let environment
        if (testnet) {
          stderr.write(`⏳ Running demo on testnet...\n\n`)
          environment = await SecretNetwork.testnet({stateBase})
        } else {
          stderr.write(`⏳ Running demo on localnet...\n\n`)
          environment = await SecretNetwork.localnet({stateBase})
        }
        await demo(environment)
        stderr.write('\n🟢 Demo executed successfully.\n')
      } catch (e) {
        console.error(e)
        stderr.write('\n👹 Demo failed.\n')
      }
    })

  .command('make-testnet-wallets',
    'Create and preseed 20 empty testnet wallets',
    makeTestnetWallets)

  .argv

const withNetwork = yargs =>
  yargs.positional('network',
    { describe: 'the network to connect to'
    , default:  'localnet'
    , choices:  ['localnet', 'testnet', 'mainnet'] })

const withSpreadsheet = yargs =>
  yargs.positional('spreadsheet',
    { describe: 'path to input spreadsheet'
    , default:  abs('settings', 'schedule.ods') })

const withSchedule = yargs =>
  yargs.positional('schedule',
    { describe: 'the schedule to use'
    , default:  abs('settings', 'schedule.json') })

main()

function cargo (...args) {
  run('cargo', '--color=always', ...args)
}

function clear () {
  if (env.TMUX) {
    run('sh', '-c', 'clear && tmux clear-history')
  }
}

function run (cmd, ...args) {
  stderr.write(`\n🏃 running:\n${cmd} ${args.join(' ')}\n\n`)
  execFileSync(cmd, [...args], {stdio:'inherit'})
}
