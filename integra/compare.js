#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # SIENNA Post-Audit Report
//
// **2021-03-09, Hack.bg;** CertiK's audit prompted us to conduct further research into ensuring correct behavior
// of smart contracts on the Secret Network. One suggestion, `SCL-07`, stood out as
// controversial, and lead us to suspect it would introduce a bug in the contract.
//
// What follows are notes from an exploration that started with investigating a few broken unit
// tests and resulted in a thorough overview of platform tooling, several candidate bug fixes (in
// both our code and that of the platform), and an integration test that demonstrates contract
// operation in real time.
//
// ## Background
// The object model of the `schedule` library, based on an overview of the schedule,
// and used by the `mgmt` contract, is described in [Appendix A](#appendix-a---schedule).
// Two of its features require attention:
// * **Allocations** allow a `Portion` to be split between multiple recipient addresses.
//   * Introduced early on as the underlying primitive that implements the
//     liquidity provision fund. Value updated at runtime by the admin by calling `reallocate`.
//   * Full history of allocations must be saved in contract storage to prevent "time travel"
//     (updates to the schedule that would result in negative vesting balances)
// * **Remainders** were approved as an addition to the scope when it became evident
//   that some of the amounts budgeted by the specification do not divide evenly by
//   the number of scheduled portions.
//   * In `rc2`, remainders are computed at runtime (by the `Periodic.claimable_by_at` function).
//
// ## Summary
// Test cases broken by applying the suggested fixes to `SCL-04` and `SCL-07`
// alerted us to a possible source of **unexpected behavior** at the intersection
// of allocation and remainder logic.
//
// My inital suspicion was that, had a developer proceeded uncritically and followed through with
// editing out the failing unit tests, it might have opened a way to claim portions allocated
// for other addresses within a `Channel` (see [Appendix A](#appendix-a---schedule) for
// description of object model) during the vesting of remainder portions.
//
// ## Measures taken
// To verify whether we have a bug on our hands, **this integration test** was implemented.
// This afforded us the visibility to ultimately deem the suspected erroneous behavior
// **impossible to exploit**.
// * However, according to the output of this test, remainders are impossible to vest at all,
//   even after the required manual reallocation, **potentially leaving funds locked in
//   the contract**.
//   * This suggests an underlying issue in determining the active set of `Allocation`s
//     for the channel, because using the wrong set of allocations would fail silently;
//     indeed, an error is returned only when trying to `claim` after the reallocation
//   * Further validation is planned to proceed with an upgraded version of the scheduling logic,
//     which does not exhibit the same constraints, and therefore **rules out the particular
//     problematic edge cases** explored in the following demonstration.
// * At this stage, verifying the code in isolation from the intended runtime environment
//   provides insufficient benefit. This is why this script puts contract binaries through
//   their paces on a **local testnet**.
//   * Currently, it consists of a single Secret Network node running in a Docker container.
//     In the future, larger test networks may be spawned to simulate consensus failure, etc.
// * This script **compares actual on-chain behavior** (no mocks!) of **specific commits**.
//   In ~120 lines of code, it demonstrates operating the vesting contract (configuring, launching,
//   and claiming) and its associated token (minting, transfers, balance check).
//   * In the future, this may be extended to interactions with other contracts from
//     the project.
//
// # Demonstration
// * If you read this document, you'll become updated about the status of the project
//   like never before
//   * Don't miss out on the [findings and conclusions](#findings-and-conclusions) section below.
// * To run this script, and see the contract working:
//   * Go to the root of the project repository
//   * Start the backend with `docker-compose up -d localnet`
//   * Build each commits to test, e.g. `./build/commit.sh 1.0.0-rc2`
//   * Run the test with `docker-compose run compare`
//   * Wait. The selected commits will be tested in sequence, at a rate of 5 seconds per block,
//     which adds up to **~1.5min per test run**.
compare().then(console.log) // When the testing ends, this will print `ok` or error for each version.
// ## Dependencies
async function compare ({
  say = require('./lib/say').tag(`${new Date().toISOString()} `), // * Logging
  SecretNetworkAgent = require('./lib/agent'), // * Pre-existing testnet wallets with gas money
  MNE   = x => require(`/shared-keys/${x}.json`).mnemonic, // * Gets mnemonics from environment
  ADMIN = SecretNetworkAgent.fromMnemonic({say, name: "ADMIN", mnemonic: MNE("ADMIN")}),
  ALICE = SecretNetworkAgent.fromMnemonic({say, name: "ALICE", mnemonic: MNE("ALICE")}),
  BOB   = SecretNetworkAgent.fromMnemonic({say, name: "BOB",   mnemonic: MNE("BOB")  }),
  commits = [  // list of Git refs to compare. (Tags/branches work fine.) However, you'll need to compile these yourself, with e.g. `./scripts/build/commit 1.0.0-rc3` before you run the script, because calling Docker from Docker is messy.
    `main`,        // * **main**: top of main branch
    //`1.0.0-rc1`, // * **rc1**: before implementing `AddChannel`
    //`1.0.0-rc2`, // * **rc2**: as delivered for preliminary audit (fails to build)
    `1.0.0-rc3`,   // * **rc3**: the above + fix to `Cargo.lock` to allow it to build
    //`1.0.0-rc4`, // * **rc4**: the above + patches SCL{01..13} + MGL{01..02} (fails test suite at SCL-04)
    `1.0.0-rc5`,   // * **rc5**: the above + revert SCL-04
  ],
  SNIP20Contract = require('./lib/contract').SNIP20Contract, // This wrapper lets us command the token
  MGMTContract   = require('./lib/contract').MGMTContract    // and this one the vesting.
}={}) {
  ;[ADMIN, ALICE, BOB] = await Promise.all([ADMIN, ALICE, BOB]) // Wait for async dependencies to be ready.
  await Promise.all([ADMIN,ALICE,BOB].map(x=>x.status())) // Print the time, and address/balance of each account.
  // Let's go!
  // ## Preparation
  // Let's deploy several different instances of the codebase, in order to see how they
  // respond to the test. Tests will execute in sequence (because trying to upload multiple
  // contracts in 1 block crashes; the rest should be more amenable to parallelization).
  // Results will be stored in `results`, and displayed at the end.
  const results = {}
  for (let commit of commits) await testCommit(commit, say.tag(`#testing{${commit}}`))
  async function testCommit (commit, say) {
    say(`-----------------------------------------------------------------------------------------`)
    try {
      // (This could've been done in parallel, but trying to
      // upload multiple contracts in 1 block crashes something?)
      // ### Deploy the token
      const TOKEN = await SNIP20Contract.fromCommit({say, agent: ADMIN, commit})
      // ### Generate viewing keys
      const [vkALICE, vkBOB] = await Promise.all([
        TOKEN.createViewingKey(ALICE, ALICE.address),
        TOKEN.createViewingKey(BOB,   BOB.address),
      ])
      // ### Deploy and launch the vesting manager
      const MGMT = await MGMTContract.fromCommit({say, agent: ADMIN, commit, token: TOKEN}) 
      let schedule
      await Promise.all([
        await MGMT.acquire(TOKEN), // * Give it a token to issue
        await MGMT.configure(schedule = getSchedule({ ALICE, BOB })) // * Configure the vesting
      ])
      await MGMT.launch() // * Launch it
      say(`launched vesting ----------------------------------------------------------------------`)
      // ## Normal behavior
      // Suppose `ALICE` and `BOB` are two swap contracts scheduled to
      // receive funds from the Liquidity Provision Fund.
      // * ALICE claims every portion.
      // * BOB claims no portions.
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
            say(' #warning')('not the error we were expecting')
          } else {
            const log = JSON.parse(error.log) 
            say.tag(' #error')(log)
            // SecretJS should parse error.log for us (assuming it's always JSON if present...?)
            // * `rc` builds respond with `nothing for you`, meaning `0` funds claimable for ALICE.
            // * `main` builds responds with `remainders not supported...`, meaning the contract's
            //   pre-launch self-validation logic has deemed the schedule erroneous (success!)
            //
            // It's the second one that provides the most clues to the real underlying issue:
            // failing to use the right allocation set.
            if ( 
              log.generic_err && (
                log.generic_err.msg === 'nothing for you' ||
                log.generic_err.msg === 'channel Channel1: remainders not supported alongside split allocations'
              )
            ) { 
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
      // Now ALICE should have `6 * 100000000000` and BOB should still have `0`:
      await TOKEN.balance({ agent: ALICE, viewkey: vkALICE, address: ALICE.address })
      await TOKEN.balance({ agent: BOB,   viewkey: vkBOB,   address: BOB.address   })
      // Because of constraints introduced in `rc2`, a remainder can't be vested to multiple
      // recipients (meaning of `SCL-07`). This means it needs to be manually reallocated,
      // otherwise it's stuck in the contract.
      //
      // So it doesn't make sense to `Disown` the contract (`MGL-03`) if it
      // contains both remainders and split allocations, because if manual intervention
      // is impossible the remainders would remain unclaimable forever.
      //
      // ## Abnormal behavior
      //
      // Indeed, remainders do remain unclaimable forever, on both `main` and `rc5`,
      // for a slightly different reason on either. (The original suspected issue would've
      // had ALICE receive BOB's unclaimed portions alongside the remainder.)
      //
      // What actually happens is **the call to `reallocate` fails silently**, so the remainders
      // can't be transferred at all. If the `reallocate` was correct, it's likely that the original
      // issue would've manifested.
      //
      // Here's how the admin allocates a tiny bit of the remainder to ALICE.
      say(`reallocate remainders -----------------------------------------------------------------`)
      await MGMT.reallocate(
        schedule.pools[0].name,
        schedule.pools[0].channels[0].name,
        [ { addr:   ALICE.address
          , amount: "1" } ])
      await ADMIN.waitForNextBlock()
      await MGMT.claim(ALICE) // ALICE claims remainder.
      await TOKEN.balance({ agent: ALICE, viewkey: vkALICE, address: ALICE.address })
      // **OOPS**! If allocations worked, here ALICE would've received all the channel's available
      // funds, including BOB's unclaimed portions (sorry Bob), in addition to the allocated
      // crumb of the remainder.
      await MGMT.claim(BOB)
      await TOKEN.balance({ agent: BOB, viewkey: vkBOB, address: BOB.address })
      await ADMIN.waitForNextBlock() // Pause for a block before trying with the next version
      say(`ok, next version ----------------------------------------------------------------------`)
      results[commit] = 'OK' // store success
    } catch (e) {
      say.tag(' #error')(`------------------------------------------------------------------------`)
      console.error(e)
      results[commit] = e // store error
      say(`done with ${commit}; next? ------------------------------------------------------------\n`)
    }
  }
  return results
}
// # Findings and conclusions
//
// ## Suggested next steps
// * A simplified, **hardened version of the schedule logic** (`schedule 2`) is already nearing
//   completion, and we will soon proceed to incrementally merge it into the `main` development
//   branch. In absence of further "discoveries", that will be the last major update to the
//   vesting contract.
// * As the SecretNetwork development workflow revolves around downloading marginally verified
//   root binaries (Docker) and user-level source code (NPM), a **further audit of the vendor
//   supply chain** is recommended; **reproducible builds** (for tooling) and **cargo-crev**
//   (for contracts) may help.
// * **Submit pull requests to SecretJS** with various things that make it more usable;
//   coordinate with Enigma to make sure it lands at the right time; maybe we can help them
//   document their JS API and publish the generated documentation?
//   * Need to implement error schema on the JS end that corresponds to Rust error variants
//   * Untangle error handling from marshaling from crypto
//   * Try to flatten stack of `EnigmaUtils->RestClient->CosmWasmClient->SigningCosmWasmClient->Agent`
//   * Implement generic `Agent` that creates a local async proxy in JS-land to easily call contract
//     methods based on the generated schema
//     * It doesn't look like any of the JSON schema generated by the Rust crates is taken into
//       account by SecretJS, although that would be the most obvious way to use it. My guess
//       is that Enigma rely on TypeScript type annotations rather than language-agnostic schema?
// * Even in a slow-moving environment such as a blockchain, **good iteration rates** are needed
//   to ensure quality developer attention. Keeping the build/test cycle under a minute (in
//   a warmed-up environment) is a good point of reference.
//   * For that purpose, the `init` method of `mgmt` can be augmented to allow loading a schedule,
//     acquiring control over the token contract, and launching the vesting, in a single operation.
//     Bundling those in one transaction may also be easier on gas costs, as it avoids multiple
//     rounds of messages being sent back and forth through the crypto layer.
//   * Further gas optimisation: `schedule 2` will support pre-computing the whole vesting
//     off-chain (as well as generating `Portion`s on the fly) - need to compare which option
//     is more efficient
// * Still unclear how claim transactions will be initiated; maybe we need to wrap the vesting
//   contract into a **HTTP API** for users (relatively easy), or as an add-on to MetaMask/Keplr?
//   * Determine how the DEX/AMM contracts will claim funds from the **Liquidity Provision Fund**?
// * Allow **early returns** in `mgmt` via update to the `fadroma` macro, in order to
//   **reduce conditional complexity** of the contract methods (which are actually
//   just block expressions and as such can't contain early returns)
// * Allow schedule configs to be moved between unit and integration tests as JSON
// * Figure out a way to run the blockchain faster/**time travel** (libfaketime?), in order to
//   run tests faster/**integration test the real schedule**.
// * The `mgmt` contract does not implement a way for anyone to **query its progress**. Add one?
//
// ## Observations
// * The architecture of this platform and its associated APIs seems reasonable enough;
//   so far, I've seen exactly **zero glaring design faults** with disaster potential.
//   The simplicity of the platform APIs is a virtue, however it also **highlights any
//   remaining oversights**.
//   * Just like it can be easy to underestimate the small quality-of-life details that
//     make up a coherent platform experience, I've admittedly fallen trap to overestimating
//     various incongruences here, initially seeing them as more severe than what their impact
//     amounts to. Better safe than sorry though.
//   * Certain `wtf`s and deficiencies in the platform inevitably emerge in due course;
//     Expect all such inconveniences to be fixed as needed, either by the platform devs
//     in their normal course of work, or by us in the process of preparing SIENNA for primetime.
//     The Secret Network codebase is certainly concise and malleable enough to allow for
//     the latter.
// * For example, during the construction of this integration test, which checks for 
//   the "correct" error being thrown in order to know when to proceed to the
//   second stage of the woult-be "exploit", it became clear that **error handling over
//   the Rust/JS barrier** is underdeveloped.
//   * it hangs on a regex match (this looks like something that needs to be fixed
//     on the server side, nodes should return structured data on errors too)
//   * after my intervention, at least the API consumer doesn't need to do the same thing again
//     to extract the JSON error contents from a contract error forwarded by the API,
//     and there aren't multiple instances of the same error decryption logic on different
//     levels of the library.
// * **SecretJS**, the library that is ostensibly meant to be used to connect
//   the Secret Contract to the outside world, feels like it needs more developer attention.
//   * An **improved JS API** stands out as an obvious milestone to **wider adoption** of
//     Secret Network-based products. It's workable as it is, but I expect further integration
//     work to expose even more room for improvement. Research in this direction is already ongoing
//     as part of the current implementation.
//   * Overall **lack of SecretJS documentation** other than a handful of examples on GitHub.
//     * The **examples omit critical steps** - such as creating an account (privkey/pubkey pair)
//       without the help of a browser extension (!).
//     * SecretJS **versions released in the presence of failing test cases**?
//
// # Appendix A - Schedule
//
// * The following is a valid JSON schedule:
//   * an intermediate representation between the configuration spreadsheet
//     and the `schedule` module's actual in-memory model.
//   * a guide to the contract's execution model.
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
