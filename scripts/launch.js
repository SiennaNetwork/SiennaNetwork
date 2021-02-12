#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

module.exports = main
if (require.main === module) main()
async function main (
  gas = require('./gas'),
  client = require('./client')(
    process.env.SECRET_REST_URL || 'http://localhost:1337',
    process.env.MNEMONIC,
    { upload: gas(3000000)
    , init:   gas( 500000)
    , exec:   gas( 500000)
    , send:   gas(  80000) }),
  MGMT = JSON.parse(process.env.MGMT||"{}"),
  output = x => console.log(require('prettyjson').render(x))
) {
  client = await Promise.resolve(client)
  output(await client.execute(MGMT.address,
    {launch: {}}))
}
