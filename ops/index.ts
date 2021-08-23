#!/usr/bin/env node
import { Chain, Scrt, prefund,
         CommandName, Commands, runCommand, printUsage,
         on, resetLocalnet, openFaucet } from '@hackbg/fadroma'

import { cargo, genCoverage, genSchema, genDocs, runTests, shell /*runDemo,*/ } from './lib/index'
import { SiennaTGE, SiennaSwap, SiennaRewards, SiennaLend } from './ensembles'

export default async function main (command: CommandName, ...args: any) {

  const tge     = new SiennaTGE()
      , rewards = new SiennaRewards()
      , amm     = new SiennaSwap()
      , lend    = new SiennaLend()

  function remoteCommands (chain: Chain): Commands {
    return [
      ["status",  HELP.STATUS,  () => chain.printStatusTables()],
      null,
      ["tge",     HELP.TGE,     null, new SiennaTGE({chain}).remoteCommands()],
      ["amm",     HELP.AMM,     null, new SiennaSwap({chain}).remoteCommands()],
      ["rewards", HELP.REWARDS, null, new SiennaRewards({chain}).remoteCommands()],
      ["lend",    HELP.LEND,    null, new SiennaLend({chain}).remoteCommands()]] }

  const commands: Commands = [
    [["help", "--help", "-h"], HELP.USAGE, () => printUsage({}, commands)],

    null,

    ["docs",     HELP.DOCS,     genDocs],
    ["test",     HELP.TEST,     runTests],
    ["coverage", HELP.COVERAGE, genCoverage],
    ["schema",   HELP.SCHEMA,   genSchema],
    ["build",    HELP.BUILD, null, [
      ["all",     HELP.BUILD_ALL,     () => cargo('build')],
      ["tge",     HELP.BUILD_TGE,     () => tge.build()],
      ["rewards", HELP.BUILD_REWARDS, () => rewards.build()],
      ["amm",     HELP.BUILD_AMM,     () => amm.build()],
      ["lend",    HELP.BUILD_LEND,    () => lend.build()]]],

    null,

    ["tge",     HELP.TGE,     null,
      [...tge.localCommands(),     null, ...Scrt.chainSelector(SiennaTGE)    ] as Commands],
    ["amm",     HELP.AMM,     null,
      [...amm.localCommands(),     null, ...Scrt.chainSelector(SiennaSwap)   ] as Commands],
    ["rewards", HELP.REWARDS, null,
      [...rewards.localCommands(), null, ...Scrt.chainSelector(SiennaRewards)] as Commands],
    ["lend",    HELP.LEND,    null,
      [...lend.localCommands(),    null, ...Scrt.chainSelector(SiennaLend)   ] as Commands],

    null,

    ["mainnet",  HELP.MAINNET, on.mainnet,   [
      ["shell",  HELP.SHELL,   shell],
      ...remoteCommands(Scrt.mainnet())]],
    ["testnet",  HELP.TESTNET, on.testnet,   [
      ["shell",  HELP.SHELL,   shell],
      ["faucet", HELP.FAUCET,  openFaucet],
      ["fund",   HELP.FUND,    prefund],
      ...remoteCommands(Scrt.testnet())]],
    ["localnet", HELP.LOCALNET, on.localnet, [
      ["shell",  HELP.SHELL,   shell],
      ["reset",  HELP.FAUCET,  resetLocalnet],
      ["fund",   HELP.FUND,    prefund],
      ...remoteCommands(Scrt.localnet())]]]

  return await runCommand({ command: [ command ] }, commands, command, ...args) }

export const HELP = {
  USAGE:    "❓ Print usage info",
  STATUS:   "Show stored receipts from uploads and instantiations.",

  TGE:      "🚀 SIENNA token + vesting",
  AMM:      "💱 Contracts of Sienna Swap/AMM",
  REWARDS:  "🏆 SIENNA token + staking rewards",
  LEND:     "🏦 Contracts of Sienna Lend",

  DOCS:     "📖 Build the documentation and open it in a browser.",
  TEST:     "⚗️  Run test suites for all the individual components.",
  COVERAGE: "📔 Generate test coverage and open it in a browser.",
  SCHEMA:   "🤙 Regenerate JSON schema for each contract's API.",

  BUILD:         "👷 Compile contracts from source",
  BUILD_ALL:     "all contracts in workspace",
  BUILD_TGE:     "snip20-sienna, mgmt, rpt",
  BUILD_REWARDS: "snip20-sienna, rewards",
  BUILD_AMM:     "amm-snip20, factory, exchange, lp-token",
  BUILD_LEND:    "snip20-lend + lend-atoken + configuration",

  MAINNET:  "Interact with the Secret Network mainnet.",
  TESTNET:  "Deploy and run contracts on the holodeck-2 testnet.",
  LOCALNET: "Run a Secret Network instance in a local container.",

  SHELL:  "🐚 Launch a JavaScript REPL for talking to contracts directly",
  FAUCET: "🚰 Open https://faucet.secrettestnet.io/ in your default browser",
  FUND:   "👛 Creating test wallets by sending SCRT to them."}
