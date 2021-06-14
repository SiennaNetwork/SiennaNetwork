#!/usr/bin/env node
import { argv } from 'process'
//import ensureWallets from '@fadroma/scrt-agent/fund.js'
import Localnet from '@fadroma/scrt-ops/localnet.js'
import { table, noBorders, bold } from '@fadroma/utilities'

import { args, cargo, genCoverage, genSchema, genDocs, runTests, runDemo } from './lib/index.js'
import TGE from './TGEContracts.js'
import Rewards from './RewardsContracts.ts'
import AMM from './AMMContracts.ts'
import Lend from './LendContracts.ts'

export type Command      = [CommandNames, CommandInfo, Function|null, ...any]
export type CommandList  = Array<Command|null>
export type CommandNames = string|Array<string>
export type CommandName  = string
export type CommandInfo  = string
export interface CommandContext {
  command?: Array<CommandName>
}

const components = [
  ["tge",     "🚀 SIENNA token + vesting",       null, ...new TGE().commands],
  ["rewards", "🏆 SIENNA token + rewards",       null, ...new Rewards().commands],
  ["amm",     "💱 Contracts of Sienna Swap/AMM", null, ...new AMM().commands],
  ["lend",    "🏦 Contracts of Sienna Lend",     null, ...new Lend().commands],
]

export const commands: CommandList = [
  [["help", "--help", "-h"], "❓ Print usage",
    () => printUsage({}, commands)],

  null,
  ["docs",     "📖 Build the documentation and open it in a browser.",  genDocs],
  ["test",     "⚗️  Run test suites for all the individual components.", runTests],
  ["coverage", "⚗️  Generate test coverage and open it in a browser.",   genCoverage],
  ["schema",   "🤙 Regenerate JSON schema for each contract's API.",    genSchema],

  null,
  ["build", "Compile contracts from source", null,
    ["all",     "all contracts in workspace", () => cargo('build')        ],
    ["tge",     "snip20-sienna, mgmt, rpt",   () => new TGE().build()     ],
    ["rewards", "snip20-sienna, rewards",     () => new Rewards().build() ],
    ["amm",     "amm-snip20, factory, exchange, lp-token",
                                              () => new AMM().build()     ],
    ["lend",    "snip20-lend-experimental + lend-atoken-experimental + lend-configuration",
                                              () => new Lend().build()    ]],

  null,
  ["localnet", "Deploy and run contracts in a local container.", selectLocalnet,
    ["reset",  "Remove the localnet container and clear its stored state", resetLocalnet],
    ["fund",   "Pre-seed some test wallets.", ensureWallets],
    ...components],
  ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.", selectTestnet,
    ["faucet", "🚰 Open https://faucet.secrettestnet.io/ in your default browser", openFaucet],
    ["fund",   "Pre-seed some test wallets.", ensureWallets],
    ...components],
  ["mainnet",  "Deploy and run contracts on the mainnet with real money.", selectMainnet,
    ...components],
]

function ensureWallets () {
  console.warn('not implemented')
}
function selectLocalnet () {
  console.debug(`Running on ${bold('localnet')}`)
  console.warn('not implemented')
}
function resetLocalnet () {
  return new Localnet().terminate()
}
function selectTestnet () {
  console.debug(`Running on ${bold('testnet')}`)
  console.warn('not implemented')
}
function selectMainnet () {
  console.debug(`Running on ${bold('mainnet')}`)
  console.warn('not implemented')
}
function openFaucet () {
}

export default function main (command: CommandName, ...args: any) {
  const context = { command: [ command ] }
  runCommand(context, commands, command, ...args)
}

function runCommand (
  context:       CommandContext,
  commands:      CommandList,
  commandToRun:  CommandName,
  ...args:       any
) {
  if (commandToRun) {
    let notFound = true
    for (const command of commands.filter(Boolean)) {
      if (!command) continue
      const [nameOrNames, info, fn, ...rest] = command
      if (
        (typeof nameOrNames === 'string' && nameOrNames === commandToRun) ||
        (nameOrNames instanceof Array  && nameOrNames.indexOf(commandToRun) > -1)
      ) {
        notFound = false
        let notImplemented = true
        if (fn) {
          context = fn(context, ...args)
          notImplemented = false
        }
        const subcommands = rest as Array<Command>
        if (subcommands && subcommands.length > 0) {
          runCommand(context, subcommands, args[0], ...args.slice(1))
          notImplemented = false
        }
        if (notImplemented) {
          console.warn(`${commandToRun}: not implemented`)
        }
      }
    }
    if (notFound) {
      console.warn(`${commandToRun}: no such command`)
    }
  } else {
    printUsage(context, commands)
  }
}

function printUsage (
  context:   CommandContext = {},
  commands:  CommandList,
  tableData: Array<[string, string]> = [],
  depth = 0
) {
  if (depth === 0) {
    console.log(`sienna ${(context.command||[]).join(' ')} [COMMAND...]\n`)
  }
  for (const commandSpec of commands) {
    if (commandSpec) {
      let [command, docstring, fn, subcommands] = commandSpec
      if (command instanceof Array) command = command.join(', ')
      tableData.push([`  ${bold(command)}`, docstring])
    } else {
      tableData.push(['',''])
    }
  }
  console.log(table(tableData, noBorders))
}

try {
  main(argv[2], ...argv.slice(3))
} catch (e) {
  console.error(e)
  const ISSUES = `https://github.com/SiennaNetwork/sienna/issues`
  console.info(`🦋 That was a bug! Report it at ${ISSUES}`)
}
