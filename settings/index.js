const { resolve, basename, dirname } = require('path')
const { readdirSync, readFileSync } = require('fs')

module.exports = readdirSync(__dirname).filter(isConfigFile).reduce(exportConfigFile, {})

function isConfigFile (file) {
  if (file === 'package.json') return false
  return file.endsWith('.json')
}

function exportConfigFile (output, file) {
  //console.info(`loading ${file}`)
  output[basename(file, '.json')] = JSON.parse(readFileSync(resolve(__dirname, file), 'utf8'))
  return output
}

module.exports.workspace = dirname(__dirname)
