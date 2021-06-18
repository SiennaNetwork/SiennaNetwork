#!/usr/bin/env node
import { argv } from 'process'
import ensureWallets from '@fadroma/scrt-agent/fund.js'
import Localnet from '@fadroma/scrt-ops/localnet.js'
import { table, noBorders, bold, runCommand, printUsage } from '@fadroma/utilities'

import {
  args, cargo, genCoverage, genSchema, genDocs, runTests, runDemo,
  ensureWallets, selectLocalnet, resetLocalnet, selectTestnet, openFaucet, selectMainnet
} from './lib/index.js'
import shell from './lib/shell.ts'
import TGE from './TGEContracts.js'
import Rewards from './RewardsContracts.ts'
import Swap from './AMMContracts.ts'
import Lend from './LendContracts.ts'

// Components of the project. Consist of multiple contracts and associated commands.
const tge     = new TGE()
const rewards = new Rewards()
const amm     = new Swap()
const lend    = new Lend()

const remoteCommands = network => [
  ["tge",     "🚀 SIENNA token + vesting",         null, new TGE({network}).remoteCommands],
  ["rewards", "🏆 SIENNA token + staking rewards", null, new Rewards({network}).remoteCommands],
  ["amm",     "💱 Contracts of Sienna Swap/AMM",   null, new Swap({network}).remoteCommands],
  ["lend",    "🏦 Contracts of Sienna Lend",       null, new Lend({network}).remoteCommands],
]

const withNetwork = Ensemble => [
  ["mainnet",  "Deploy and run contracts on the mainnet with real money.", selectMainnet,
    new Ensemble({network:  'mainnet'}).remoteCommands],
  ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.",      selectTestnet,
    new Ensemble({network:  'testnet'}).remoteCommands],
  ["localnet", "Deploy and run contracts in a local container.",           selectLocalnet,
    new Ensemble({network: 'localnet'}).remoteCommands],
]

export const commands: CommandList = [
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
    [...tge.localCommands,     null, ...withNetwork(TGE)]],
  ["rewards", "🏆 SIENNA token + staking rewards", null,
    [...rewards.localCommands, null, ...withNetwork(Rewards)]],
  ["amm",     "💱 Contracts of Sienna Swap/AMM",   null,
    [...amm.localCommands,     null, ...withNetwork(Swap)]],
  ["lend",    "🏦 Contracts of Sienna Lend",       null,
    [...lend.localCommands,    null, ...withNetwork(Lend)]],
  null,
  ["mainnet",  "Deploy and run contracts on the mainnet with real money.", selectMainnet, [
    ...remoteCommands('mainnet')]],
  ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.", selectTestnet, [
    ["faucet", "🚰 Open https://faucet.secrettestnet.io/ in your default browser", openFaucet],
    ["fund",   "👛 Creating test wallets by sending SCRT to them.",                ensureWallets],
    null
    ...remoteCommands('testnet')]],
  ["localnet", "Deploy and run contracts in a local container.", selectLocalnet, [
    ["shell",  "🐚 Launch a JavaScript REPL for talking to contracts directly", shell],
    ["reset",  "Remove the localnet container and clear its stored state",      resetLocalnet],
    ["fund",   "👛 Creating test wallets by sending SCRT to them.",             ensureWallets],
    null
    ...remoteCommands('localnet')]],
]

export default async function main (command: CommandName, ...args: any) {
  return await runCommand({ command: [ command ] }, commands, command, ...args)
}

try {
  process.on('unhandledRejection', onerror)
  main(argv[2], ...argv.slice(3))
} catch (e) {
  onerror(e)
}

function onerror (e: Error) {
  console.error(e)
  const ISSUES = `https://github.com/SiennaNetwork/sienna/issues`
  console.info(`🦋 That was a bug! Report it at ${ISSUES}`)
  process.exit(1)
}
