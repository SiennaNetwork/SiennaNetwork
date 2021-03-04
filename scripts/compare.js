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
//
// * This caused "remainder portions" to be approved as an addition to the scope. Remainders
//   are computed at runtime by the `Portion.claimable_by_at` function,
//
// * Reinstating one of the unused errors that the CertiK report suggested removed as per SCL-07
//   tipped us off to a possible source of unexpected behavior in the contract when claiming
//   those remainder portions.
//
const ADMIN_KEY = require('/shared-keys/admin_key.json') // thank rootful containers for this gem

// ## Dependencies
require('./lib')(module, async function compare ({
  // * Logging:
  say = require('./lib/say'),
  // * Testnet wallet with enough balance to pay for gas
  //   TODO get this balance from localnet faucet/initial wallet:
  SecretNetworkAgent = require('./lib/agent'),
  ADMIN   = SecretNetworkAgent.fromMnemonic({ say, name: "ADMIN", mnemonic: ADMIN_KEY.mnemonic }),
  // * It also creates some empty wallets:
  ALICE   = SecretNetworkAgent.fromKeyPair({say, name: "ALICE"}),
  BOB     = SecretNetworkAgent.fromKeyPair({say, name: "BOB"}),
  CHARLIE = SecretNetworkAgent.fromKeyPair({say, name: "CHARLIE"}),
  // * A list of the commits to compare:
  commits = [ 
    //`1.0.0-rc1`, // * where `MGMT`'s `AddChannel` method is not implemented yet
    //`1.0.0-rc2`, // * as delivered for audit (fails to build)
    `1.0.0-rc3`, // * the above + fix to Cargo.lock to allow it to build
    `1.0.0-rc4`, // * the above + patches as per SCL-04/07/11
    `1.0.0-rc5`, // * the above, fixed.
  ],
  // * Interfaces to the contracts
  SNIP20Contract = require('./lib/contract').SNIP20Contract,
  MGMTContract   = require('./lib/contract').MGMTContract

}={}) {

  // ## Preparation

  // * Wait for asynchronous calls in arguments to finish:
  ;[ADMIN, ALICE, BOB, CHARLIE] = await Promise.all([ADMIN, ALICE, BOB, CHARLIE])
  // * Send 1uscrt to recipient addresses so that they exist
  for (let {address} of [ALICE, BOB, CHARLIE]) {
    ADMIN.say.tag(' #send')(await ADMIN.API.sendTokens(
      address, [{amount: "1000000", denom: "uscrt"}], "exist!"
    ))
  }
  // * This just prints the time and address and balance of the initial account.
  await ADMIN.status()
  // * Let's deploy several different instances of the codebase
  //   to see how they handle the error condition described below:
  const instances = {}
  //   * This could've been done in parallel, but trying to
  //     upload multiple contracts in 1 block crashes
  for (let commit of commits) {
    // * Deploy the token
    const TOKEN = await SNIP20Contract.fromCommit({agent: ADMIN, commit})
    // * Deploy the vesting manager
    const MGMT = await MGMTContract.fromCommit({agent: ADMIN, commit, token: TOKEN}) 
    // * Connect it to the token
    await MGMT.acquire(TOKEN)
    // * Configure it with a schedule corresponding to the situation described below
    await MGMT.configure(getSchedule({ ALICE, BOB, CHARLIE }))
    // * Launch it
    await MGMT.launch()
    // * It should finish in a few seconds and then it gets interesting.
    instances[commit] = { MGMT, TOKEN }
  }

  // ## Demonstration
  while (true) {
    await ADMIN.status()
    await ADMIN.waitForNextBlock()
    for (let commit of commits) {
      const {MGMT} = instances[commit]
      say(await MGMT.claim(ALICE))
      say(await MGMT.claim(BOB))
    }
  }

  process.exit(0)

  // ## Hocus pocus
  //
  // * So let's imagine something caused claims to the above channel
  //   to be delayed until after the vesting period had ended.
  //   * For example, suppose `ALICE` and `BOB` from above are two swap contracts that
  //     are scheduled to receive funds from the Liquidity Provision Fund.
  await setTimeout(async ()=>{
    await test(A)
    await test(B)
    await test(C)
    async function test (X) {
      //
      // * When the vesting schedule ends, there are all of BOB's unclaimed funds
      //   left the contract, as well as 4000 - the remainder from uneven division.
      //   * Since the possibility to split the remainders was not implemented until
      //     after the audit, a stop-gap measure was in place for `v1.0.0-rc2`, requiring
      //     the admin to `reallocate` the remainder to a single recipient, e.g. `CHARLIE`.
      //   * Due to an error in the vesting logic, that recipient will also claim
      //     `BOB`'s unclaimed portions.
    }
  }, 10000)

  // * The specification implied that an actor with admin access should be unable to
  //   withdraw the lump sum or alter the contract arbitrarily, and indeed the contract
  //   should serve as a guarantee that the vesting process is conducted correctly.
  //   * This is why there is a `Disown` method: in the case that a final schedule is reached,
  //     and all parties are certain that no more amendments will be necessary,
  //     they have the option of leaving the contract immutable.
  //   * Hence an `Disown`ed contract with the above vulnerability would be a ticking time bomb.
  // * Allocations can be freely modified at runtime by the admin, allowing a malicious party who
  //   has seized control of the contract to freely reallocate unclaimed portions to themselves.
  //   *  In a Byzantine environment, even a formerly trusted recipient may refuse to remit an
  //      unexpected transfer.
  //   *  The upcoming version of the `schedule` crate will implement partial protection against a
  //      rogue admin by preventing portions in the past from being altered.
})

// * The above demonstration is provided as motivation to review the upcoming `v2` of the
//  `schedule` crate, which:
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
    "total": "1000000",
    "pools": [ // * `Pool`s map to the first-level categories from the spec: 
      {        //   Investors, Founders, Advisors,... 
        "name": "Pool",     // * If the `Pool` is marked `partial` (as is the default)
        "total": "1000000", //   new `Channel`s can be added by the admin to it before or 
        "partial": true,    //   after launch, up to the maximum pool amount.
        "channels": [ // * `Channel`s correspond to individual budget items like Investor1,
          {           //   Founder2, Advisor3, as well as DevFund, Liquidity Provision Fund...
            "name": "Channel",   // * Recipients need to actively claim from the channel to receive
            "amount": "1000000", //   the funds that are `Periodic`ally unlocked. It is expected
            "periodic": {        //   that this can happen just as easily daily or long after the
              "type":            //   vesting has ended; the contract is used for safekeeping.
              "channel_periodic",
              "cliff":    "0",   // * simple periodic configuration: no cliff;
              "start_at":  0,    //   start right away;
              "interval":  5,    //   about one portion per localnet block for clarity;
              "duration":  30,   //   is it immediately obvious whether there are 6 or 7 portions?
              "expected_portion": "166000",  // * It's key to have some remainder (the amount not
              "expected_remainder": "4000"   //   dividing evenly by the duration) to trigger the bug.
            }, 
            "allocations": [ // * Channels also have `Allocation`s. They're address/amount pairs
              [              //   that implement the "liquidity provision fund" part of the spec by
                0,           //   splitting the daily portion between multiple configured addresses
                [            //   (e.g. the SIENNA Swap AMM contracts)
                  { "addr": ALICE.address, "amount": "100000" },
                  { "addr": BOB.address,   "amount":  "66000" }
                ]
              ] // * Calling `reallocate` on the contract adds a new record here,
            ]   //   with an updated timestamp and the updated allocations.
          }
        ]
      }
    ]
  }
}
