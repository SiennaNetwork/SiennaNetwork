#!/usr/bin/env node
require('./lib')(module, async function deploy ({
  agent   = require('./lib/agent').fromMnemonic(/* process.env.MNEMONIC */),
  say     = agent.say.tag(' | deploy.js'),

  version, //

  SNIP20Contract = require('./lib/contract').SNIP20Contract,
  tokenBinary = `${__dirname}/../dist/${version}-snip20-reference-impl.wasm`,
  tokenLabel  = `SIENNA SNIP20 (${new Date().toISOString()})`,
  tokenConfig = {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    admin:     agent.address,
    prng_seed: "insecure",
    config:    { public_total_supply: true }
  },

  MGMTContract = require('./lib/contract').SNIP20Contract,
  mgmtBinary = `${__dirname}/../dist/${version}-sienna-mgmt.wasm`,
  mgmtLabel  = `SIENNA MGMT (${new Date().toISOString()})`,
  mgmtConfig = {},

  schedule = require('../config.json'),

}={}) {
  agent = await Promise.resolve(agent)
  say('deploying token...---------------------------------')
  const SNIP20 = await SNIP20Contract.deploy({
    agent, binary: tokenBinary, label: tokenLabel, initData: tokenConfig
  })
  say('deploying mgmt...----------------------------------')
  const MGMT = await MGMTContract.deploy({
    agent, binary: mgmtBinary, label: mgmtLabel, initData: {
      token_addr: token.address,
      token_hash: token.hash,
      ...mgmtConfig,
    },
  })
  say('allowing mgmt to mint tokens...--------------------')
  say(await agent.execute(token.address, { set_minters: { minters: [mgmt.address] } }))
  say(`${mgmt.address} can now tell ${token.address} to mint`)
  say('transferring ownership of token to mgmt...---------')
  say(await agent.execute(token.address, { change_admin: { address: mgmt.address } }))
  say(`${mgmt.address} is now admin of ${token.address}`)
  say('setting schedule in mgmt...------------------------')
  say(await (require('./config')({agent, mgmt, schedule})))
  say('ready to launch!-----------------------------------')
  return {
    agent,
    mgmt,
    token
  }
})
