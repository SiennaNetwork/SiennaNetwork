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

  function getSettings (directory, file) {
    const path = resolve(directory, file)
    if (!existsSync(path)) {
      throw new Error(`settings/${chainId}/${file} does not exist`)
    }
    return JSON.parse(readFileSync(path))
  }
}

module.exports.workspace = dirname(__dirname)

module.exports.schedule = JSON.parse(readFileSync(resolve(__dirname, 'schedule.json'), 'utf8'))
