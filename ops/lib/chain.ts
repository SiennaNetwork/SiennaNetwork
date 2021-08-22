import { Chain, Scrt, ScrtNode, Ensemble, EnsembleOptions, bold } from '@hackbg/fadroma'
import open from 'open'

export function onChain (
  E: new (args: EnsembleOptions) => Ensemble
) {
  return [
    ["mainnet", "Deploy and run contracts on the mainnet with real money.",
      on.mainnet,  new E({chain: Scrt.mainnet() as Chain}).remoteCommands()],
    ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.",
      on.testnet,  new E({chain: Scrt.testnet() as Chain}).remoteCommands()],
    ["localnet", "Deploy and run contracts in a local container.",
      on.localnet, new E({chain: Scrt.localnet() as Chain}).remoteCommands()] ] }

export const on = {
  localnet (context: any = {}) {
    console.debug(`Running on ${bold('localnet')}:`)
    context.chain = 'localnet' },
  testnet (context: any = {}) {
    console.debug(`Running on ${bold('testnet')}:`)
    context.chain = 'testnet' },
  mainnet (context: any = {}) {
    console.debug(`Running on ${bold('mainnet')}:`)
    context.chain = 'mainnet' } }

export function resetLocalnet () {
  return new ScrtNode().terminate() }

export function openFaucet () {
  const url = `https://faucet.secrettestnet.io/`
  console.debug(`Opening ${url}...`)
  open(url) }
