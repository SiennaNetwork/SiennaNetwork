#!/usr/bin/env node
// # RE:SCL-07
//
// ## Overview
//
// * Response: escalated to medium severity.
//   Reinstating one of the unused errors that you suggested removed in SCL-07 actually unravelled
//   a possible unexpected behavior when claiming portions, by way of breaking three unit tests.
//   The following does not constitute a way for an outsider party to subvert the contract,
//   however it is a way for the contract to act in violation of the intent of the specification,
//   and therefore of its purpose.
// * Reason: commit ... line ...
//   Fixed @ commit ... line ...
// * Resolution: A version with a simplified and hardened vesting algorithm is scheduled to
//   be released around the end of this week. This algorithm will protect participants by
//   invalidating schedules that would cancel past unclaimed vestings.
// * Embedded self-promotion: we're up all night building our in-house validation workflow.
//   If you're experiencing an influx of Rust contracts to review,
//   now you know who to reach out to.
async function poc ({
  // ## Dependencies
  // * Logger
  say = require('./say')('[poc]'), 
  // * System calls
  existsSync = require('fs').existsSync,
  spawnSync  = require('child_process').spawnSync,
  resolve    = require('path').resolve,
  // * Testnet wallet with enough balance to pay for gas
  ADMIN   = require('./agent').fromEnvironment(),
  // * Some empty wallets
  ALICE   = require('./agent').fromKeyPair("ALICE"),
  BOB     = require('./agent').fromKeyPair("BOB"),
  CHARLIE = require('./agent').fromKeyPair("CHARLIE"),
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
                , [ { "addr": ALICE.addr, "amount": "83000" }
                  , { "addr": BOB.addr,   "amount": "83000" } ] ] ] } ] } ] }
  // * The above schedule can be accidentally leaky.
  //   Here's how this is handled in three different
  //   versions of the codebase:
  for (const commit of commits) {
    say(`testing commit ${commit}`)
    const tokenBin = `dist/${commit}-snip20-reference-impl.wasm`
    const mgmtBin  = `dist/${commit}-sienna-mgmt.wasm`
    // * Compile the contract if it doesn't exist
    if (!existsSync(tokenBin) || !existsSync(mgmtBin)) {
      say(`building commit ${commit}`)
      const build = spawnSync(resolve(__dirname, 'build-git-commit.sh'), [ commit ], { stdio: 'inherit' })
      console.log(build)
    }
    // * Deploy an instance
    say(`deploying commit ${commit}`)
    const mgmt = await require('./deploy')({ agent: ADMIN, token: tokenBin, mgmt: mgmtBin })
    // * Launch the contract
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
      // * Suppose `ALICE` and `BOB` from the `Allocation` are two swap contracts that are
      //   scheduled to receive funds from the Liquidity Provision Fund.
      //
      // * Suppose the `ALICE` project has been going according to schedule, and the `ALICE` contract
      //   has been claiming funds correctly from day 1. The vesting schedule ends, there's 4000 left
      //   in the channel, and since remainders don't vest if there's more than 1 recipient in the
      //   current allocation (L...) the admin decides to let `CHARLIE` have the 4000 and sets the
      //   allocation accordingly:
      //
      // * Suppose that the `BOB` contract, unlike `ALICE`, only manages to claim a couple of
      //   `Portion`s before experiencing delays, but then gets back on track after the end of the
      //   vesting. `BOB` pings `ADMIN` to reinstate them as a claimant and makes a claim for their
      //   portion but... `CHARLIE` already got that because it was transferred with the `4000`.
    }
  }, 10000)

  // * This is why there is a `Disown` method: in the case that a final schedule is reached,
  //   and all parties are certain that no more amendments will be necessary,
  //   they have the option of leaving the contract immutable.
  //
  // * Allocations can be freely modified at runtime by the admin, allowing a malicious party who
  //   has seized control of the contract to freely reallocate unclaimed portions to themselves.
  //   *  In a Byzantine environment, even a formerly trusted recipient may refuse to remit an
  //      unexpected transfer.
  //   *  The upcoming version of the `schedule` crate will implement partial protection against a
  //      rogue admin by preventing portions in the past from being altered.
}

module.exports=(require.main&&require.main!==module)?poc:poc()
