const { resolve, basename } = require('path')
const { readdirSync, readFileSync } = require('fs')
module.exports =
  readdirSync(__dirname)
    .filter(x=>x.endsWith('.json') && x!=='package.json')
    .reduce((output, file)=>{
      console.log(output, file)
      output[basename(file, '.json')] = JSON.parse(readFileSync(resolve(__dirname, file), 'utf8'))
      return output }, {})
