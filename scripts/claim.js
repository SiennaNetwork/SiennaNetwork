#!/usr/bin/env node
const $ = require('./lib')(module,
  async function claim ({
    say   = require('./lib/say')("[claim]"),
    env   = require('./lib/env')(),
    agent = require('./agent').fromEnvironment(),
    mgmt  = JSON.parse(process.env.MGMT||"{}"),
  }={}) {
    agent = await Promise.resolve(agent)
    return say(await agent.execute(mgmt.address, {claim: {}}))
  })
