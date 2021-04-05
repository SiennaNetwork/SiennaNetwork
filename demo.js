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

  const task = taskmaster({
    header: [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ],
    output: resolve(__dirname, 'docs', 'gas-report.md'),
    agent
  })

  const here       = import.meta.url
  const schedule   = loadJSON('./settings/schedule.json', here)
  const recipients = await prepare(network, agent, schedule)
  const contracts  = await deploy(builder, schedule, recipients)
  const result     = await verify(agent, recipients, contracts, schedule)

  async function prepare (chain, agent, schedule) {

    const wallets    = []
        , recipients = {}

    await task('shorten schedule and replace placeholders with test accounts',
      async () => {
        await Promise.all(schedule.pools.map(function mutatePool (pool) {
          return Promise.all(pool.accounts.map(mutateAccount))
        }))
        async function mutateAccount (account) {
          // * create an agent for each recipient address (used to test claims)
          const {name} = account
          const recipient = await chain.getAgent(name) // create agent
          const {address} = recipient
          account.address = address        // replace placeholder with real address
          wallets.push([address, 1000000]) // balance to cover gas costs
          recipients[name] = {agent: recipient, address } // store agent

          // * divide all times in account by 86400, so that a day passes in a second
          account.start_at /= 86400
          account.interval /= 86400
          account.duration /= 86400
        }
      })

    await task('create extra test accounts for reallocation tests', async () => {
      for (let name of [ // extra accounts for reconfigurations
        'TokenPair1', 'TokenPair2', 'TokenPair3',
        'NewAdvisor', 'NewInvestor1', 'NewInvestor2',
      ]) {
        const extra = await chain.getAgent(name) // create agent
        wallets.push([extra.address, 1000000])
        recipients[name] = {agent: extra, address: extra.address}
      }
    })

    await task(`create ${wallets.length} recipient accounts`, async report => {
      const tx = await agent.sendMany(wallets, 'create recipient accounts')
      report(tx.transactionHash)
    })

    return recipients

  }

  async function deploy (builder, schedule, recipients) {
    const workspace  = dirname(fileURLToPath(here))
    builder.configure({
      buildImage: 'enigmampc/secret-contract-optimizer:latest',
      buildUser:  'root',
      outputDir:  resolve(workspace, 'artifacts'), })
    const binaries  = await build({ task, builder })
    const receipts  = await upload({ task, builder, binaries })
    const timestamp = String(+ new Date())
    const contracts = await initialize({ task, agent, receipts, inits: {
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
             , initMsg: { pool:      'MintingPool'
                        , account:   'RPT'
                        , config:    [[recipients.TokenPair1.address, "2500000000000000000000"]]}} } })
    return contracts
  }

  async function verify (agent, recipients, contracts, schedule) {
    const { TOKEN, MGMT, RPT } = contracts

    await task('set null viewing keys', async () => {
      const vk = "entropy"
      return (await Promise.all(
        Object.values(recipients).map(({agent})=>
          TOKEN.setViewingKey(agent, "entropy")
        )
      )).map(({tx})=>tx)
    })

    await task('make mgmt owner of token', async () => {
      return await MGMT.acquire(TOKEN) // TODO auto-acquire on init
    })

    await task('point RPT account in schedule to RPT contract', async () => {
      schedule
        .pools.filter(x=>x.name==='MintingPool')[0]
        .accounts.filter(x=>x.name==='RPT')[0]
        .address = RPT.address
      const {transactionHash: tx} = await MGMT.configure(schedule)
      return [tx]
    })

    let launched
    await task('launch the vesting', async () => {
      const result = await MGMT.launch()
      launched = 1000 * Number(result.logs[0].events[1].attributes[1].value)
      return [result.transactionHash]
    })

    await task.done()

    while (true) {
      await agent.nextBlock
      for (const [name, recipient] of Object.entries(recipients)) {
        const now = new Date()
        const elapsed = now - launched
        const { unlocked, claimed } = await MGMT.progress(recipient.address, now)
        console.log(`${now.toISOString()} (${elapsed} after start)`, name, unlocked)
      }
    }
  }

}
