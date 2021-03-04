#!/usr/bin/env node
require('./lib')(module, async function config ({
  say      = require('./lib/say')('[config]'),
  agent    = require('./lib/agent').fromEnvironment(),
  schedule = require('../config.json'),
  mgmt     = JSON.parse(process.env.MGMT||"{}"),
}) {
  say(`configure ${mgmt} as ${agent.addr} with ${schedule}`)
  agent = await Promise.resolve(agent)
  say(await agent.execute(mgmt.address, { configure: { schedule }}))
})
