#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

module.exports = main
if (require.main === module) main()
async function main ({
  say      = require('./say')('[config]'),
  client   = require('./client')(),
  schedule = require('../config.json'),
  mgmt     = JSON.parse(process.env.MGMT||"{}"),
}) {
  client = await Promise.resolve(client)
  say(await client.execute(mgmt.address,
    { configure: { schedule }
  }))
}
