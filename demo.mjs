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
import { writeFile } from 'fs/promises'
import { fileURLToPath } from 'url'
import { resolve, dirname } from 'path'
import { backOff } from "exponential-backoff";
import { loadJSON, SecretNetwork } from '@hackbg/fadroma'
import SNIP20Contract from '@hackbg/snip20'
import MGMTContract from '@hackbg/mgmt'
import RPTContract from '@hackbg/rpt'

const say       = x => console.log(x)
const here      = import.meta.url
const workspace = dirname(fileURLToPath(here))
const schedule  = loadJSON('./settings/schedule.json', here)

const localnet = SecretNetwork.connect(here).then(async ({chain, agent, builder})=>{

  const rows = [
    ['time', 'description', 'took', 'gas', 'profiling overhead'],
    ['---', '---', '---', '---', '---']
  ]

  const recipients = await prepare(chain, agent, schedule)
  const contracts  = await deploy(builder, schedule, recipients)
  const result     = await verify(agent, recipients, contracts, schedule)
  say(result)

  async function prepare (chain, agent, schedule) {

    const wallets    = []
        , recipients = {}

    await step('shorten schedule and replace placeholders with test accounts',
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

    await step('create extra test accounts for reallocation tests', async () => {
      for (let name of [ // extra accounts for reconfigurations
        'TokenPair1', 'TokenPair2', 'TokenPair3',
        'NewAdvisor', 'NewInvestor1', 'NewInvestor2',
      ]) {
        const extra = await chain.getAgent(name) // create agent
        wallets.push([extra.address, 1000000])
        recipients[name] = {agent: extra, address: extra.address}
      }
    })

    await step('preseed all test accounts', async () => {
      const {transactionHash} = await agent.sendMany(wallets, 'create recipient wallets')
      return [transactionHash]
      //agent.API.searchTX
    })

    return recipients

  }

  async function deploy (builder, schedule, recipients) {

    builder = builder.configure({
      buildImage: 'hackbg/secret-contract-optimizer:latest',
      buildUser:  'root',
      outputDir:  resolve(workspace, 'artifacts'),
    })

    const contracts = {}
    const initTXs = {}

    await step('build and deploy token', async () => {
      const label = +new Date()+'-snip20'
      const crate = 'snip20-reference-impl'
      const {codeId, compressedSize} = await builder.getUploadReceipt(workspace, crate)
      console.log(`⚖️  compressed size ${compressedSize} bytes`)
      contracts.TOKEN = new SNIP20Contract({ agent, codeId, label, initMsg: {
        name:      "Sienna",
        symbol:    "SIENNA",
        decimals:  18,
        admin:     builder.agent.address,
        prng_seed: "insecure",
        config:    { public_total_supply: true }
      } })
      return [initTXs.TOKEN = await contracts.TOKEN.init()]
    })

    await step('build and deploy mgmt', async () => {
      const label = +new Date()+'-mgmt'
      const crate = 'sienna-mgmt'
      const {codeId, compressedSize} = await builder.getUploadReceipt(workspace, crate)
      console.log(`⚖️  compressed size ${compressedSize} bytes`)
      contracts.MGMT = new MGMTContract({ agent, codeId, label, initMsg: {
        token:     [contracts.TOKEN.address, contracts.TOKEN.codeHash],
        schedule
      } })
      return [initTXs.MGMT = await contracts.MGMT.init()]
    })

    await step('build and deploy rpt', async () => {
      const label = +new Date()+'-rpt'
      const crate = 'sienna-rpt'
      const {codeId, compressedSize} = await builder.getUploadReceipt(workspace, crate)
      console.log(`⚖️  compressed size ${compressedSize} bytes`)
      contracts.RPT = new RPTContract({ agent, codeId, label, initMsg: {
        token:     [contracts.TOKEN.address, contracts.TOKEN.codeHash],
        mgmt:      [contracts.MGMT.address,  contracts.MGMT.codeHash],
        pool:      'MintingPool',
        account:   'RPT',
        config:    [ [recipients.TokenPair1.address, "2500000000000000000000"]]
      } })
      return [initTXs.RPT = await contracts.RPT.init()]
    })

    return contracts

  }

  async function verify (agent, recipients, contracts, schedule) {
    const { TOKEN, MGMT, RPT } = contracts

    await step('set null viewing keys', async () => {
      const vk = "entropy"
      return (await Promise.all(
        Object.values(recipients).map(({agent})=>
          TOKEN.setViewingKey(agent, "entropy")
        )
      )).map(({tx})=>tx)
    })

    await step('make mgmt owner of token', async () => {
      return await MGMT.acquire(TOKEN) // TODO auto-acquire on init
    })

    await step('point RPT account in schedule to RPT contract', async () => {
      schedule
        .pools.filter(x=>x.name==='MintingPool')[0]
        .accounts.filter(x=>x.name==='RPT')[0]
        .address = RPT.address
      const {transactionHash: tx} = await MGMT.configure(schedule)
      return [tx]
    })

    let launched
    await step('launch the vesting', async () => {
      const result = await MGMT.launch()
      launched = 1000 * Number(result.logs[0].events[1].attributes[1].value)
      return [result.transactionHash]
    })

    writeFile(
      resolve(workspace, 'artifacts', 'gas-report.md'),
      rows.filter(Boolean).map(step=>`| `+step.join(' | ')+`| `).join('\n'),
      'utf8'
    )

    while (true) {
      await agent.waitForNextBlock()
      const now = new Date()
      const elapsed = now - launched
      console.log({launched, elapsed})
      await Promise.all(Object.values(recipients).map(({address})=>
        MGMT.progress(address, now)))
    }
  }

  async function step (description, callback) {
    const t1 = new Date()
    say(`\n${description}`)
    const txHashes = await Promise.resolve(callback())
    const t2 = new Date()
    say(`⏱️  took ${t2-t1}msec`)
    if (txHashes) {
      const txs = await Promise.all(txHashes.map(id=>
        backOff(async ()=>{
          try {
            return await agent.API.restClient.get(`/txs/${id}`)
          } catch (e) {
            throw e
          }
        })))
      const totalGasUsed = txs.map(x=>Number(x.gas_used)).reduce((x,y)=>x+y, 0)
      const t3 = new Date()
      say(`⛽ cost ${totalGasUsed} gas`)
      say(`🔍 gas check took ${t3-t2}msec`)
      rows.push([t1.toISOString(), description, t2-t1, totalGasUsed, t3-t2])
    } else {
      rows.push([t1.toISOString(), description, t2-t1])
    }
  }

})
