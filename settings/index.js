import Konzola from '@hackbg/konzola'
import { resolve, dirname } from 'path'
import { existsSync, readFileSync } from 'fs'
import { fileURLToPath } from 'url'

const console = Konzola.default('@sienna/settings')

import YAML from 'js-yaml'

export const __dirname = dirname(fileURLToPath(import.meta.url))

export const workspace = dirname(__dirname)

export const abs = (...args) => resolve(module.exports.workspace, ...args)

export default function getSettingsForChain (mode) {

  // Make sure the settings file exists
  const source = resolve(__dirname, 'by-mode', `${mode}.yml`)
  if (!existsSync(source)) {
    throw new Error(`settings/by-mode/${mode}.yml does not exist`)
  }

  // Load the settings from the settings file
  const content  = readFileSync(source, 'utf8')
  const settings = YAML.load(content)

  // The TGE vesting schedule is global, but in the future it
  // may need to be modified for devnet deployments.
  // So reexport it through the chain settings, too.
  settings.schedule = schedule

  return settings
}

export const schedule = JSON.parse(readFileSync(resolve(__dirname, 'schedule.json'), 'utf8'))

export const SIENNA_DECIMALS = 18
export const ONE_SIENNA = BigInt(
  `1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`
)
