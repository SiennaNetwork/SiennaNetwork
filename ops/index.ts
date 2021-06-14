#!/usr/bin/env node
import { argv } from 'process'
import ensureWallets from '@fadroma/scrt-agent/fund.js'
import Localnet from '@fadroma/scrt-ops/localnet.js'
import { table, noBorders, bold, runCommand, printUsage } from '@fadroma/utilities'

import {
  args, cargo, genCoverage, genSchema, genDocs, runTests, runDemo,
  ensureWallets, selectLocalnet, resetLocalnet, selectTestnet, openFaucet, selectMainnet
} from './lib/index.js'
import TGE from './TGEContracts.js'
import Rewards from './RewardsContracts.ts'
import AMM from './AMMContracts.ts'
import Lend from './LendContracts.ts'

const components = [
  ["tge",     "🚀 SIENNA token + vesting",       null, new TGE().commands],
  ["rewards", "🏆 SIENNA token + rewards",       null, new Rewards().commands],
  ["amm",     "💱 Contracts of Sienna Swap/AMM", null, new AMM().commands],
  ["lend",    "🏦 Contracts of Sienna Lend",     null, new Lend().commands],
]

export const commands: CommandList = [
  [["help", "--help", "-h"], "❓ Print usage",
    () => printUsage({}, commands)],

  ["docs",     "📖 Build the documentation and open it in a browser.",  genDocs],
  ["test",     "⚗️  Run test suites for all the individual components.", runTests],
  ["coverage", "⚗️  Generate test coverage and open it in a browser.",   genCoverage],
  ["schema",   "🤙 Regenerate JSON schema for each contract's API.",    genSchema],

  ["build", "👷 Compile contracts from source", null, [
    ["all",     "all contracts in workspace", () => cargo('build')        ],
    ["tge",     "snip20-sienna, mgmt, rpt",   () => new TGE().build()     ],
    ["rewards", "snip20-sienna, rewards",     () => new Rewards().build() ],
    ["amm",     "amm-snip20, factory, exchange, lp-token",
                                              () => new AMM().build()     ],
    ["lend",    "snip20-lend-experimental + lend-atoken-experimental + lend-configuration",
                                              () => new Lend().build()    ]]],

  ["mainnet",  "Deploy and run contracts on the mainnet with real money.", selectMainnet, [
    ...components]],

  ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.", selectTestnet, [
    ["faucet", "🚰 Open https://faucet.secrettestnet.io/ in your default browser", openFaucet],
    ["fund",   "👛 Creating test wallets by sending SCRT to them.", ensureWallets],
    ...components]],

  ["localnet", "Deploy and run contracts in a local container.", selectLocalnet, [
    ["reset",  "Remove the localnet container and clear its stored state", resetLocalnet],
    ["fund",   "👛 Creating test wallets by sending SCRT to them.", ensureWallets],
    ...components]],
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
