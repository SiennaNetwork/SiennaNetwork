#!/usr/bin/env node

module.exports = main
if (require.main === module) main()
async function main () {
  const { client, mgmt, token } = await require('./deploy')()
  console.log('token info before launch:')
  console.log(await client.query(token.address, { "token_info": {} }))
  console.log('token minters before launch:')
  console.log(await client.query(token.address, { "minters": {} }))
  console.log('letting mgmt contract mint tokens...')
  console.log(await client.execute(token.address, { "set_minters": { "minters": [ mgmt.address ] } }))
  console.log('updated token minters before launch:')
  console.log(await client.query(token.address, { "minters": {} }))
  console.log('launching...')
  await client.execute(mgmt.address, { "launch": {} })
  console.log('token info after launch:')
  console.log(await client.query(token.address, { "token_info": {} }))
}
