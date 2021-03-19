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
//     `Schedule(Pool(Channel(Periodic,Vec<(Seconds,Vec<Allocation>))`
//     has now become simply `Schedule(Pool(Account))`. The two alternate vesting modes
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
compare().then(console.log)
async function compare ({

  // The contracts:
  SNIP20Contract = class SNIP20Contract extends require('@hackbg/snip20') { // * Token
    static fromCommit = async ({commit, agent}) => super.fromCommit({ ...args,
      name: `TOKEN{${args.commit}}`, binary: `${args.commit}-snip20-reference-impl.wasm`, data:
        { name:      "Sienna"
        , symbol:    "SIENNA"
        , decimals:  18
        , admin:     args.agent.address
        , prng_seed: "insecure"
        , config:    { public_total_supply: true } } }) },

  MGMTContract = class MGMTContract extends require('@hackbg/mgmt') { // * Vesting
    static fromCommit = async ({commit,token,...args}) => super.fromCommit({ ...args,
      name: `MGMT{${commit}}`, binary: `${commit}-sienna-mgmt.wasm`, data:
        { token_addr: token.address
        , token_hash: token.hash
        , ...args.data } }) },

  RPTContract = class RPTContract extends require('@hackbg/rpt') { // * Splitting
    static fromCommit = async ({commit,token,mgmt,...args}) => super.fromCommit({ ...args,
      name: `RPT{${commit}}`, binary: `${commit}-sienna-rpt.wasm`, data:
        { token_addr: token.address
        , token_hash: token.hash
        , mgmt_addr:  mgmt.address
        , mgmt_hash:  mgmt.hash
        , ...args } }) },

  // Tools for interacting with the contracts:
  Fadroma = require('@hackbg/fadroma'),
  now     = () => new Date().toISOString(), // * Timestamper
  say     = Fadroma.say.tag(`${now()} `),   // * Logger
  Agent = Fadroma.SecretNetworkAgent, // wrapper around SigningCosmWasmClient

  // Pre-existing testnet wallets with gas money:
  MNE   = x => require(`/shared-keys/${x}.json`).mnemonic, // get mnemonic from file
  ADMIN = Agent.fromMnemonic({say, name: "ADMIN", mnemonic: MNE("ADMIN")}),
  ALICE = Agent.fromMnemonic({say, name: "ALICE", mnemonic: MNE("ALICE")}),
  BOB   = Agent.fromMnemonic({say, name: "BOB",   mnemonic: MNE("BOB")  }),

  // List of Git refs to compare:
  commits = [ `main` ],

}={}) {

  // The usual preparation. Check report 001 for info about this.
  ;[ADMIN, ALICE, BOB] = await Promise.all([ADMIN, ALICE, BOB])
  await Promise.all([ADMIN,ALICE,BOB].map(x=>x.status()))
  const results = {}
  for (let commit of commits) await testCommit(commit, say.tag(`#testing{${commit}}`))

  async function testCommit (commit, say) {
    say(`-----------------------------------------------------------------------------------------`)
    try {

      // ### The token
      const TOKEN =             // Deploy the token
        await SNIP20Contract.fromCommit({say, agent: ADMIN, commit})
      const [vkALICE, vkBOB] =  // Generate viewing keys to monitor token balances
        await Promise.all([ALICE,BOB].map(x=>TOKEN.createViewingKey(x, "entropy")))

      // ### The vesting
      const schedule =          // This time, let's try with the real schedule
        getSchedule({ ALICE, BOB })
      const MGMT =              // Deploy the vesting manager with the schedule
        await MGMTContract.fromCommit({say, agent: ADMIN, commit, token: TOKEN, schedule})
      await MGMT.acquire(TOKEN) // Make MGMT the admin and sole minter of the token

      // ### The RPT splitter
      const RPT =
        await RPTContract.fromCommit({say, agent: ADMIN, commit, token: TOKEN, mgmt: MGMT})

      // ### Launch the vesting
      await MGMT.launch()

    } catch (e) {
      say.tag(' #error')(`------------------------------------------------------------------------`)
      console.error(e)
      results[commit] = e // store error
      say(`done with ${commit}; next? ------------------------------------------------------------\n`)
    }
  }
  return results
}

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
