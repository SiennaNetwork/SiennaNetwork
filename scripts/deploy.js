#!/usr/bin/env node
process.on('unhandledRejection', up => {throw up})

async function deploy ({

  say    = require('./say')('[deploy]'),
  env    = require('./env')(),
  client = require('./client')(),

  version = `2021-03-02-80f6297`,

  token       = `${__dirname}/../dist/${version}-snip20-reference-impl.wasm`,
  tokenLabel  = `SIENNA SNIP20 (${new Date().toISOString()})`,
  tokenConfig = {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    admin:     client.address,
    prng_seed: "insecure",
    config:    { public_total_supply: true }
  },

  mgmt       = `${__dirname}/../dist/${version}-sienna-mgmt.wasm`,
  mgmtLabel  = `SIENNA MGMT (${new Date().toISOString()})`,
  mgmtConfig = {},

}={}) {
  client = await Promise.resolve(client)

  say('deploying token...') ////////////////////////////////////////////////////////////////////////
  token = await client.deploy(token, tokenLabel, tokenConfig)
  say(env.write('TOKEN', token))

  say('deploying mgmt...') /////////////////////////////////////////////////////////////////////////
  mgmt = await client.deploy(mgmt, mgmtLabel, {
    token_addr: token.address,
    token_hash: token.hash,
    ...mgmtConfig,
  })
  say(env.write('MGMT', mgmt))

  say('allowing mgmt to mint tokens...') ///////////////////////////////////////////////////////////
  say(await client.execute(token.address, { set_minters: { minters: [mgmt.address] } }))
  say(`${mgmt.address} can now tell ${token.address} to mint`)

  say('transferring ownership of token to mgmt...') ////////////////////////////////////////////////
  say(await client.execute(token.address, { change_admin: { address: mgmt.address } }))
  say(`${mgmt.address} is now admin of ${token.address}`)

  say('setting schedule in mgmt...') ///////////////////////////////////////////////////////////////
  say(await (require('./config')({client, mgmt})))

  say('ready to launch!') //////////////////////////////////////////////////////////////////////////
  return {
    client,
    mgmt,
    token
  }
}

module.exports=(require.main&&require.main!==module)?deploy:deploy()
