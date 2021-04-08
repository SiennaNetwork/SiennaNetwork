#!/usr/bin/env node
/* vim: set ts=2 sts=2 sw=2 et cc=100 */
// # SIENNA Vesting Contract Demo
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
import { resolve, dirname } from 'path'
import { loadJSON, taskmaster, SecretNetwork } from '@hackbg/fadroma'
import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

import { build, upload, initialize } from './ops.js'

const __dirname = fileURLToPath(dirname(import.meta.url))

export default async function demo ({network, agent, builder}) {

  const deployTask = taskmaster({
    header: [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ],
    output: resolve(__dirname, 'artifacts', 'profile-deploy.md'),
    agent
  })

  const here                  = import.meta.url
      , schedule              = loadJSON('./settings/schedule.json', here)
      , {recipients, wallets} = await prepare(network, agent, schedule)
      , contracts             = await deploy(builder, schedule, recipients)
      , result                = await verify(agent, recipients, wallets, contracts, schedule)

  async function prepare (chain, agent, schedule) {

    await deployTask('allow adding accounts to Advisors pool in place of AdvisorN', () => {
      for (const pool of schedule.pools) {
        if (pool.name === 'Advisors') {
          pool.partial = true
          for (const i in pool.accounts) {
            if (pool.accounts[i].name === 'AdvisorN') {
              pool.accounts.splice(i, 1)
              break
            }
          }
          break
        }
      }
    })

    const wallets    = []
        , recipients = {}

    await deployTask('shorten schedule and replace placeholders with test accounts', async () => {
      await Promise.all(schedule.pools.map(pool=>Promise.all(pool.accounts.map(mutateAccount))))
      async function mutateAccount (account) {
        // * create an agent for each recipient address (used to test claims)
        const {name} = account
        const recipient = await chain.getAgent(name) // create agent
        const {address} = recipient
        account.address = address         // replace placeholder with real address
        wallets.push([address, 10000000]) // balance to cover gas costs
        recipients[name] = {agent: recipient, address} // store agent

        // * divide all times in account by 86400, so that a day passes in a second
        account.start_at /= 86400
        account.interval /= 86400
        account.duration /= 86400
      }
    })

    await deployTask('create extra test accounts for reallocation tests', async () => {
      const extras = [ 'NewAdvisor', 'TokenPair1', 'TokenPair2', 'TokenPair3', ]
      for (const name of extras) {
        const extra = await chain.getAgent(name) // create agent
        wallets.push([extra.address, 10000000])
        recipients[name] = {agent: extra, address: extra.address}
      }
    })

    return {recipients, wallets}

  }

  async function deploy (builder, schedule, recipients) {
    const workspace  = dirname(fileURLToPath(here))
    builder.configure({
      buildImage: 'enigmampc/secret-contract-optimizer:latest',
      buildUser:  'root',
      outputDir:  resolve(workspace, 'artifacts'), })
    const binaries  = await build({ task: deployTask, builder })
    const receipts  = await upload({ task: deployTask, builder, binaries })
    const timestamp = String(+ new Date())
    const contracts = await initialize({ task: deployTask, agent, receipts, inits: {
      TOKEN: { label:  `[${timestamp}] snip20`
             , initMsg: { name:      "Sienna"
                        , symbol:    "SIENNA"
                        , decimals:  18
                        , admin:     builder.agent.address
                        , prng_seed: "insecure"
                        , config:    { public_total_supply: true } } },
      MGMT:  { label:  `[${timestamp}] mgmt`
             , initMsg: { schedule } },
      RPT:   { label:  `[${timestamp}] rpt`
             , initMsg: { portion:   "2500000000000000000000"
                        , config:    [[recipients.TokenPair1.address, "2500000000000000000000"]]}} } })
    return contracts
  }

  async function verify (agent, recipients, wallets, contracts, schedule) {

    const { TOKEN, MGMT, RPT } = contracts

    await deployTask(`create ${wallets.length} recipient accounts`, async report => {
      const tx = await agent.sendMany(wallets, 'create recipient accounts')
      report(tx.transactionHash)
    })

    const VK = ""
    await deployTask('set null viewing keys', async report => {
      let txs = Object.values(recipients).map(({agent})=>TOKEN.setViewingKey(agent, VK))
      txs = await Promise.all(txs)
      for (const {tx} of txs) report(tx.transactionHash)
    })

    await deployTask('make mgmt owner of token', async report => {
      const [tx1, tx2] = await MGMT.acquire(TOKEN) // TODO auto-acquire on init
      report(tx1.transactionHash)
      report(tx2.transactionHash)
    })

    await deployTask('point RPT account in schedule to RPT contract', async report => {
      schedule.pools.filter(x=>x.name==='MintingPool')[0]
              .accounts.filter(x=>x.name==='RPT')[0]
              .address = RPT.address
      recipients['RPT'] = { address: RPT.address }
      const {transactionHash} = await MGMT.configure(schedule)
      report(transactionHash)
    })

    let launched
    await deployTask('launch the vesting', async report => {
      const {transactionHash, logs} = await MGMT.launch()
      launched = 1000 * Number(logs[0].events[1].attributes[1].value)
      report(transactionHash)
    })

    await deployTask.done()

    const runtimeTask = taskmaster({
      header: [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ],
      output: resolve(__dirname, 'artifacts', 'profile-runtime.md'),
      agent
    })

    let addedAccount = false
    let reallocated  = false
    while (true) {
      try {
        await agent.nextBlock

        const now = new Date()
        const elapsed = now - launched
        console.info(`\n⏱️  ${Math.floor(elapsed/1000)} "days" (seconds) after launch:`)

        const claimable = []

        await runtimeTask('query vesting progress', async report => {
          for (const [name, recipient] of Object.entries(recipients)) {
            if (name.startsWith('TokenPair')) {
              // token pairs are only visible to the RPT contract
              // so it doesn't make sense to pass them to the `unlocked` method
              continue
            }
            const {progress} = await MGMT.progress(recipient.address, now)
            console.info(
              `${name}:`.padEnd(15),
              progress.claimed.padStart(30), `/`, progress.unlocked.padStart(30)
            )
            // one random recipient with newly unlocked balance will claim:
            if (name !== 'RPT' && progress.claimed < progress.unlocked) {
              claimable.push(name)
            }
          }
        })

        if (claimable.length > 0) {
          await runtimeTask('make one claim', async report => {
            const claimant = claimable[Math.floor(Math.random() * claimable.length)]
            console.info(`\n${claimant} claims...`)
            const recipient = recipients[claimant]
            await MGMT.claim(recipient.agent)
            const balance = String(await TOKEN.balance(recipient.agent, VK))
            console.info(`balance of ${claimant} is now: ${balance}`)
          })
        }

        if (!addedAccount && elapsed > 20000) {
          await runtimeTask('add new account to advisors pool', async report => {
            addedAccount = true
            await MGMT.add('Advisors', {
              name:     'NewAdvisor',
              address:  recipients['NewAdvisor'].address,
              amount:   "600000000000000000000",
              cliff:    "100000000000000000000",
              start_at: Math.floor(elapsed / 1000) + 5,
              interval: 5,
              duration: 25,
            })
          })
        }

        if (!reallocated && elapsed > 30000) {
          await runtimeTask('reallocate RPT...', async report => {
            reallocated = true
            await RPT.configure([
              [recipients.TokenPair1.address,  "250000000000000000000"],
              [recipients.TokenPair2.address, "1250000000000000000000"],
              [recipients.TokenPair3.address, "1000000000000000000000"],
            ])
          })
        }

        await runtimeTask('vest RPT tokens', async report => {
          await RPT.vest()
        })

        await runtimeTask('query balances of RPT recipients', async report => {
          for (const [name, recipient] of Object.entries(recipients)) {
            if (name.startsWith('TokenPair')) {
              console.log(
                `${name}:`.padEnd(15),
                String(await TOKEN.balance(recipient.agent, VK)).padStart(30)
              )
            }
          }
        })

      } catch (e) {
        console.info(`demo exited with error: ${e.message}`)
        console.error(e)
        break
      }
    }

    await runtimeTask.done()
  }

}
