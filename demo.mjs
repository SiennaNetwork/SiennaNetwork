#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # SIENNA Vesting Contract Demo
// * [x] by using a local testnet container
// * [ ] that allows time to be fast-forwarded using `libfaketime`
// * this script demonstrates:
//   * [x] deploying and configuring the token and vesting contracts
//   * [x] making claims according to the initial schedule
//   * [x] checking unlocked funds without making a claim
//   * [x] splitting the Remaining Pool Tokens between multiple addresses
//   * [ ] reconfiguring the Remaining Pool Token split, preserving the total portion size
//   * [ ] adding new accounts to Advisor/Investor pools
import assert from 'assert'
import { fileURLToPath } from 'url'
import { resolve, dirname } from 'path'
import { say as sayer, loadJSON, SecretNetwork } from '@hackbg/fadroma'
import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

SecretNetwork.connect(import.meta.url, async ({chain, agent, builder})=>{
  console.log({chain,builder,agent})
  const schedule   = loadJSON('./settings/schedule.json', import.meta.url)
  const recipients = await prepare(chain, agent, schedule)
  const contracts  = await deploy(builder, schedule, recipients)
  const result     = await verify(agent, recipients, contracts, schedule) 
  say(result)
})

async function prepare (chain, agent, schedule) {
  const wallets    = []
      , recipients = {}
      , mutatePool = pool =>
          Promise.all(pool.accounts.map(mutateAccount))
      , mutateAccount = async account => {
          // * create an agent for each recipient address (used to test claims)
          const { name } = account
          const agent = await chain.getAgent(name) // create agent
          const { address } = agent
          account.address = address        // replace placeholder with real address
          wallets.push([address, 1000000]) // balance to cover gas costs
          recipients[name] = {agent}       // store agent
          // * divide all times in account by 86400, so that a day passes in a second
          account.start_at /= 86400
          account.interval /= 86400
          account.duration /= 86400 }
  for (let name of [ // extra accounts for reconfigurations
    'TokenPair1',
    'TokenPair2',
    'TokenPair3',
    'NewAdvisor',
    'NewInvestor1',
    'NewInvestor2',
  ]) {
    const agent = await chain.getAgent(name) // create agent
    wallets.push([agent.address, 1000000])
    recipients[name] = {agent}
  }
  await Promise.all(schedule.pools.map(mutatePool))
  // seed agent wallets so the network recognizes they exist
  await agent.sendMany(wallets, 'create recipient wallets')
  return recipients
}

async function deploy (builder, schedule, recipients) {
  const repo = dirname(fileURLToPath(import.meta.url))
  builder = builder.configure({
    buildImage: 'hackbg/secret-contract-optimizer:latest',
    buildUser:  'root',
    outputDir:  resolve(repo, 'build', 'output'),
    repo
  })
  console.log({builder})
  const contracts = {}
  contracts.Token =
    await builder.crate('snip20-reference-impl').deploy(SNIP20Contract, {
      name:      "Sienna",
      symbol:    "SIENNA",
      decimals:  18,
      admin:     builder.agent.address,
      prng_seed: "insecure",
      config:    { public_total_supply: true }
    })
  contracts.MGMT =
    await builder.crate('sienna-mgmt').deploy(MGMTContract, {
      token:     [contracts.Token.address, contracts.Token.hash],
      schedule
    })
  contracts.RPT =
    await builder.crate('sienna-rpt').deploy(RPTContract, {
      token:     [contracts.Token.address, contracts.Token.hash],
      mgmt:      [contracts.MGMT.address,  contracts.MGMT.hash],
      pool:      'MintingPool',
      account:   'RPT',
      config:    [ [recipients.TokenPair1.address, "2500000000000000000000"]]
    })
  return contracts
}

async function verify (agent, recipients, contracts, schedule) {
  // create viewing keys
  const vk = await Promise.all(Object.values(recipients).map(async recipient=>
    recipient.vk = await TOKEN.createViewingKey(recipient.agent, "entropy")
  ))
  // mgmt takes over token; TODO auto-acquire on init
  await MGMT.acquire(TOKEN)
  // update schedule to point at RPT contract
  schedule
    .pools.filter(x=>x.name==='MintingPool')[0]
    .accounts.filter(x=>x.name==='RPT')[0]
    .address = RPT.address
  // load updated schedule into contract
  await MGMT.configure(schedule)
  // launch the vesting
  const {logs:{launched}} = await MGMT.launch()
  while (true) {
    await ADMIN.waitForNextBlock()
    const elapsed = (+ new Date() / 1000) - launched
    say.tag("elapsed")(elapsed)
  }
}
