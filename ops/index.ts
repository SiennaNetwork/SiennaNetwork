#!/usr/bin/env node
import { argv } from 'process'
import ensureWallets from '@fadroma/agent/scrt_fund.js'
import { printUsage, runCommand } from '@fadroma/cli'
import { on, cargo, genCoverage, genSchema, genDocs, runTests, runDemo,
         resetLocalnet, openFaucet } from './lib/index'
import { withNetwork } from './lib/network'
import shell from './lib/shell'
import printStatus from './lib/status'

import { SiennaTGE, SiennaSwap, SiennaRewards, SiennaLend } from './ensembles'

export type CommandName = string
export type CommandInfo = string
export type Command     = [CommandName|Array<CommandName>, CommandInfo, Function, Commands?]
export type Commands    = Array<Command|null>

export default async function main (command: CommandName, ...args: any) {

  const tge     = new SiennaTGE()
      , rewards = new SiennaRewards()
      , amm     = new SiennaSwap()
      , lend    = new SiennaLend()

  const commands: Commands = [
    [["help", "--help", "-h"], "❓ Print usage", () => printUsage({}, commands)],

    null,
    ["docs",     "📖 Build the documentation and open it in a browser.",  genDocs],
    ["test",     "⚗️  Run test suites for all the individual components.", runTests],
    ["coverage", "📔 Generate test coverage and open it in a browser.",   genCoverage],
    ["schema",   "🤙 Regenerate JSON schema for each contract's API.",    genSchema],
    ["build", "👷 Compile contracts from source", null, [
      ["all",     "all contracts in workspace",                () => cargo('build')],
      ["tge",     "snip20-sienna, mgmt, rpt",                  () => tge.build()],
      ["rewards", "snip20-sienna, rewards",                    () => rewards.build()],
      ["amm",     "amm-snip20, factory, exchange, lp-token",   () => amm.build()],
      ["lend",    "snip20-lend + lend-atoken + configuration", () => lend.build()]]],

    null,
    ["tge",     "🚀 SIENNA token + vesting",         null,
      [...tge.localCommands(),     null, ...withNetwork(SiennaTGE)]],
    ["amm",     "💱 Contracts of Sienna Swap/AMM",   null,
      [...amm.localCommands(),     null, ...withNetwork(SiennaSwap)]],
    ["rewards", "🏆 SIENNA token + staking rewards", null,
      [...rewards.localCommands(), null, ...withNetwork(SiennaRewards)]],
    ["lend",    "🏦 Contracts of Sienna Lend",       null,
      [...lend.localCommands(),    null, ...withNetwork(SiennaLend)]],

    null,
    ["mainnet",  "Deploy and run contracts on the mainnet with real money.", on.mainnet, [
      ["shell",  "🐚 Launch a JavaScript REPL for talking to contracts directly", shell],
      ...remoteCommands('mainnet')]],
    ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.", on.testnet, [
      ["shell",  "🐚 Launch a JavaScript REPL for talking to contracts directly", shell],
      ["faucet", "🚰 Open https://faucet.secrettestnet.io/ in your default browser", openFaucet],
      ["fund",   "👛 Creating test wallets by sending SCRT to them.",                ensureWallets],
      ...remoteCommands('testnet')]],
    ["localnet", "Deploy and run contracts in a local container.", on.localnet, [
      ["shell",  "🐚 Launch a JavaScript REPL for talking to contracts directly", shell],
      ["reset",  "Remove the localnet container and clear its stored state",      resetLocalnet],
      ["fund",   "👛 Create test wallets by sending SCRT to them.",             ensureWallets],
      ...remoteCommands('localnet')]]]

  return await runCommand({ command: [ command ] }, commands, command, ...args) }

try {
  process.on('unhandledRejection', onerror)
  main(argv[2], ...argv.slice(3)) }
catch (e) {
  onerror(e) }

function onerror (e: Error) {
  console.error(e)
  const ISSUES = `https://github.com/SiennaNetwork/sienna/issues`
  console.info(`🦋 That was a bug! Report it at ${ISSUES}`)
  process.exit(1) }

function remoteCommands (network: any): Commands {
  return [
    ["status", "Show stored receipts.", printStatus],
    null,
    ["tge",     "🚀 SIENNA token + vesting",
      null, new SiennaTGE({network}).remoteCommands()],
    ["amm",     "💱 Contracts of Sienna Swap/AMM",
      null, new SiennaSwap({network}).remoteCommands()],
    ["rewards", "🏆 SIENNA token + staking rewards",
      null, new SiennaRewards({network}).remoteCommands()],
    ["lend",    "🏦 Contracts of Sienna Lend",
      null, new SiennaLend({network}).remoteCommands()]] }
