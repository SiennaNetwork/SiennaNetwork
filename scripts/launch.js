#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

async function launch (
  say    = require('./say')('[launch]'),
  agent = require('./agent').fromEnvironment(),
  mgmt   = JSON.parse(process.env.MGMT||"{}"),
) {
  say(`launch ${mgmt} as ${agent.addr}`)
  agent = await Promise.resolve(agent)
  say(await agent.execute(mgmt.address, {launch: {}}))
}

module.exports=(require.main&&require.main!==module)?launch:config()
