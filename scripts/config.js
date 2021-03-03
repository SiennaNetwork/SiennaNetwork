#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})
const envfile = require('path').resolve(__dirname, '../.env')
require('dotenv').config({ path: envfile })

async function config ({
  say      = require('./say')('[config]'),
  agent   = require('./agent').fromEnvironment(),
  schedule = require('../config.json'),
  mgmt     = JSON.parse(process.env.MGMT||"{}"),
}) {
  say(`configure ${mgmt} as ${agent.addr} with ${schedule}`)
  agent = await Promise.resolve(agent)
  say(await agent.execute(mgmt.address, { configure: { schedule }}))
}

module.exports=(require.main&&require.main!==module)?config:config()
