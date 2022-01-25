const { resolve, dirname } = require('path')
const { existsSync, readFileSync } = require('fs')

module.exports = function getSettingsForChain (chainId) {

  const directory = resolve(__dirname, chainId)
  if (!existsSync(directory)) {
    throw new Error(`settings/${chainId}/ does not exist`)
  }

  return {
    get amm () {
      return getSettings('amm.json')
    },
    get placeholderTokens () {
      return getSettings('placeholderTokens.json')
    },
    get rewardPairs () {
      return getSettings('rewardPairs.json')
    },
    get swapPairs () {
      return getSettings('swapPairs.json')
    },
    get swapTokens () {
      return getSettings('swapTokens.json')
    }
  }

  function getSettings (file) {
    const path = resolve(directory, file)
    if (!existsSync(path)) {
      throw new Error(`settings/${chainId}/${file} does not exist`)
    }
    return JSON.parse(readFileSync(path))
  }

}

module.exports.workspace = dirname(__dirname)

module.exports.abs = (...args) => resolve(module.exports.workspace, ...args)

module.exports.schedule = JSON.parse(readFileSync(resolve(__dirname, 'schedule.json'), 'utf8'))

const SIENNA_DECIMALS = module.exports.SIENNA_DECIMALS = 18

const ONE_SIENNA = module.exports.ONE_SIENNA = BigInt(`1${[...Array(SIENNA_DECIMALS)].map(()=>'0').join('')}`)
