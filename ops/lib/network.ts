import { ScrtNode } from '@fadroma/localnet'
import { Ensemble } from '@fadroma/ensemble'
import open from 'open'
import colors from 'colors/safe.js'
const { bold } = colors

export function withNetwork (E: Ensemble) {
  return [
    ["mainnet", "Deploy and run contracts on the mainnet with real money.",
      on.mainnet,  new E({network: 'mainnet'}).remoteCommands()],
    ["testnet",  "Deploy and run contracts on the holodeck-2 testnet.",
      on.testnet,  new E({network: 'testnet'}).remoteCommands()],
    ["localnet", "Deploy and run contracts in a local container.",
      on.localnet, new E({network: 'localnet'}).remoteCommands()] ] }

export const on = {
  localnet (context: any = {}) {
    console.debug(`Running on ${bold('localnet')}:`)
    context.network = 'localnet' },
  testnet (context: any = {}) {
    console.debug(`Running on ${bold('testnet')}:`)
    context.network = 'testnet' },
  mainnet (context: any = {}) {
    console.debug(`Running on ${bold('mainnet')}:`)
    context.network = 'mainnet' } }

export function resetLocalnet () {
  return new ScrtNode().terminate() }

export function openFaucet () {
  const url = `https://faucet.secrettestnet.io/`
  console.debug(`Opening ${url}...`)
  open(url) }
