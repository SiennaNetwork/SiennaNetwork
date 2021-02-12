#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

module.exports = main
if (require.main === module) main()
async function main (
  output = require('./output'),
  client = require('./client')(),

  MGMT   = JSON.parse(process.env.MGMT||"{}"),
) {
  client = await Promise.resolve(client)
  output(await client.execute(MGMT.address,
    {launch: {}}))
}
