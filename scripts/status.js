#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

module.exports = main
if (require.main === module) main()
async function main (
  output = require('./output'),
  agent = require('./agent')(),

  MGMT   = JSON.parse(process.env.MGMT||"{}"),
) {
  agent = await Promise.resolve(agent)
  output(await agent.query(MGMT.address,
    { status: {} }))
  output(await agent.query(MGMT.address,
    { get_schedule: {} }))
}
