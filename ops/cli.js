#!/usr/bin/env node
import { argv } from 'process'
import yargs from 'yargs'
import ensureWallets from '@fadroma/scrt-agent/fund.js'
import Localnet from '@fadroma/scrt-ops/localnet.js'
import { table, noBorders, bold } from '@fadroma/utilities'

import { args, cargo, genCoverage, genSchema, genDocs, runTests, runDemo } from './lib/index.js'
import TGE from './TGEContracts.js'
import Rewards from './RewardsContracts.js'
import AMM from './AMMContracts.js'
import Lend from './LendContracts.js'

const selectLocalnet = () => {
  console.debug('running on localnet')
}
const resetLocalnet = () => {
  return new Localnet().terminate()
}
const selectTestnet  = () => {
  console.debug(`Running on ${bold('testnet')}`)
}
const selectMainnet  = () => {
  console.debug(`Running on ${bold('mainnet')}`)
}
const openFaucet = () => {
}

export default function main (command, ...args) {

  const context = { command: [ command ] }

  const components = [
    ["tge",     "🚀 SIENNA token + vesting",       null, ...new TGE().commands],
    ["rewards", "🏆 SIENNA token + rewards",       null, ...new Rewards().commands],
    ["amm",     "💱 Contracts of Sienna Swap/AMM", null, ...new AMM().commands],
    ["lend",    "🏦 Contracts of Sienna Lend",     null, ...new Lend().commands],
  ]

  const commands = [
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

  runCommand(context, commands, command, ...args)

    //[>* Deploy and run in a local container. <]
    //"localnet": (...args) => {
      //[>* '♻️  Try to terminate a loose localnet container and remove its state files <]
      //if (cmd === 'reset') {
        //return new Localnet().terminate()
      //}
    //},

    //[>* Deploy and run on the holodeck-2 testnet. <]
    //"testnet": (...args) => {}

    //[>* Deploy and run on the holodeck-2 testnet. <]
    //"mainnet": (...args) => {}
  //}

  //const subCommands = network => {
    //[> ⚗️  Ensure there are testnet wallets for the demo. <]
    //"fund": ensureWallets(args),

    //"tge": new TGEContracts().commands2()

    //"rewards": new RewardsContracts().commands2(cmd)

    //"amm": new AMMContracts().commands2(cmd)

    ////"lend": (...args) {}
  //}

  //let cmd = yargs(argv.slice(2))
    //.scriptName('sienna')
    //.wrap(yargs().terminalWidth())
    //.demandCommand(1, '')
    //.command('docs [crate]',
      //'📖 Build the documentation and open it in a browser.',
      //args.Crate, genDocs)
    //.command('clean-localnet',
      
      //() => new Localnet().terminate())
    //.command('demo [--testnet]',
      //'⚗️  Run integration test/demo.',
      //args.IsTestnet, runDemo)
    //.command('ensure-wallets',
      //,
      //ensureWallets)
    //.command('schema',
      //,
      //genSchema)

  //return cmd.argv
}

function runCommand (context, commands, command, ...args) {
  if (command) {
    let notFound = true
    for (const [nameOrNames, info, fn, ...subcommands] of commands.filter(Boolean)) {
      if (
        nameOrNames === command ||
        (nameOrNames instanceof Array && nameOrNames.indexOf(command) > -1)
      ) {
        notFound = false
        let notImplemented = true
        if (fn) {
          context = fn(context, ...args)
          notImplemented = false
        }
        if (subcommands && subcommands.length > 0) {
          runCommand(context, subcommands, args[0], ...args.slice(1))
          notImplemented = false
        }
        if (notImplemented) {
          console.warn(`${command}: not implemented`)
        }
      }
    }
    if (notFound) {
      console.warn(`${command}: no such command`)
    }
  } else {
    printUsage(context, commands)
  }
}

function printUsage (context = { command: [] }, commands, tableData = [], depth = 0) {
  if (depth === 0) {
    console.log(`sienna ${context.command.join(' ')} [COMMAND...]\n`)
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
  main(...argv.slice(2))
} catch (e) {
  console.error(e)
  const ISSUES = `https://github.com/hackbg/sienna-secret-token/issues`
  console.info(`👹 That was a bug. Report it at ${ISSUES}`)
}
