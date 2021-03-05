#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
//
// # RE:SCL-07
//
// ## Status
//
// * Response: escalated to medium severity.
//
// * Reason: commit ... line ...
//   Fixed @ commit ... line ...
//
// ## Summary
//
// * At a certain point during the implementation it became clear that some of the amounts
//   budgeted by the specification do not divide evenly by the number of scheduled portions.
//   * This caused "remainder portions" to be approved as an addition to the scope.
//     Remainders are computed at runtime by the `Portion.claimable_by_at` function.
//   * Reinstating one of the unused errors that the CertiK report suggested removed as per SCL-07
//     tipped us off to a possible source of unexpected behavior in the contract when claiming
//     those remainder portions.
//
// * The behavior was ultimately deemed impossible to exploit. However, the search for it
//   led us to give the SIENNA smart contract system its first good test run in a real
//   production-like environment (a single Secret Network test node, which you can run
//   locally in a Docker container).
//   * What the vulnerability search did expose was a critical lack of available tooling
//     and documentation by our platform vendors (Secret Network, Cosmos Network).
//   * A significant step towards resolving this lack was undertaken during the search:
//     trying to demonstrate erroneous behavior step-by-step in <100LOC produced a
//     concise interface for addressing smart contracts with known schemas from a
//     familiar JavaScript environment.
//
// * Various other omissions in the source code were identified and resolved,
//   thanks to the feedback from the CertiK report.
//   * Most glaringly, what we delivered for the audit
//     didn't even compile. (Why didn't you tell us?)
//   * This was identified to be simply the result of
//     an outdated `Cargo.lock` file and fixed in due course.
//
// ## Demonstration
//
// ### Dependencies
require('./lib')(module, async function compare ({
  // * Logging:
  say = require('./lib/say'),
  // * Pre-existing testnet wallets with enough balance to pay for gas:
  SecretNetworkAgent = require('./lib/agent'),
  MNE   = x => require(`/shared-keys/${x}.json`).mnemonic, // (mnemonic getter)
  ADMIN = SecretNetworkAgent.fromMnemonic({say, name: "ADMIN", mnemonic: MNE("ADMIN")}),
  ALICE = SecretNetworkAgent.fromMnemonic({say, name: "ALICE", mnemonic: MNE("ALICE")}),
  BOB   = SecretNetworkAgent.fromMnemonic({say, name: "BOB",   mnemonic: MNE("BOB")  }),
  // * A list of the commits to compare:
  commits = [ 
    //`1.0.0-rc1`, // * before implementing `AddChannel`
    //`1.0.0-rc2`, // * as delivered for audit (fails to build)
    `1.0.0-rc3`,   // * the above + fix to Cargo.lock to allow it to build
    //`1.0.0-rc4`, // * the above + patches SCL{01..13} + MGL{01..02} (fails test suite at SCL-04)
    `1.0.0-rc5`,   // * the above + revert SCL-04
  ],
  // * Interfaces to the contracts:
  SNIP20Contract = require('./lib/contract').SNIP20Contract,
  MGMTContract   = require('./lib/contract').MGMTContract
}={}) {

  // ### Preparation

  // * Wait for asynchronous calls in arguments to finish:
  ;[ADMIN, ALICE, BOB] = await Promise.all([ADMIN, ALICE, BOB])
  // * This just prints the time and address and balance of each initial account.
  await Promise.all([ADMIN,ALICE,BOB].map(x=>x.status()))
  // * Let's deploy several different instances of the codebase
  //   to see how they handle the error condition described below:
  for (let commit of commits) {
    // * This could've been done in parallel, but trying to
    //   upload multiple contracts in 1 block crashes something
    // * Deploy the token
    const TOKEN = await SNIP20Contract.fromCommit({agent: ADMIN, commit})
    //await ADMIN.waitForNextBlock()
    // * Generate viewing keys
    const [vkALICE, vkBOB] = await Promise.all([
      TOKEN.createViewingKey(ALICE, ALICE.address),
      TOKEN.createViewingKey(BOB,   BOB.address),
    ])
    // * Deploy the vesting manager
    const MGMT = await MGMTContract.fromCommit({agent: ADMIN, commit, token: TOKEN}) 
    let schedule
    await Promise.all([
      // * Connect it to the token
      await MGMT.acquire(TOKEN),
      // * Configure it with a schedule corresponding to the situation described below
      await MGMT.configure(schedule = getSchedule({ ALICE, BOB }))
    ])
    // * Launch it
    await MGMT.launch()
    // * It should finish in a few seconds and then it gets interesting.

    // ### Hocus Pocus
    // * Suppose `ALICE` and `BOB` are two swap contracts scheduled to
    //   receive funds from the Liquidity Provision Fund.
    //   * ALICE claims every portion. BOB claims no portions.
    while (true) {
      await ADMIN.waitForNextBlock()
      const result = await MGMT.claim(ALICE)
      await TOKEN.balance({ agent: ALICE, viewkey: vkALICE, address: ALICE.address })
      await TOKEN.balance({ agent: BOB,   viewkey: vkBOB,   address: BOB.address   })
      if (result.error) {
        const {generic_err:{msg}} = JSON.parse(result.log)
        if (msg === "nothing for you") break // * Vesting ends.
        else throw new Error(JSON.stringify(result))
      }
    }
    // * Remainder can't be vested to multiple recipients
    //   because of validity constraint introduced in `rc2`.
    //   * Admin allocates remainder to ALICE.
    await MGMT.reallocate(
      schedule.pools[0].name,
      schedule.pools[0].channels[0].name,
      [ { addr:   ALICE.address
        , amount: schedule.pools[0].channels[0].periodic.expected_remainder } ])
    await ADMIN.waitForNextBlock()
    //   * ALICE claims remainder.
    await MGMT.claim(ALICE)
    await TOKEN.balance({ agent: ALICE, viewkey: vkALICE, address: ALICE.address })
    //   * **OOPS!** ALICE has received BOB's unclaimed portions.
    await MGMT.claim(BOB) // Sorry Bob.
    await TOKEN.balance({ agent: BOB, viewkey: vkBOB, address: BOB.address })

    // * Pause for a block before trying with the next version
    await ADMIN.waitForNextBlock()
  }

})

// ## Conclusions
//
// We'd like you to review our upcoming `1.0.0-rc2` of the `schedule` crate, which:
//  * features a simplified and hardened object model which does not allow for this
//    class of erroneous representation of the expected logic to be expressed
//  * allows for portions to be computed in advance from the command line,
//    as well as on-chain, allowing for the vesting schedule to be reviewed manually
//    (as will be required from SIENNA before launching the contract
//    and any run-time reconfigurations to it).
//  * prevents past unclaimed vestings from being "eaten" by an account introduced by
//    a run-time reconfiguration

// ## Appendix A - Schedule
//
// * The following is a valid JSON schedule - an intermediate representation between
//   the configuration spreadsheet and the `schedule` module's actual in-memory model.
function getSchedule ({ ALICE, BOB }) {
  return {
    "total": "1000000000000",
    "pools": [ // * `Pool`s map to the first-level categories from the spec: 
      {        //   Investors, Founders, Advisors,... 
        "name": "Pool",     // * If the `Pool` is marked `partial` (as is the default)
        "total": "1000000000000", //   new `Channel`s can be added by the admin to it before or 
        "partial": true,    //   after launch, up to the maximum pool amount.
        "channels": [ // * `Channel`s correspond to individual budget items like Investor1,
          {           //   Founder2, Advisor3, as well as DevFund, Liquidity Provision Fund...
            "name": "Channel",   // * Recipients need to actively claim from the channel to receive
            "amount": "1000000000000", //   the funds that are `Periodic`ally unlocked. It is expected
            "periodic": {        //   that this can happen just as easily daily or long after the
              "type":            //   vesting has ended; the contract is used for safekeeping.
              "channel_periodic",
              "cliff":    "0",   // * simple periodic configuration: no cliff;
              "start_at":  0,    //   start right away;
              "interval":  5,    //   about one portion per localnet block for clarity;
              "duration":  30,   //   is it immediately obvious whether there are 6 or 7 portions?
              "expected_portion": "166000000000",  // * It's key to have some remainder (the amount not
              "expected_remainder": "4000000000"   //   dividing evenly by the duration) to trigger the bug.
            }, 
            "allocations": [ // * Channels also have `Allocation`s. They're address/amount pairs
              [              //   that implement the "liquidity provision fund" part of the spec by
                0,           //   splitting the daily portion between multiple configured addresses
                [            //   (e.g. the SIENNA Swap AMM contracts)
                  { "addr": ALICE.address, "amount": "100000000000" },
                  { "addr": BOB.address,   "amount":  "66000000000" }
                ]
              ] // * Calling `reallocate` on the contract adds a new record here,
            ]   //   with an updated timestamp and the updated allocations.
          }
        ]
      }
    ]
  }
}
