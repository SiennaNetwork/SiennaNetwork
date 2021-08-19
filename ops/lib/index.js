import open from 'open'
import colors from 'colors/safe.js'
const { bold } = colors
import { ScrtNode } from '@fadroma/localnet'

import { abs, projectRoot } from './root.js'
import { fmtSIENNA } from './decimals.js'
import { genCoverage, genSchema, genDocs } from './gen.js'
import { cargo, runTests, runDemo } from './run.js'

export function selectLocalnet (context = {}) {
  console.debug(`Running on ${bold('localnet')}:`)
  context.network = 'localnet'
}

export function resetLocalnet () {
  return new ScrtNode().terminate()
}

export function selectTestnet (context = {}) {
  console.debug(`Running on ${bold('testnet')}:`)
  context.network = 'testnet'
}

export function selectMainnet (context = {}) {
  console.debug(`Running on ${bold('mainnet')}:`)
  context.network = 'mainnet'
}

export function openFaucet () {
  const url = `https://faucet.secrettestnet.io/`
  console.debug(`Opening ${url}...`)
  open(url)
}

export {
  abs,
  cargo,
  fmtSIENNA,
  projectRoot,
  genCoverage,
  genSchema,
  genDocs,
  runTests,
  runDemo
}
