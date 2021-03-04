#!/usr/bin/env node
require('./lib')(module, async function status (
  say   = require('./lib/say')('[status]'),
  agent = require('./lib/agent')(),
  mgmt  = JSON.parse(process.env.mgmt||"{}"),
) {
  agent = await Promise.resolve(agent)
  return {
    status:   say(await agent.query(mgmt.address, { status: {} }))
    schedule: say(await agent.query(mgmt.address, { get_schedule: {} }))
  }
})
