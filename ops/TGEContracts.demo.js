#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # SIENNA Vesting Contract Demo
//
// ## What you're looking at
//
// This script is intended to demonstrate correct behavior of the smart contracts
// when interoperating with a JS environment.
//
// **Run this with `./sienna.js demo`.**
// * `./sienna.js demo --testnet` runs this on `holodeck-2`
// * `./sienna.js demo` runs this on a local testnet in a Docker container, which has id
//   `enigma-pub-testnet-3`, and is referred to as `localnet`. Needs
//   [Docker](https://docs.docker.com/get-docker/).
//
// ## The following features are tested:
// * 👷 **deploying** and **configuring** the token, mgmt, and rpt contracts.
// * ⚠️  **viewing unlocked funds for any known address** without having to make a claim
// * 💸 **making claims** according to the initial **schedule** (sped up by a factor of 8400)
// * 🤵 **allocating unassigned funds** from a pool to a **new account**
// * 💰 **splitting the Remaining Pool Tokens** between multiple addresses
// * 🍰 **reconfiguring that split**, preserving the **total portion size**
import assert from 'assert'
import { loadJSON, taskmaster, bignum, fileURLToPath, resolve, dirname } from '@fadroma/utilities'
import { SecretNetwork } from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'
import { fmtSIENNA } from './lib/index.js'
import TGEContracts from './TGEContracts.js'
import { SNIP20Contract, MGMTContract, RPTContract } from '../api/index.js'
//
// Required: access to a testnet (holodeck-2), or in absence of testnet,
// a handle to a localnet (automatically instantiated
// in a Docker container from `sienna.js`)

const projectRoot = resolve(fileURLToPath(dirname(import.meta.url)), '..')

// ## Overview of the demo procedure
/** Conducts a test run of the contract deployment. */
export default async function demo (environment) {
  // * The operational **environment** provided by [Fadroma](https://fadroma.tech/js/)
  //   contains the `agent` and `builder` helpers, as well as the `chainId` of the `network`.
  const {network, agent, builder} = environment
  // * **taskmaster** is a tiny high-level profiler that records how much time and gas
  // each operation took, and writes a report in `artifacts` with a Markdown table of events.
  const header = [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ]
      , output = resolve(projectRoot, 'artifacts', network.chainId, 'profile-deploy.md')
      , task   = taskmaster({ header, output, agent })
  // * Prepare **schedule** and **recipients**
  //   * The schedule is shortened by a factor of 86400 (number of seconds in a day)
  //     in order to run in  about 15 minutes. This is necessitated by the node being
  //     resistant to `libfaketime`.
  //   *  The recipient wallets are created if they don't exist -
  //      the admin sendings a gas budget to them (in uSCRT).
  const schedule   = loadJSON('../settings/schedule.json', import.meta.url)
      , prepared   = await prepare({task, network, agent, schedule})
      , wallets    = prepared.wallets
      , recipients = prepared.recipients
  // * **Build**, **deploy**, and **initialize** contracts
      , contracts  = new TGEContracts()
      , binaries   = await contracts.build({task, builder})
      , receipts   = await contracts.upload({task, builder, binaries})
      , initArgs   = {task: task, agent, receipts, schedule}
      , instances  = await contracts.initialize({
        ...initArgs,
        initialRPTRecipient: agent.address
      })
  // * **Launch** the vesting and confirm that the **claims** and **mutations** work as specified.
  await verify({task, agent, recipients, wallets, instances, schedule})
}

// # Preparation
async function prepare ({task, network, agent, schedule}) {

  // * Let's delete the `AdvisorN` account from the schedule
  // to allow the `AddAccount` method to be tested.
  await task('allow adding accounts to Advisors pool in place of AdvisorN', () => {
    for (const pool of schedule.pools) if (pool.name === 'Advisors') {
      pool.partial = true
      for (const i in pool.accounts) if (pool.accounts[i].name === 'AdvisorN') {
        pool.accounts.splice(i, 1)
        break } break } })

  // * And now, for my next trick, I'm gonna need some **wallets**!
  const recipientGasBudget = bignum(10000000) // uscrt
      , wallets    = []
      , recipients = {}
  await task('shorten schedule and replace placeholders with test accounts', async () => {
    await Promise.all(schedule.pools.map(pool=>Promise.all(pool.accounts.map(
      async function mutateAccount (account) {
        // Create an agent with a new address for each recipient account.
        const recipient = await network.getAgent(account.name)
        const {address} = recipient
        // Put that address in the schedule
        account.address = address
        wallets.push([address, 10000000]) // balance to cover gas costs
        recipients[account.name] = {agent: recipient, address, total: account.amount} // store agent
        // While we're here, *divide all timings in that account by 86400*,
        // so that a day passes in a second
        account.start_at /= 86400
        account.interval /= 86400
        account.duration /= 86400 })))) })

  // * Some more wallets please. These will be used for the mutation tests.
  await task('create extra test accounts for reallocation tests', async () => {
    const extras = [ 'NewAdvisor', 'TokenPair1', 'TokenPair2', 'TokenPair3', ]
    for (const name of extras) {
      const extra = await network.getAgent(name) // create agent
      wallets.push([extra.address, recipientGasBudget.toString()])
      recipients[name] = {agent: extra, address: extra.address} } })

  // * Make sure the wallets exist on-chain.
  await ensureWallets({ task, connection: null, agent, wallets, recipients, recipientGasBudget })

  return { wallets, recipients }
}

// # Verification
export async function verify ({task, agent, recipients, wallets, instances, schedule}) {

  const { TOKEN, MGMT, RPT } = instances

  // Let's just give every recipient an empty viewing key so we can check their balances.
  const VK = ""
  await task(`set null viewing key on ${Object.keys(recipients).length} SIENNA accounts`,
    async report => {
      let txs = Object.values(recipients).map(({agent})=>TOKEN.setViewingKey(agent, VK))
      txs = await Promise.all(txs)
      for (const {tx} of txs) report(tx.transactionHash) })

  // ## And let's go! 🚀
  let launched
  await task('launch the vesting', async report => {
    const {transactionHash, logs} = await MGMT.launch()
    launched = 1000 * Number(logs[0].events[1].attributes[1].value)
    report(transactionHash) })

  // Okay, **new taskmaster instance** (2nd part of profiling - runtime).
  // This one will measure the claims.
  await task.done()
  task = taskmaster({
    header: [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ],
    output: resolve(projectRoot, 'artifacts', agent.network.chainId, 'profile-runtime.md'),
    agent })

  // The following happen **once** in the whole test cycle:
  let addedAccount = false
  let reallocated  = false

  // And this one is expected to happen **zero** times:
  let error

  // ## Main test loop 🔁
  while (true) {
    try {
      await agent.nextBlock
      const now = new Date()
      const elapsed = now - launched
      console.info(`\n⏱️  ${Math.floor(elapsed/1000)} "days" (seconds) after launch:`)

      const claimable = []

      // ⚠️  Vesting info is public!
      await task('query vesting progress', async report => {
        process.env.NODEBUG = true
        console.info( `ACCOUNT`.padEnd(11)
                    , `CLAIMED`.padEnd(26), `  `
                    , `UNLOCKED`.padEnd(26), `  `
                    , `TOTAL`.padEnd(26) )
        for (const [name, recipient] of Object.entries(recipients)) {
          if (name.startsWith('TokenPair')) continue // token pairs are only visible to the RPT contract
          const {progress} = await MGMT.progress(recipient.address, now)
          const {claimed, unlocked} = progress
          console.info( `${name}`.padEnd(11)
                      , fmtSIENNA(claimed).padStart(26), `of`
                      , fmtSIENNA(unlocked).padStart(26), `of`
                      , fmtSIENNA(recipient.total||0).padStart(26)
                      , 'SIENNA')
          if (name === 'RPT') continue
          // Every iteration, one random recipient
          // with newly unlocked balance will claim. Collect the names of such recipients:
          if (progress.claimed < progress.unlocked) claimable.push(name)
        }
        process.env.NODEBUG = false
      })

      if (claimable.length > 0) {
        await task('make one claim', async report => {
          // * **Test claim.**
          const claimant  = claimable[Math.floor(Math.random() * claimable.length)]
              , _         = console.info(`\n${claimant} claims...`)
              , recipient = recipients[claimant]
              , tx        = await MGMT.claim(recipient.agent)
              , balance   = String(await TOKEN.balance(recipient.address, VK))
          console.info(`balance of ${claimant} is now: ${fmtSIENNA(balance)} SIENNA`)
          report(tx.transactionHash) }) }

      // * **Test mutation 1**: add account, occurs 20 "days" in
      if (!addedAccount && elapsed > 20000) {
        await task('add new account to advisors pool', async report => {
          addedAccount = true
          const tx = await MGMT.add('Advisors', {
            name:     'NewAdvisor',
            address:  recipients['NewAdvisor'].address,
            amount:   "600000000000000000000",
            cliff:    "100000000000000000000",
            start_at: Math.floor(elapsed / 1000) + 5,
            interval: 5,
            duration: 25 })
          report(tx.transactionHash) }) }

      // * **Test mutation 2**: reallocate RPT, occurs 30 "days" in
      if (!reallocated && elapsed > 30000) {
        await task('reallocate RPT...', async report => {
          reallocated = true
          const tx = await RPT.configure([
            [recipients.TokenPair1.address,  "250000000000000000000"],
            [recipients.TokenPair2.address, "1250000000000000000000"],
            [recipients.TokenPair3.address, "1000000000000000000000"] ])
          report(tx.transactionHash) }) }

      // * **Test RPT vesting**. This 
      // Since claims happen every ~5 seconds (= 1 block = 5 "days" of the shortened schedule)
      // if this method is meant to be called daily, its cost must be multiplied by 5.
      await task('vest RPT tokens', async report => {
        const tx = await RPT.vest()
        report(tx.transactionHash) })
      await task('query balances of RPT recipients', async report => {
        for (const [name, recipient] of Object.entries(recipients)) {
          if (name.startsWith('TokenPair')) {
            console.log(
              `${name}:`.padEnd(15),
              fmtSIENNA(String(await TOKEN.balance(recipient.address, VK))).padStart(30),
              'SIENNA') } } })

    } catch (e) {
      error = e
      console.info(`demo exited with error: ${e.message}`)
      console.error(e)
      break
    }
  }

  await task.done() // save the runtime profile even on error
  if (error) throw error
}
