#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # SIENNA Vesting Contract Demo
// * by using a local testnet container,
// * this script demonstrates:
//   * deploying and configuring the token and vesting contracts
//   * making claims according to the initial schedule
//   * checking unlocked funds without making a claim
//   * splitting the Remaining Pool Tokens between multiple addresses
//   * reconfiguring the Remaining Pool Token split, preserving the total portion size
//   * adding new accounts to Advisor/Investor pools
import assert from 'assert'
import { fileURLToPath } from 'url'
import { resolve, dirname } from 'path'
import { loadJSON, taskmaster, SecretNetwork } from '@hackbg/fadroma'

import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

import { build, upload, initialize } from './ops.js'

const __dirname = fileURLToPath(dirname(import.meta.url))

export default async function demo (conn) {
  // fadroma provides a connection as well as agent and builder classes
  const {network, agent, builder} = conn
  // record timing and gas costs of deployment operations
  const header = [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ]
      , output = resolve(__dirname, 'artifacts', network.chainId, 'profile-deploy.md')
      , deployTask = taskmaster({ header, output, agent })
  // modify schedule and prepare environment for demo
  const schedule = loadJSON('./settings/schedule.json', import.meta.url)
      , {wallets, recipients} = await prepare(deployTask, network, agent, schedule)
  // build, deploy, and initialize contracts
  const binaries = await build({task: deployTask, builder})
      , receipts = await upload({task: deployTask, builder, binaries})
      , initialRPTRecipient = recipients.TokenPair1.address
      , initArgs = {task: deployTask, agent, receipts, schedule}
      , contracts = await initialize({...initArgs, initialRPTRecipient})
  // launch the vesting and confirm that the claims work as expected
  await verify(deployTask, agent, recipients, wallets, contracts, schedule)
}

async function prepare (task, chain, agent, schedule) {
  const wallets = [], recipients = {}
  await task('allow adding accounts to Advisors pool in place of AdvisorN', () => {
    for (const pool of schedule.pools) if (pool.name === 'Advisors') {
      pool.partial = true
      for (const i in pool.accounts) if (pool.accounts[i].name === 'AdvisorN') {
        pool.accounts.splice(i, 1)
        break } break } })
  await task('shorten schedule and replace placeholders with test accounts', async () => {
    await Promise.all(schedule.pools.map(pool=>Promise.all(pool.accounts.map(mutateAccount))))
    async function mutateAccount (account) {
      // * create an agent for each recipient address (used to test claims)
      const {name} = account
      const recipient = await chain.getAgent(name) // create agent
      const {address} = recipient
      account.address = address         // replace placeholder with real address
      wallets.push([address, 10000000]) // balance to cover gas costs
      recipients[name] = {agent: recipient, address, total: account.amount} // store agent
      // * divide all times in account by 86400, so that a day passes in a second
      account.start_at /= 86400
      account.interval /= 86400
      account.duration /= 86400 } })
  await task('create extra test accounts for reallocation tests', async () => {
    const extras = [ 'NewAdvisor', 'TokenPair1', 'TokenPair2', 'TokenPair3', ]
    for (const name of extras) {
      const extra = await chain.getAgent(name) // create agent
      wallets.push([extra.address, 10000000])
      recipients[name] = {agent: extra, address: extra.address} } })
  await task(`create ${wallets.length} wallets for the test accounts`, async report => {
    const tx = await agent.sendMany(wallets, 'create recipient accounts')
    report(tx.transactionHash) })
  return { wallets, recipients } }

export async function verify (task, agent, recipients, wallets, contracts, schedule) {
  const { TOKEN, MGMT, RPT } = contracts
  const VK = ""
  await task('set null viewing keys', async report => {
    let txs = Object.values(recipients).map(({agent})=>TOKEN.setViewingKey(agent, VK))
    txs = await Promise.all(txs)
    for (const {tx} of txs) report(tx.transactionHash) })
  let launched
  await task('launch the vesting', async report => {
    const {transactionHash, logs} = await MGMT.launch()
    launched = 1000 * Number(logs[0].events[1].attributes[1].value)
    report(transactionHash) })
  await task.done()
  task = taskmaster({
    header: [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ],
    output: resolve(__dirname, 'artifacts', agent.network.chainId, 'profile-runtime.md'),
    agent })
  let addedAccount = false
  let reallocated  = false
  while (true) {
    try {
      await agent.nextBlock

      const now = new Date()
      const elapsed = now - launched
      console.info(`\n⏱️  ${Math.floor(elapsed/1000)} "days" (seconds) after launch:`)

      const claimable = []

      await task('query vesting progress', async report => {
        console.info( `ACCOUNT`.padEnd(11)
                    , `CLAIMED`.padEnd(25), `  `
                    , `UNLOCKED`.padEnd(25), `  `
                    , `TOTAL`.padEnd(25) )
        for (const [name, recipient] of Object.entries(recipients)) {
          // token pairs are only visible to the RPT contract
          // so it doesn't make sense to pass them to the `Progress` query
          if (name.startsWith('TokenPair')) continue
          const {progress} = await MGMT.progress(recipient.address, now)
          const {claimed, unlocked} = progress
          console.info( `${name}`.padEnd(11)
                      , claimed.padStart(25), `of`
                      , unlocked.padStart(25), `of`
                      , (recipient.total||'').padStart(25) )
          if (name === 'RPT') continue
          // one random recipient with newly unlocked balance will claim:
          if (progress.claimed < progress.unlocked) claimable.push(name) } })

      if (claimable.length > 0) {
        await task('make one claim', async report => {
          const claimant = claimable[Math.floor(Math.random() * claimable.length)]
          console.info(`\n${claimant} claims...`)
          const recipient = recipients[claimant]
          const tx = await MGMT.claim(recipient.agent)
          const balance = String(await TOKEN.balance(recipient.agent, VK))
          console.info(`balance of ${claimant} is now: ${balance}`)
          report(tx.transactionHash) }) }

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

      if (!reallocated && elapsed > 30000) {
        await task('reallocate RPT...', async report => {
          reallocated = true
          const tx = await RPT.configure([
            [recipients.TokenPair1.address,  "250000000000000000000"],
            [recipients.TokenPair2.address, "1250000000000000000000"],
            [recipients.TokenPair3.address, "1000000000000000000000"] ])
          report(tx.transactionHash) }) }

      await task('vest RPT tokens', async report => {
        const tx = await RPT.vest()
        report(tx.transactionHash) })

      await task('query balances of RPT recipients', async report => {
        for (const [name, recipient] of Object.entries(recipients)) {
          if (name.startsWith('TokenPair')) {
            console.log(
              `${name}:`.padEnd(15),
              String(await TOKEN.balance(recipient.agent, VK)).padStart(30)) } } })

    } catch (e) {
      console.info(`demo exited with error: ${e.message}`)
      console.error(e)
      break
    }
  }

  await task.done()
}
