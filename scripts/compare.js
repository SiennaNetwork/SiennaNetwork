#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # RE:SCL-07
//
// ## Background
// * At a certain point during the implementation it became clear that some of the amounts
//   budgeted by the specification do not divide evenly by the number of scheduled portions.
//   * This caused "remainder portions" to be approved as an addition to the scope.
//     Remainders are computed at runtime by the `Portion.claimable_by_at` function.
//   * The decision to reinstate an unused errors message that was to be removed
//     as per e.g. SCL-07 alerted us to a possible source of unexpected behavior
//     when claiming remainder portions (by way of several broken unit tests.)
//   * Had I followed through with editing those out, it might have opened a way
//     to claim portions allocated for other addresses within a `Channel`
//     (see Appendix A for description of object model).
//
// ## Response
// * The suspected behavior was ultimately deemed impossible to exploit (e.g. to claim
//   another recipient's portions). However, the hunt for it proved beneficial for Hack.bg's
//   overall capacity for working with SecretNetwork and related projects.
//   * Verifying the code in isolation from the intended environment provides little benefit.
//     In order to validate that the contract behaves correctly in a real-time production-like
//     environment, the following baroque integration test had to be added to the existing tests.
// * This script contains reference examples of operating the contract (configuring,
//   launching, and claiming). These are our first available samples of integrating the
//   contract into a larger system; it is reasonably trivial to make e.g. REST API method
//   bodies out of the following sections.
//
// ## Reflection
// * As you can see, the API complexity of SecretNetwork is not ideal. The overall architecture
//   of the APIs is sturdy enough, and I see no obvious faults that would make this project a no-go.
//   However, certain deficiencies inevitably had to be corrected in the process of constructing
//   this integration test and preparing the SIENNA contracts for primetime.
// * For example, error handling in SecretJS still hangs on a regex match...
//   * (something that has to be fixed on the server side)
//   * ...but at least now the API consumer doesn't need to do the same match again
//     to extract the error contents.
//   * The backend error schema should be mirrored in the client
// * SecretJS in general looks "optimized for the happy path"
//   * Versioned builds of SecretJS are released with broken tests - not healthy
//   * Overall lack of SecretJS documentation other than a handful of examples
//     that omit critical steps - such as creating an account (privkey/pubkey pair)
//     without the help of a browser extension (!).
//     * So once I started getting my distinct, ungoogleable error, I had to dive
//       into the source and sort out how error handling works
//     * Since it's _encrypted_ error handling, it's all tangled up with the
//       marshaling/unmarshaling code.
//       * Those things deserve to be on separate layers
//         to rule out developers not being able to see/act on error messages.
//         First steps were taken in that direction.
// * As the SecretNetwork development workflow revolves on downloading marginally verified
//   root binaries (Docker) and user-level source code (NPM), a further audit of the vendor
//   supply chain is recommended.
// * Various other omissions in the source code were identified and resolved,
//   thanks to the feedback from the CertiK report.
//   * Thanks for the syntactic idioms! Evidently my knowledge of Rust is imperfect,
//     but you helped to make it less so.
//   * Most glaringly, what we delivered for the audit didn't even compile.
//     (thanks to an outdated `Cargo.lock` that was and fixed in due course)
//     Why didn't you tell us?
//
// ## Next steps
// * Wrap the vesting contract into a HTTP API, where it can be hooked up
//   to the "big CLAIM button".
//   * So how is this going to work for users?
//   * I assume the AMM contracts funded by the LPF will have a way of claiming funds
//     that we might need to look at together with Asparuh to design an API with the
//     fewest moving parts.
//   * What about everyone else then? Do they have their own cron daemons,
//     or is there going to be a "big CLAIM button" of some sort - maybe as an add-on
//     to Keplr or similar? Or is our cron job going to end up triggering transfers
//     for all accounts?
// * Submit pull requests to SecretJS with various ways of making it less clunky.
//   * Coordinate with Enigma to make sure it lands at the right time
//   * Maybe we can help them document their JS API and publish the generated documentation
//   * Need to implement error schema on the JS end that corresponds to Rust error variants
//   * Important: interface to instantiate local async proxies for contracts from binary
//     and generated schema (it doesn't look like any of the JSON schema generated by the
//     Rust crates is taken into account by SecretJS, even though that would be the most
//     obvious place to use it)
//   * As you can see, plenty of easy pickings.
// * Allow early returns in Fadroma to reduce complexity of the contract methods (which are
//   actually just block expressions and as such can't contain early returns)
// * Incrementally merge `schedule v2.0.0-rc1` into `main`.
//   * It does not change the API of `mgmt`, so its behavior can be
//     integration/regression tested with this script.
//   * It completely rules out the Heisenbug that started it all.
//   * It reads configuration from a local spreadsheet
//     instead of depending on a Google Doc.
//   * So don't be scared by the major version, it's a collection of
//     incremental improvements with inessential backwards incompatibilities.
//     We're counting on you to be onboard with us for this one
//     (and please warn us if we deliver code that doesn't compile on the first try
//     or anything obvious like that... we won't hold it against you!)
//
// ## Demonstration
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
    //`poc`,         // * top of `poc` branch
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
  // * Wait for asynchronous dependencies to be ready:
  ;[ADMIN, ALICE, BOB] = await Promise.all([ADMIN, ALICE, BOB])
  // * This just prints the time, and address/balance of each initial account:
  await Promise.all([ADMIN,ALICE,BOB].map(x=>x.status()))
  // * Let's deploy several different instances of the codebase
  //   to see how they handle the error condition described below:
  const results = {}
  for (let commit of commits) await testCommit(commit, say.tag(`#testing{${commit}}`))
  async function testCommit (commit, say) {
    say(`-----------------------------------------------------------------------------------------`)
    try {
      // (This could've been done in parallel, but trying to
      // upload multiple contracts in 1 block crashes something?)
      // * Deploy the token
      const TOKEN = await SNIP20Contract.fromCommit({say, agent: ADMIN, commit})
      //await ADMIN.waitForNextBlock()
      // * Generate viewing keys
      const [vkALICE, vkBOB] = await Promise.all([
        TOKEN.createViewingKey(ALICE, ALICE.address),
        TOKEN.createViewingKey(BOB,   BOB.address),
      ])
      // * Deploy the vesting manager
      const MGMT = await MGMTContract.fromCommit({say, agent: ADMIN, commit, token: TOKEN}) 
      let schedule
      await Promise.all([
        await MGMT.acquire(TOKEN), // * Give it a token to issue
        await MGMT.configure(schedule = getSchedule({ ALICE, BOB })) // * Configure the vesting
      ])
      await MGMT.launch() // * Launch it (it should finish in a few seconds)
      say(`launched vesting ----------------------------------------------------------------------`)
      // ### Hocus Pocus
      // * Suppose `ALICE` and `BOB` are two swap contracts scheduled to
      //   receive funds from the Liquidity Provision Fund.
      //   * ALICE claims every portion.
      //   * BOB claims no portions.
      //   * Loop until vesting is over.
      while (true) {
        // * Wait for next portion to vest
        await ADMIN.waitForNextBlock()
        // * Claim portion
        say(`vesting portion ---------------------------------------------------------------------`)
        try {
          await MGMT.claim(ALICE)
          // * Confirm that it's been received
          await TOKEN.balance({ agent: ALICE, viewkey: vkALICE, address: ALICE.address })
        } catch (error) {
          // * Unless
          if (!error.log) {
            say(' #warning')('not the error we were expecting')
          } else {
            const log = JSON.parse(error.log)
            say.tag('#error.log')(log)
            if (log.generic_err && log.generic_err.msg === 'nothing for you') {
              say.tag('#MGMT')('vesting ended')
              break //   * The contract runs out of money
                    //     * With which vesting ends and we move forward onto remainders...
            } else {
              say(' #warning')('not the error we were expecting either, try again')
            }
          }
          console.log(error)
          break
        }
      }
      say(`vesting ended -------------------------------------------------------------------------`)
      // * Now ALICE should have 6 portions and BOB should have zero.
      await TOKEN.balance({ agent: ALICE, viewkey: vkALICE, address: ALICE.address })
      await TOKEN.balance({ agent: BOB,   viewkey: vkBOB,   address: BOB.address   })
      // * Because of validity constraint introduced in `rc2`,
      //   remainder can't be vested to multiple recipients.
      //   * This means it needs to be manually reallocated
      //   * Otherwise it's stuck in the contract
      //     * So yeah you can't really `Disown` the contract with the current schedule...
      //   * Admin allocates remainder to ALICE.
      say(`reallocate remainders -----------------------------------------------------------------`)
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
      say(`ok, next version ----------------------------------------------------------------------`)
      results[commit] = 'OK'
    } catch (e) {
      say.tag(' #error')(`------------------------------------------------------------------------`)
      console.error(e)
      results[commit] = e
      say(`next version? -------------------------------------------------------------------------`)
    }
  }
  console.log(results)
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
// * The following is a valid JSON schedule:
//   * an intermediate representation between the configuration spreadsheet
//     and the `schedule` module's actual in-memory model.
//   * a guide to the contract's execution model.
function getSchedule ({ ALICE, BOB }) {
  return {
    "total": "1000000000000",
    "pools": [ // * `Pool`s map to the first-level categories from the spec:
      {        //   Investors, Founders, Advisors,...
        "name": "Pool",           // * If the `Pool` is marked `partial` (as is the default),
        "total": "1000000000000", //   new `Channel`s can be added by the admin to it before or
        "partial": true,          //   after launch, up to the maximum pool amount.
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
