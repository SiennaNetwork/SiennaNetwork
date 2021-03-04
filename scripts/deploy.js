#!/usr/bin/env node
require('./lib')(module, async function deploy ({

  say   = require('./lib/say')('[deploy]'),
  env   = require('./lib/env')(),
  agent = require('./lib/agent').fromEnvironment(),

  version = `2021-03-02-80f6297`,

  token       = `${__dirname}/../dist/${version}-snip20-reference-impl.wasm`,
  tokenLabel  = `SIENNA SNIP20 (${new Date().toISOString()})`,
  tokenConfig = {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    admin:     agent.address,
    prng_seed: "insecure",
    config:    { public_total_supply: true }
  },

  mgmt       = `${__dirname}/../dist/${version}-sienna-mgmt.wasm`,
  mgmtLabel  = `SIENNA MGMT (${new Date().toISOString()})`,
  mgmtConfig = {},

  schedule = require('../config.json'),

}={}) {
  agent = await Promise.resolve(agent)
  say(await agent.status())

  say('deploying token...') ////////////////////////////////////////////////////////////////////////
  token = await agent.deploy(token, tokenLabel, tokenConfig)
  say(env.write('TOKEN', token))
  say(await agent.status())

  say('deploying mgmt...') /////////////////////////////////////////////////////////////////////////
  mgmt = await agent.deploy(mgmt, mgmtLabel, {
    token_addr: token.address,
    token_hash: token.hash,
    ...mgmtConfig,
  })
  say(env.write('MGMT', mgmt))
  say(await agent.status())

  say('allowing mgmt to mint tokens...') ///////////////////////////////////////////////////////////
  say(await agent.execute(token.address, { set_minters: { minters: [mgmt.address] } }))
  say(`${mgmt.address} can now tell ${token.address} to mint`)
  say(await agent.status())

  say('transferring ownership of token to mgmt...') ////////////////////////////////////////////////
  say(await agent.execute(token.address, { change_admin: { address: mgmt.address } }))
  say(`${mgmt.address} is now admin of ${token.address}`)
  say(await agent.status())

  say('setting schedule in mgmt...') ///////////////////////////////////////////////////////////////
  say(await (require('./config')({agent, mgmt, schedule})))
  say(await agent.status())

  say('ready to launch!') //////////////////////////////////////////////////////////////////////////
  say(await agent.status())
  return {
    agent,
    mgmt,
    token
  }
})
