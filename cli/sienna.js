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
import { SecretNetwork } from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'
import { scheduleFromSpreadsheet } from '@sienna/schedule'

import { abs } from './root.js'
import { clear, cargo, run, runTests, runDemo } from './run.js'
import { genConfig, genCoverage, genSchema, genDocs } from './gen.js'
import { stateBase, TGEContracts, RewardsContracts } from './ops.js'

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
      args.Sequential, TGEContracts.build)
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
    .command('clean-localnet',
      '♻️  Try to terminate a loose localnet container and remove its state files',
      () => new SecretNetwork.Node().terminate())

    // deployment and configuration:
    .command('deploy-tge [network] [schedule]',
      '🚀 Build, init, and deploy the TGE',
      combine(args.Network, args.Schedule),
      x => TGEContracts.deploy(x).then(console.info))
    .command('upload <network>',
      '📦 Upload compiled contracts to network',
      args.Network,
      TGEContracts.upload)
    .command('init <network> [<schedule>]',
      '🚀 Just instantiate uploaded contracts',
      combine(args.Network, args.Schedule),
      x => TGEContracts.initialize(x).then(console.info))
    .command('launch <network> <address>',
      '🚀 Launch deployed vesting contract',
      combine(args.Network, args.Address),
      TGEContracts.launch)
    .command('transfer <network> <address>',
      '⚡ Transfer ownership to another address',
      combine(args.Network, args.Address),
      TGEContracts.transfer)
    .command('configure <deployment> <schedule>',
      '⚡ Upload a JSON config to an initialized contract',
      combine(args.Deployment, args.Schedule),
      TGEContracts.configure)
    .command('reallocate <deployment> <allocations>',
      '⚡ Update the allocations of the RPT tokens',
      combine(args.Deployment, args.Allocations),
      TGEContracts.reallocate)
    .command('add-account <deployment> <account>',
      '⚡ Add a new account to a partial vesting pool',
      combine(args.Deployment, args.Account),
      TGEContracts.addAccount)

    .command('deploy-rewards [network]',
      '🚀 Build, init, and deploy the rewards component',
      combine(args.Network, args.Schedule),
      x => RewardsContracts.deploy(x).then(console.info))

    // claiming:
    .command('claim <network> <contract> [<claimant>]',
      '⚡ Claim funds from a deployed contract',
      combine(args.Network, args.Contract, args.Claimant), TGEContracts.claim)

    .argv
}

const combine = (...args) =>
  yargs => args.reduce((yargs, argfn)=>argfn(yargs), yargs)
const args =
  { IsTestnet:   yargs => yargs.option(
      'testnet',
      { describe: 'run on holodeck-2 instead of a local container' })
  , Sequential:  yargs => yargs.option(
      'sequential',
      { describe: 'build contracts one at a time instead of simultaneously' })
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
