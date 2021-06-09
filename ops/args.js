import { abs } from './root.js'

export const combine = (...args) =>
  yargs => args.reduce((yargs, argfn)=>argfn(yargs), yargs)

export const args =
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

