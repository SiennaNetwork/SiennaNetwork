#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

module.exports = main
if (require.main === module) main()
async function main (
  output = require('./output'),
  client = require('./client')(),
) {
  client = await Promise.resolve(client)

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
