#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # SIENNA Vesting Contract Demo
//
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
import { resolve } from 'path'

import { say as sayer, loadJSON, SecretNetwork } from '@hackbg/fadroma'
import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

const say = sayer.tag(() => new Date().toISOString()) // Timestamped logger
const MNE = x => loadJSON(`/shared-keys/${x}.json`).mnemonic // get mnemonic from file

Promise.all([
  SecretNetwork.Agent.fromMnemonic({ say, name: 'ADMIN', mnemonic: MNE('ADMIN') }),
  SecretNetwork.Agent.fromMnemonic({ say, name: 'ALICE', mnemonic: MNE('ALICE') }),
  SecretNetwork.Agent.fromMnemonic({ say, name: 'BOB',   mnemonic: MNE('BOB') })

]).then(async function prepare ([ADMIN, ALICE, BOB]) {
  const commit    = 'HEAD' // git ref
  const buildRoot = fileURLToPath(new URL('../build', import.meta.url))
  const outputDir = resolve(buildRoot, 'outputs')
  const builder   = new SecretNetwork.Builder({ say: say.tag('builder'), outputDir, agent })
  // ### Deploy the token, generate viewing keys
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
  const [vkALICE, vkBOB] = await Promise.all([
    TOKEN.createViewingKey(ALICE, "entropy"),
    TOKEN.createViewingKey(BOB,   "entropy"),
  ])
  // ### Deploy and launch the vesting manager
  const MGMT = await builder.deploy(MGMTContract, {
    token: [TOKEN.address, TOKEN.hash],
    schedule
  }, {
    name: 'sienna-mgmt',
    commit,
    say,
  })
  const schedule = getSchedule({ ALICE, BOB })
  await Promise.all([ MGMT.acquire(TOKEN), MGMT.configure(schedule) ])
  await MGMT.launch()
  return { ADMIN, ALICE, BOB, TOKEN, MGMT, schedule }

}).then(async function compare ({ ADMIN, ALICE, BOB, TOKEN, MGMT, schedule}) {
  say(`launched vesting ----------------------------------------------------------------------`)
  while (true) { // Repeat the claim until vesting has ended:
    await ADMIN.waitForNextBlock() // Wait for next portion to vest (in the schedule, the interval is configured to be ~= block time)
    try {
      say(`ALICE claims ------------------------------------------------------------------------`)
      await MGMT.claim(ALICE) // Claim a portion and check recipient balance to confirm it has been transferred
      await TOKEN.balance({
        agent: ALICE,
        viewkey: vkALICE,
        address: ALICE.address
      })
    } catch (error) { // If that fails:
      if (!error.log) {
        say.tag(' #warning')('not the error we were expecting')
      } else {
        const log = JSON.parse(error.log)
        say.tag(' #error')(log)
        const expectedErrors = [
          'nothing for you',
          'channel Channel1: remainders not supported alongside split allocations'
        ]
        if (log.generic_err && (expectedErrors.indexOf(log.generic_err.msg) > -1)) {
          say.tag('#MGMT')('vesting ended')
          break // That means the vesting has ended and we can move forward onto remainders
        } else {
          say.tag(' #warning')('not the error we were expecting either, try again')
        }
      }
      console.log(error)
      break
    }
  }
  say(`vesting ended -------------------------------------------------------------------------`)
  await TOKEN.balance(ALICE, vkALICE)
  await TOKEN.balance(BOB,   vkBOB)
  say(`reallocate remainders -----------------------------------------------------------------`)
  await MGMT.reallocate(
    schedule.pools[0].name,
    schedule.pools[0].channels[0].name,
    { [ALICE.address]: "1" })
  await ADMIN.waitForNextBlock()
  await MGMT.claim(ALICE) // ALICE claims remainder.
  await TOKEN.balance(ALICE, vkALICE)
  await MGMT.claim(BOB)
  await TOKEN.balance(BOB, vkBOB)
  await ADMIN.waitForNextBlock() // Pause for a block before trying with the next version
  say(`ok, next version ----------------------------------------------------------------------`)
  results[commit] = 'OK' // store success
})

function getSchedule ({ ALICE, BOB }) {
  return {
    "total": "1000000000000",
    "pools": [ // `Pool`s map to the first-level categories from the spec: Investors, Founders, Advisors,...
      {
        "name": "Pool1",
        "total": "1000000000000",
        "partial": true, // If the `Pool` is marked `partial` (as is the default), new `Channel`s can be added by the admin to it before or after launch, up to the `total` pool amount.
        "channels": [
          {
            "name": "Channel1", // `Channel`s correspond to individual budget items like Investor1, Founder2, Advisor3, as well as DevFund, Liquidity Provision Fund...
            "amount": "1000000000000",
            "periodic": { // Recipients need to actively claim from the channel to receive the funds that are `Periodic`ally unlocked. It is expected that this can happen just as easily daily or long after the vesting has ended; the contract is used for safekeeping.
              "type":     "channel_periodic", // simple periodic vesting configuration:
              "cliff":    "0", // no cliff;
              "start_at":  0,  // start right away;
              "interval":  5,  // about one portion per localnet block for the test's sake;
              "duration":  30, // is it immediately obvious whether there are 6 or 7 portions?
              "expected_portion": "166666666666", // Purely informational.
              "expected_remainder": "4"  // Also informational; it's necessary for the allocations to leave have some remainder in order to trigger the bug.
            },
            "allocations": [ // Channels also have `Allocation`s. They're address/amount pairs that implement the "liquidity provision fund" part of the spec by splitting the daily portion between multiple configured addresses (e.g. the SIENNA Swap AMM contracts).
              [ 0 , [ { "addr": ALICE.address, "amount": "100000000000" }
                    , { "addr": BOB.address,   "amount":  "66666666660" } ] ]
              // Calling `reallocate` on the contract adds a new record here, containing an updated timestamp and the new allocations.
            ]
          }
        ]
      }
    ]
  }
}
