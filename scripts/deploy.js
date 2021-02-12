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
               , send:   gas(  80000) },
  output = (x={}) => {
    if (x.data instanceof Uint8Array) x.data = new TextDecoder('utf-8').decode(x.data)
    console.log(require('prettyjson').render(x))
  }
) {

  const client = await require('./client')(httpUrl, mnemonic, customFees)

  output('deploying token...')
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
  output(token)
  require('fs').appendFileSync(envfile,
    `\nTOKEN=${JSON.stringify(token)}`)

  output('deploying mgmt...')
  const mgmt = await client.deploy(
    `${__dirname}/../dist/sienna-mgmt.wasm`,
    `SIENNA MGMT (${new Date().toISOString()})`, {
      token_addr: token.address,
      token_hash: token.hash
    })
  output(mgmt)
  require('fs').appendFileSync(envfile,
    `\nMGMT=${JSON.stringify(mgmt)}`)

  output('allowing mgmt to mint tokens...')
  output(
    await client.execute(token.address,
      { set_minters: { minters: [mgmt.address] } }))

  output('transferring ownership of token to mgmt...')
  output(
    await client.execute(token.address,
      { change_admin: { address: mgmt.address } }))

  return { client, mgmt, token }

}
