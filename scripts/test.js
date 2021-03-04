#!/usr/bin/env node

require('./lib')(module, async function main () {
  const { agent, mgmt, token } = await require('./deploy')()
  console.log('token info before launch:')
  console.log(await agent.query(token.address, { "token_info": {} }))
  console.log('token minters before launch:')
  console.log(await agent.query(token.address, { "minters": {} }))
  console.log('letting mgmt contract mint tokens...')
  console.log(await agent.execute(token.address, { "set_minters": { "minters": [ mgmt.address ] } }))
  console.log('updated token minters before launch:')
  console.log(await agent.query(token.address, { "minters": {} }))
  console.log('launching...')
  await agent.execute(mgmt.address, { "launch": {} })
  console.log('token info after launch:')
  console.log(await agent.query(token.address, { "token_info": {} }))
})
