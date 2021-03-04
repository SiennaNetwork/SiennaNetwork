#!/usr/bin/env node
require('./lib')(module, async function launch ({
  say   = require('./lib/say')('[launch]'),
  agent = require('./lib/agent').fromEnvironment(),
  mgmt  = JSON.parse(process.env.MGMT||"{}"),
} = {}) {
  say(`launch ${mgmt} as ${agent.addr}`)
  agent = await Promise.resolve(agent)
  say()
})
