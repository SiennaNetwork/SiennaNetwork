#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

const gas = x => ({amount:[{amount:String(x),denom:'uscrt'}],gas:String(x)})

module.exports = main
if (require.main === module) main()
async function main (
  httpUrl    = process.env.SECRET_REST_URL || 'http://localhost:1337',
  mnemonic   = process.env.MNEMONIC,
  customFees = { upload: gas(3000000)
               , init:   gas( 500000)
               , exec:   gas( 500000)
               , send:   gas(  80000) }
) {

  const client = await require('./client')(httpUrl, mnemonic, customFees)

  console.log('deploying token...')
  const token = await client.deploy(
    `${__dirname}/../dist/snip20-reference-impl.wasm`,
    `SIENNA SNIP20 (${new Date().toISOString()})`, {
      name:      "Sienna",
      symbol:    "SIENNA",
      decimals:  18,
      admin:     client.address,
      prng_seed: "insecure",
      config:    { public_total_supply: true }
    })
  console.log(token)
  require('fs').appendFileSync(envfile,
    `\nTOKEN=${JSON.stringify(token)}`)

  console.log('deploying mgmt...')
  const mgmt = await client.deploy(
    `${__dirname}/../dist/sienna-mgmt.wasm`,
    `SIENNA MGMT (${new Date().toISOString()})`, {
      token_addr: token.address,
      token_hash: token.hash
    })
  console.log(mgmt)
  require('fs').appendFileSync(envfile,
    `\nMGMT=${JSON.stringify(mgmt)}`)

  return { client, mgmt, token }

}
