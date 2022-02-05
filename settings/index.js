import Konzola from '@hackbg/konzola'
import { resolve, dirname } from 'path'
import { existsSync, readFileSync } from 'fs'
import { fileURLToPath } from 'url'

const console = Konzola.default('@sienna/settings')

import YAML from 'js-yaml'

export default function getSettingsForChain (chainId) {
  const source = resolve(__dirname, 'by-chain-id', chainId + '.yml')
  //console.info('Getting settings from', source)
  if (!existsSync(source)) {
    throw new Error(`settings/by-chain-id/${chainId}.yml does not exist`)
  }
  const content = readFileSync(source, 'utf8')
  //console.log(content)
  const settings = YAML.load(content)
  return settings
}

export const __dirname = dirname(fileURLToPath(import.meta.url))

export const workspace = dirname(__dirname)

export const abs = (...args) => resolve(module.exports.workspace, ...args)

export const schedule = JSON.parse(readFileSync(resolve(__dirname, 'schedule.json'), 'utf8'))

export const SIENNA_DECIMALS = 18

export const ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`)
