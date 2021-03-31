#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # SIENNA Progress Report
//
// **2021-03-19, Hack.bg;** Although a little more extensive than originally expected,
// this latest rework of the vesting codebase doubles down on our commitment to ensuring
// verifiably correct behavior of all produced software artifacts.
//
// * In response to efficiency concerns, the vesting logic that is defined
//   in the `schedule` library and accessed via the `mgmt` contract does
//   not materialize individual portions anymore.
//   * As originally envisioned, the `schedule` logic does not anymore return a list of
//     portions to be individually handled, but simply calculates the unlocked amount for
//     a certain address at a specified point in time, and stores how much each address has
//     received.
//   * Thanks to this, the data model implemented in `schedule` was simplified considerably.
//     The former stack of, roughly speaking,
//     `Schedule(Pool(Channel(Periodic{..},Vec<(Seconds,Vec<Allocation>))`
//     has now become simply `Schedule(Pool(Account{..}))`. The two alternate vesting modes
//     (immediate and periodic) are now described in terms of the same set of fields
//     on the `Account` struct.
//     * In accordance with the above, as well as with the intention of the project,
//       the possibilty to arbitrarily reconfigure the vesting schedule at runtime
//       was dropped.
//     * The `AddAccount` remains an append-only way to add new recipients after launch
//     * The `rpt` contract takes over the responsibilities of `Allocation`s.
//
// * Separation of concerns motivated the special case of the
//   "remaining pool tokens" (RPT) to be moved to a separate contract.
//   * This is the same piece of logic that was suspected vulnerable in the previous report,
//     prompted the development of an integration testing workflow, and was eventually
//     identified as effectively non-functional.
//   * Splitting an account into multple portions over time (as performed by
//     the `mgmt` contract) was deemed orthogonal to splitting every portion
//     between multiple recipients (as performed by the new `rpt`) contract.
//   * Separating these two facets of the vesting implementation aims to
//     reduce the risk of invalid behaviors slipping under the radar;
//
// As the old adage goes, every piece of software should be designed to "do one thing
// and do it well". Besides modularizing the moving parts of the implementation, this
// approach allows us to establish boundaries of what inter-contract communication on
// Cosmos-based networks can achieve.
//
import assert from 'assert'
import { fileURLToPath } from 'url'
import { resolve } from 'path'

import { say as sayer, loadJSON, SecretNetwork } from '@hackbg/fadroma'
import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

const say = sayer.tag(() => new Date().toISOString()) // Timestamped logger

SecretNetwork.Agent.fromMnemonic({
  say:      say.tag('agent'),
  name:     "agent",
  mnemonic: loadJSON(`../integration/localnet/keys/ADMIN.json`, import.meta.url).mnemonic

}).then(async function prepare (agent) {
  const wallets    = []
  const recipients = {}
  const schedule   = loadJSON('../settings/schedule.json', import.meta.url)
  // create a wallet for each address in the schedule
  for await (const pool of schedule.pools) {
    for await (const account of pool.accounts) {
      const agent = await SecretNetwork.Agent.fromKeyPair({say, name: account.name})
      account.address = agent.address // replace placeholder address with test address
      wallets.push([agent.address, 1000000]) // balance to cover gas costs
      recipients[account.name] = {agent} // store agent
    }
  }
  await agent.sendMany(wallets, 'create recipient wallets')
  return { agent, recipients, schedule }

}).then(async function deploy ({ agent, recipients, schedule }) {
  const commit    = 'HEAD' // git ref
  const buildRoot = fileURLToPath(new URL('../build', import.meta.url))
  const outputDir = resolve(buildRoot, 'outputs')
  const builder   = new SecretNetwork.Builder({ say: say.tag('builder'), outputDir, agent })

  const TOKEN = await builder.deploy(SNIP20Contract, {
    name:      "Sienna",
    symbol:    "SIENNA",
    decimals:  18,
    admin:     agent.address,
    prng_seed: "insecure",
    config:    { public_total_supply: true }
  }, {
    name: 'snip20-reference-impl',
    commit,
    say,
  })

  for (const [name, {agent}] of Object.entries(recipients)) {
    recipients[name].viewkey = await TOKEN.createViewingKey(agent, "entropy")
  }

  const MGMT = await builder.deploy(MGMTContract, {
    token_addr: TOKEN.address,
    token_hash: TOKEN.hash,
    schedule
  }, {
    name: 'sienna-mgmt',
    commit,
    say,
  })

  schedule
    .pools.filter(x=>x.name==='MintingPool')[0]
    .accounts.filter(x=>x.name==='RPT')[0]
    .address = RPT.address

  await MGMT.configure(schedule)

  return { agent, TOKEN, MGMT, RPT, recipients }

}).then(async function test ({ agent, recipients, TOKEN, MGMT, RPT }) {

  const fastForward = t => { /* TODO: Kill localnet and restart it with `libfaketime` set to `t` */ }
  await MGMT.acquire(TOKEN)
  await MGMT.launch()
  try {
    for (const [name, agent] of Object.entries(recipients)) {
      say.tag('progress')({ name, ...(await MGMT.progress(agent.address)) })
    }
    //await RPT.vest()
  } finally {
    //for (const [name, {agent, viewkey}] of Object.entries(recipients)) {
      //await TOKEN.balance(agent, viewkey)
    //}
    //// admin:
    //const viewkey = await TOKEN.createViewingKey(agent, "entropy")
    //await TOKEN.balance(agent, viewkey)
  }

})
