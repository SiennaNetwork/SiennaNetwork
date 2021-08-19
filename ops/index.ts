#!/usr/bin/env node
import { argv } from 'process'
import ensureWallets from '@fadroma/agent/scrt_fund.js'
import { Scrt } from '@fadroma/agent'
import { basename, extname, resolve, readdirSync, readFile } from '@fadroma/util-sys'
import { bold, table, noBorders, printUsage, runCommand } from '@fadroma/cli'
import { args, cargo, genCoverage, genSchema, genDocs, runTests, runDemo,
         selectLocalnet, selectTestnet, selectMainnet,
         resetLocalnet, openFaucet } from './lib/index.js'
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

const printStatus = async ({network}) => {
  const { receipts, instances } = Scrt.hydrate(network)

  const idToName = {}

  const uploadReceipts = [[
    bold('  code id'), bold('name\n'), bold('size'), bold('hash')
  ]].concat(await Promise.all(readdirSync(receipts).map(async x=>{
    x = resolve(receipts, x)
    const {codeId, originalSize, compressedSize, originalChecksum, compressedChecksum} = JSON.parse(await readFile(x))
    const name = idToName[codeId] = basename(x, '.upload.json')
    return [
      `  ${codeId}`,
      `${bold(name)}\ncompressed:\n`,
      `${originalSize}\n${String(compressedSize).padStart(String(originalSize).length)}`,
      `${originalChecksum}\n${compressedChecksum}`]
  })))

  if (uploadReceipts.length > 1) {
    console.log(`\nUploaded binaries on ${bold(network)}:`)
    console.log('\n'+table(uploadReceipts, noBorders))
  } else {
    console.log(`\n  No known uploaded binaries on ${bold(network)}`)
  }

  const initReceipts = [[
    bold('  label')+'\n  address', '(code id) binary name\ncode hash\ninit tx\n'
  ]].concat(await Promise.all(readdirSync(instances).map(async x=>{
    x = resolve(instances, x)
    const name = basename(x, '.json')
    const {codeId, codeHash, initTx} = await JSON.parse(await readFile(x))
    const {contractAddress, transactionHash} = initTx
    return [
      `  ${bold(name)}\n  ${contractAddress}`,
      `(${codeId}) ${idToName[codeId]||''}\n${codeHash}\n${transactionHash}\n`,
      //`${contractAddress}\n${transactionHash}`,
    ]
  })))

  if (initReceipts.length > 1) {
    console.log(`Instantiated contracts on ${bold(network)}:`)
    console.log('\n'+table(initReceipts, noBorders))
  } else {
    console.log(`\n  No known contracts on ${bold(network)}`)
  }

}

const remoteCommands = network => [
  ["status", "Show stored receipts.", printStatus],
  null,
  ["tge",     "🚀 SIENNA token + vesting",         null, new TGE({network}).remoteCommands],
  ["rewards", "🏆 SIENNA token + staking rewards", null, new Rewards({network}).remoteCommands],
  ["amm",     "💱 Contracts of Sienna Swap/AMM",   null, new Swap({network}).remoteCommands],
  ["lend",    "🏦 Contracts of Sienna Lend",       null, new Lend({network}).remoteCommands],
]

const withNetwork = Ensemble => [
  ["mainnet",  "Deploy and run contracts on the mainnet with real money.", selectMainnet,
    new Ensemble({network: 'mainnet'}).remoteCommands],
  ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.",      selectTestnet,
    new Ensemble({network: 'testnet'}).remoteCommands],
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
    ["shell",  "🐚 Launch a JavaScript REPL for talking to contracts directly", shell],
    ...remoteCommands('mainnet')]],
  ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.", selectTestnet, [
    ["shell",  "🐚 Launch a JavaScript REPL for talking to contracts directly", shell],
    ["faucet", "🚰 Open https://faucet.secrettestnet.io/ in your default browser", openFaucet],
    ["fund",   "👛 Creating test wallets by sending SCRT to them.",                ensureWallets],
    ...remoteCommands('testnet')]],
  ["localnet", "Deploy and run contracts in a local container.", selectLocalnet, [
    ["shell",  "🐚 Launch a JavaScript REPL for talking to contracts directly", shell],
    ["reset",  "Remove the localnet container and clear its stored state",      resetLocalnet],
    ["fund",   "👛 Creating test wallets by sending SCRT to them.",             ensureWallets],
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
