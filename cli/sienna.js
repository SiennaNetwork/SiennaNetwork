#!/usr/bin/env node
// core
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync } from 'fs'
import { resolve, basename, extname, dirname } from 'path'
import { env, argv, stdout, stderr, exit } from 'process'
import { fileURLToPath } from 'url'

// 3rd party
import open from 'open'
import yargs from 'yargs'

// custom
import { SecretNetwork } from '@hackbg/fadroma'
import { scheduleFromSpreadsheet } from '@hackbg/schedule'

import { abs } from './root.js'
import { clear, cargo, run, runTests, runDemo } from './run.js'
import { genConfig, genCoverage, genSchema, genDocs } from './gen.js'
import { CONTRACTS, stateBase
       , deploy, build, upload, initialize, launch, transfer
       , configure, reallocate, addAccount
       , ensureWallets, claim } from './ops.js'

export default function main () {
  return yargs(process.argv.slice(2))
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

    // deployment and configuration:
    .command('deploy [network] [schedule]',
      '🚀 Build, init, and deploy (step by step with prompts)',
      combine(args.Network, args.Schedule), x => deploy(x).then(console.info))
    .command('upload <network>',
      '📦 Upload compiled contracts to network',
      args.Network, upload)
    .command('init <network> [<schedule>]',
      '🚀 Just instantiate uploaded contracts',
      combine(args.Network, args.Schedule), x => initialize(x).then(console.info))
    .command('launch <network> <address>',
      '🚀 Launch deployed vesting contract',
      combine(args.Network, args.Address), launch)
    .command('transfer <network> <address>',
      '⚡ Transfer ownership to another address',
      combine(args.Network, args.Address), transfer)
    .command('configure <deployment> <schedule>',
      '⚡ Upload a JSON config to an initialized contract',
      combine(args.Deployment, args.Schedule), configure)
    .command('reallocate <deployment> <allocations>',
      '⚡ Update the allocations of the RPT tokens',
      combine(args.Deployment, args.Allocations), reallocate)
    .command('add-account <deployment> <account>',
      '⚡ Add a new account to a partial vesting pool',
      combine(args.Deployment, args.Account), addAccount)

    // claiming:
    .command('claim <network> <contract> [<claimant>]',
      '⚡ Claim funds from a deployed contract',
      combine(args.Network, args.Contract, args.Claimant), claim)

    .argv
}

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
  , Address: yargs => yargs.positional(
      'address',
      { describe: 'secret network address' })
  , Contract: yargs => yargs.positional(
      'contract',
      { describe: 'secret network address' })
  , Claimant: yargs => yargs.positional(
      'claimant',
      { describe: 'secret network address' })
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

try {
  main()
} catch (e) {
  console.error(e)
  const ISSUES = `https://github.com/hackbg/sienna-secret-token/issues`
  console.info(`👹 That was a bug. Report it at ${ISSUES}`)
}
