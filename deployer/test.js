#!/usr/bin/env node

module.exports = main
if (require.main === module) main()
async function main () {
  const { client, mgmt, token } = await require('./deploy')()
  console.log(await client.query(token.address, { "token_info": {} }))
  await client.execute(mgmt.address, { "launch": {} })
  console.log(await client.query(token.address, { "token_info": {} }))
}
