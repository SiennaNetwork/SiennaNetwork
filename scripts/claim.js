#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

module.exports = main
if (require.main === module) main()
async function main (
  say    = require('./say')("[claim]"),
  agent = require('./agent')(),
  mgmt   = JSON.parse(process.env.MGMT||"{}"),
) {
  agent = await Promise.resolve(agent)
  return say(await agent.execute(mgmt.address, {claim: {}}))
}
