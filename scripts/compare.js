#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */

// # RE:SCL-07
//
// ## Summary
//
// * Response: escalated to medium severity.
//
// * Reason: commit ... line ...
//   Fixed @ commit ... line ...
// 
// * Description:
//   Reinstating one of the unused errors that you suggested removed in SCL-07 actually unravelled
//   a possible unexpected behavior when claiming portions, by way of breaking three unit tests.
//
//   It was long my doubt that there's some uncertainty around the logic delivered particularly
//   around remainders, which were approved as an addition to the implementation after it became
//   clear that some ammounts do not divide evenly by the number of portions, and are computed
//   at runtime by the `Portion.claimable_by_at`.
//
//   The following does not constitute a way for an outsider party to subvert the contract,
//   (it requires an actor with admin access to load an erroneous config to trigger the bug)
//   however it is a way for the contract to act in violation of the intent of the specification,
//   and therefore of its purpose.
//
// * Embedded self-promotion: we're up all night building our in-house validation workflow.
//   If you're experiencing an influx of Rust contracts to review, now you know who to reach out to.
//
require('./lib')(module, async function compare ({
  // ## Dependencies
  // * Logger
  say = require('./lib/say')('[compare]'), 
  // * System calls
  existsSync = require('fs').existsSync,
  spawnSync  = require('child_process').spawnSync,
  resolve    = require('path').resolve,
  // * Testnet wallet with enough balance to pay for gas
  ADMIN   = require('./lib/agent').fromEnvironment(),
  // * Some empty wallets
  ALICE   = require('./lib/agent').fromKeyPair("ALICE"),
  BOB     = require('./lib/agent').fromKeyPair("BOB"),
  CHARLIE = require('./lib/agent').fromKeyPair("CHARLIE"),
  // * Commits to compare
  commits = [ 
    //`9462051`, // * `A` = `1.0.0-rc2` as delivered for audit SCL-07
    `9f52d86`, // * `B` = `1.0.0-rc2` patched as per SCL-07
    `9cbf968`, // * `C` = `1.0.0-rc2` fixed
  ]
}={}) {
  // (wait for all asynchronous calls in arguments to finish)
  ;[ADMIN, ALICE, BOB, CHARLIE] = await Promise.all([ADMIN, ALICE, BOB, CHARLIE])
  // ## PoC
  // * The following is a valid JSON schedule - an intermediate representation between
  //   the configuration spreadsheet and the `schedule` module's actual in-memory model.
  const schedule =
    { "total": "1000000", "pools": // * `Pool`s map to the first-level categories from the spec: 
                                   //   Investors, Founders, Advisors,... 
      [ { "name": "Pool", "total": "1000000",
          "partial": true // * If the `Pool` is marked `partial` (as is the default)
                          //   new `Channel`s can be added by the admin to it before or 
                          //   after launch, up to the maximum pool amount.
        , "channels": // * `Channel`s correspond to individual budget items like Investor1,
                      //   Founder2, Advisor3, as well as DevFund, Liquidity Provision Fund...
          [ { "name": "Channel", "amount": "1000000"
            , "periodic": // * The `Periodic` subroutines generate the `Portion`s that recipients
                          //   can claim. Recipients need to actively make a claim to receive
                          //   the unlocked funds, and it is expected for recipients to be able to
                          //   claim the total of their unlocked `Portion`s daily just as easily as
                          //   claiming them long after the vesting has ended.
              { "type":     "channel_periodic"
              , "cliff":    "0" // * simple periodic configuration: no cliff;
              , "start_at":  0  //   start right away;
              , "interval": "1" //   one portion every second;
              , "duration": "6" //   is it immediately obvious whether there are 6 or 7 portions?
              , "amount":   "1000000"
              , "expected_portion":   "166000" // * some remainder because the amount 
              , "expected_remainder": "4000" } //   doesn't divide evenly by the duration.
            , "allocations": // * Channels also have `Allocation`s. They're address/amount pairs
                             //   that implement the "liquidity provision fund" part of the spec by
                             //   splitting the daily portion between multiple configured addresses
              [ [ 0          //   (e.g. the SIENNA Swap AMM contracts)
                , [ { "addr": ALICE.addr,   "amount": "100000" }
                  , { "addr": BOB.addr,     "amount":  "33000" }
                  , { "addr": CHARLIE.addr, "amount":  "33000" } ] ] ] } ] } ] }
  // * The above schedule can be accidentally leaky.
  //   Let's instantiate several different versions
  //   of the codebase to see how this is handled.
  for (let commit of commits) {
    // * This could've been just `await Promise.all(commits.map(prepare))`
    //   but trying to deploy multiple contracts in 1 block crashes
    await prepare(commit)
  }
  async function prepare (commit) {
    const tokenBin = resolve(__dirname, `../dist/${commit}-snip20-reference-impl.wasm`)
    const mgmtBin  = resolve(__dirname, `../dist/${commit}-sienna-mgmt.wasm`)
    // * Compile the contract pair if either binary is absent
    if (!existsSync(tokenBin) || !existsSync(mgmtBin)) {
      say(`building commit ${commit}`)
      const build = spawnSync(resolve(__dirname, 'build/commit.sh'), [ commit ], { stdio: 'inherit' })
      console.log(build)
    }
    // * Deploy an instance of the vesting
    say(`deploying commit ${commit}`)
    const {mgmt, token} = await require('./deploy')({ agent: ADMIN, token: tokenBin, mgmt: mgmtBin })
    // * Check everyone's token balance
    const key = await require('./balance').createViewingKey()
    await Promise.all(Object.entries({
      ADMIN, mgmt, token,
      ALICE, BOB, CHARLIE,
    }).map(async ([name, wallet])=>{
      say(wallet.name)
      await require('./balance').getBalance({
        agent:ADMIN,
        token,
        address:wallet.address
      })
    }))
    // * Launch the vesting
    say(`launching commit ${commit}`)
    await require('./launch')({agent: ADMIN, mgmt})
  }

  // * Let's imagine something caused claims to the above channel to be delayed
  //   until after the vesting period had ended.
  console.log('waiting for investments to mature...')
  while (true) {
    say(await ADMIN.status())
  }
  await setTimeout(async ()=>{
    await test(A)
    await test(B)
    await test(C)
    async function test (X) {
      // * Suppose `ALICE` and `BOB` from above are two swap contracts that are scheduled to
      //   receive funds from the Liquidity Provision Fund.
      //
      // * Suppose the `ALICE` project has been going according to schedule, and the `ALICE`
      //   contract has been claiming funds correctly from day 1. `BOB`, on the other hand,
      //   only manages to claim a couple of `Portion`s before experiencing some sort of delay
      //   and letting its allocated funds accumulate unclaimed in the vesting contract.
      //
      // * Then the vesting schedule ends, and there's 4000 + BOB's unclaimed funds left
      //   in the channel. Since remainders don't vest if there's more than 1 recipient
      //   in the current Allocations (L..-..), neither ALICE nor BOB can claim those funds.
      //
      // * After the vesting is over, ADMIN decides to update the allocations so that the 4000
      //   go to CHARLIE. Some time later BOB catches up and wants to claim all vested funds in
      //   bulk. But if CHARLIE claims the remainder before that, CHARLIE will also get BOB's
      //   allocations also got those, because the contract 
      //   in the channel, and since remainders don't vest if there's more than 1 recipient in the
      //   current allocation (L...) the admin decides to let `CHARLIE` have the 4000 and sets the
      //   allocation accordingly:
      //
      // *  but then gets back on track after the end of the
      //   vesting. `BOB` pings `ADMIN` to reinstate them as a claimant and makes a claim for their
      //   portion but... `CHARLIE` already got that because it was transferred with the `4000`.
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
