/// # Sienna Rewards v3 Benchmark
///
/// ## Dependencies


import {
  Agent,
  ContractUpload,
  Scrt,
  ScrtGas,
  Console,
  resolve, dirname, fileURLToPath
} from '@hackbg/fadroma'

const console = Console('[@sienna/amm/benchmarkRewards]')

import { RewardsContract, SiennaSNIP20Contract, LPTokenContract } from '@sienna/api'

/// ## Overview


export async function benchmarkRewards () {
  const { agents } = await setupNetwork(20)
  const { REWARDS, LPTOKEN } = await setupContracts(agents[0])
  const { actions } = setupActions(REWARDS, LPTOKEN)
  await runBenchmark(actions, agents)
}


/// ## Network setup


const projectRoot = resolve(dirname(fileURLToPath(import.meta.url))),
      abs = (...args: Array<string>) => resolve(projectRoot, ...args)

async function setupNetwork (numberOfAgents: number) {
  console.info(`Setting up devnet with ${numberOfAgents} agents`)

  const identities = ["ADMIN", ...[...Array(numberOfAgents)].map((_,i)=>`Agent${i}`)]

  const network = await Scrt.devnet_1_2({ identities }).ready

  const agents = await Promise.all(identities.map(name=>network.getAgent({
    name,
    mnemonic: network.node.genesisAccount(name).mnemonic
  })))

  for (const agent of agents) {
    agent.API.fees = {
      upload: new ScrtGas(10000000),
      init:   new ScrtGas(10000000),
      exec:   new ScrtGas(10000000),
      send:   new ScrtGas(10000000)
    }
  }

  return { network, agents }
}


/// ## Contract setup



async function setupContracts (admin: Agent) {

  const prefix  = `RewardsBenchmark_${+new Date()}`
  const LPTOKEN = new LPTokenContract({ prefix, admin, name: 'BENCHMARK' })
  const SIENNA  = new SiennaSNIP20Contract({ prefix, admin })
  const REWARDS = new RewardsContract({
    prefix, admin,
    lpToken: LPTOKEN, rewardToken: SIENNA,
    bonding: 0
  })
  const contracts = [SIENNA, LPTOKEN, REWARDS]
  await buildAndUpload(contracts)

  console.info(`Instantiating contracts`)
  for (const contract of contracts) {
    await contract.instantiate()
    await admin.nextBlock
  }
  await LPTOKEN.setMinters([admin.address], admin)

  return { REWARDS, LPTOKEN, SIENNA }

  async function buildAndUpload (contracts: Array<ContractUpload>) {
    console.info(`Building contracts`)
    await Promise.all(contracts.map(contract=>contract.build()))
    console.info(`Uploading contracts`)
    for (const contract of contracts) await contract.upload()
  }
}


/// # Benchmark actions


type Action      = (agent: Agent) => Promise<void>
type Transaction = [string, Agent, { transactionHash: string }]


function setupActions (REWARDS: RewardsContract, LPTOKEN: LPTokenContract) {
  console.info(`Preparing benchmark`)

  const transactionLog: Array<Transaction> = []

  const actions = [
    async (recipient: Agent) => {
      console.log(`----- ${recipient.name}: lock 100`)
      await LPTOKEN.mint("100", undefined, recipient.address)
      const result = await REWARDS.TX(recipient).deposit("100")
      transactionLog.push(['lock', recipient, result])
    },
    async (recipient: Agent) => {
      console.log(`----- ${recipient.name}: retrieve 5`)
      const result = await REWARDS.TX(recipient).withdraw("5")
      transactionLog.push(['retrieve', recipient, result])
    },
    async (recipient: Agent) => {
      console.log(`----- ${recipient.name}: claim`)
      const result = await REWARDS.TX(recipient).claim()
      transactionLog.push(['claim', recipient, result])
    },
  ]

  return { actions, transactionLog }
}


async function runBenchmark (actions: Array<Action>, agents: Array<Agent>) {
  console.info(`Running benchmark`)

  for (let i = 0; i < 1000000; i++) {
    const action    = pickRandom(actions)
    const recipient = pickRandom(agents)
    try {
      await action(recipient)
    } catch (e) {
      console.warn(e)
    }
    console.info(`next`)
  }

  function pickRandom <T> (arr: Array<T>) {
    return arr[Math.floor(Math.random()*arr.length)]
  }
}


//import { Scrt, gas, taskmaster, randomHex }  from "@hackbg/fadroma"

//import { abs } from '../ops/lib/index.js'

//import SNIP20  from './SNIP20.js'
//import Rewards from './Rewards.js'

//describe("Rewards", () => {

  //const fees = { upload: gas(10000000)
               //, init:   gas(10000000)
               //, exec:   gas(10000000)
               //, send:   gas(10000000) }

  //const context = {}

  //before(async function setupAll () {
    //this.timeout(600000)
    //const T0 = + new Date()

    //// connect to a devnet with a large number of predefined agents
    //const numberOfAgents = 20
    //const agentNames = [...Array(numberOfAgents)].map((_,i)=>`Agent${i}`)
    //const devnet = Scrt.devnet({
      //stateBase:       abs("artifacts"),
      //genesisAccounts: ["ADMIN", ...agentNames]
    //})
    //const {node, network, builder, agent} = await devnet.connect()
    //const agents = await Promise.all(agentNames.map(name=>
      //network.getAgent(name, { mnemonic: node.genesisAccount(name).mnemonic })))
    //console.log({agents})
    //agent.API.fees = fees

    //const T1 = + new Date()
    //console.debug(`connecting took ${T1 - T0}msec`)

    //// build the contracts
    //const workspace = abs()
    //const [ tokenBinary, poolBinary ] = await Promise.all([
      //builder.build({workspace, crate: 'amm-snip20'    }),
      //builder.build({workspace, crate: 'sienna-rewards'}), ])

    //const T2 = + new Date()
    //console.debug(`building took ${T2 - T1}msec`)

    //// upload the contracts
    //const {
      //codeId:           tokenCodeId,
      //originalChecksum: tokenCodeHash,
    //} = await builder.uploadCached(tokenBinary)
    //await agent.nextBlock

    //const {
      //codeId:           poolCodeId,
      //originalChecksum: poolCodeHash
    //} = await builder.uploadCached(poolBinary)
    //await agent.nextBlock

    //const T3 = + new Date()
    //console.debug(`uploading took ${T3 - T2}msec`)
    //console.debug(`total preparation time: ${T3 - T0}msec`)

    //Object.assign(context, {
      //node, network, builder, agent, agents,
      //token: { id: tokenCodeId, code_hash: tokenCodeHash },
      //pool:  { id: poolCodeId,  code_hash: poolCodeHash  },
    //})
  //})

  //beforeEach(async function setupEach () {})

  //it("uses a reasonable amount of gas when processing claims", async function () {
    //this.timeout(600000)

    //const T0 = + new Date()

    //const header = [ 'time', 'info', 'time (msec)', 'gas (uSCRT)', 'overhead (msec)' ]
        //, output = abs('artifacts', 'rewards-benchmark.md')
        //, task   = taskmaster({ header, output, agent: context.agent })

    //console.debug('init reward token:')
    //const rewardToken = await context.agent.instantiate(new SNIP20({
      //codeId: context.token.id,
      //label:  'RewardToken',
      //initMsg: { prng_seed: randomHex
               //, name:     "RewardToken"
               //, symbol:   "REWARD"
               //, decimals: 18
               //, config:
                 //{ public_total_supply: true
                 //, enable_deposit: true
                 //, enable_redeem: true
                 //, enable_mint: true
                 //, enable_burn: true } } }))

    //console.debug('init reward pool:')
    //const rewardPool = await context.agent.instantiate(new Rewards({
      //codeId: context.pool.id,
      //label: 'RewardPool',
      //initMsg: { reward_token: rewardToken.reference
               //, viewing_key:  ""
               //, threshold:    24} }))

    //console.debug('mint reward budget:')
    //await rewardToken.mint("500000000000000000000", undefined, rewardPool.address)

    //console.debug('init asset token:')
    //const lpToken = await context.agent.instantiate(new SNIP20({
      //codeId: context.token.id,
      //label:  'LPToken',
      //initMsg: { prng_seed: randomHex
               //, name:     "LPToken"
               //, symbol:   "LPTOKE"
               //, decimals: 18
               //, initial_balances:
                 //[ { address: context.agent.address, amount: '1000000000000000000' }
                 //, ...context.agents.map(user=>({
                     //address: user.address,
                     //amount: '1000000000000000000'
                   //}))]
               //, initial_allowances:
                 //[ { owner: context.agent.address
                   //, spender: rewardPool.address
                   //, amount: '1000000000000000000' }
                 //, ...context.agents.map(user=>({
                   //owner:   user.address,
                   //spender: rewardPool.address,
                   //amount: '1000000000000000000'
                 //})) ]
               //, config:
                 //{ public_total_supply: true
                 //, enable_deposit: true
                 //, enable_redeem: true
                 //, enable_mint: true
                 //, enable_burn: true } } }))

    //console.debug('set asset token in rewards pool:')
    //await rewardPool.setLPToken(lpToken.reference.address, lpToken.reference.code_hash)

    //const T1 = + new Date()
    //console.info(`instantiation took ${T1 - T0}msec`)

    //const getRandomAmount = () => BigInt(String(Math.floor(Math.random()*1000000)))
    //await rewardPool.lock(getRandomAmount()).catch(console.error)
    //await rewardPool.claim().catch(console.error)
    //await rewardPool.retrieve(getRandomAmount()).catch(console.error)

    //const T2 = + new Date()

    //// K*N times have a random user do a random operation (lock/retrieve random amount or claim)
    //const transactions = [
      //// type,
      //// { transactionHash }
    //]

    //const known = new Set()

    //;(async function checkGas () {
      //const txGasCheckResults = await Promise.all(transactions.map(async ([label, recipient, tx])=>{
        //const {transactionHash:txhash} = tx
        //const txdata = await context.agent.API.restClient.get(`/txs/${txhash}`) || {}
        //return [txhash, label, recipient.name, txdata.gas_used, known.size]
      //}))
      //console.table(txGasCheckResults)
      //setTimeout(checkGas, 5000)
    //})()

    //const actions = [
      //async recipient => {
        //known.add(recipient)
        //console.log(`----- ${recipient.name}: lock 100`)
        //transactions.push([
          //'lock',
          //recipient,
          //(await rewardPool.lock("100", recipient.address))])},
      //async recipient => {
        //console.log(`----- ${recipient.name}: retrieve 5`)
        //transactions.push([
          //'retrieve',
          //recipient,
          //(await rewardPool.retrieve("5", recipient.address))])},
      //async recipient => {
        //console.log(`----- ${recipient.name}: claim`)
        //transactions.push([
          //'claim',
          //recipient,
          //(await rewardPool.claim(recipient.address))])},
      ////recipient => task(`${recipient}: lock 100`,
        ////() => rewardPool.lock("100", recipient)),
      ////recipient => task(`${recipient}: retrieve 5`,
        ////() => rewardPool.retrieve("5", recipient)),
      ////recipient => task(`${recipient}: claim`,
        ////() => rewardPool.claim(recipient)),
      ////() => {} // skip turn
    //]

    //console.log('--------- PRELOCK BEGINS -----------')

    //for (const agent of context.agents) {
      //await actions[0](agent).catch(console.error)
    //}

    //console.log('---------PRELOCK COMPLETE-----------')

    //const pickRandom = arr => arr[Math.floor(Math.random()*arr.length)]
    //for (let i = 0; i < 1000000; i++) {
      //const action    = pickRandom(actions)
      //const recipient = pickRandom(context.agents)
      //// track average and maximum gas cost
      //try {
        //console.debug(await action(recipient))
      //} catch (e) {
        //console.warn(e)
      //}
    //}

    //const T3 = + new Date()
    //console.info(`benchmark took ${T3 - T2}msec`)
    
  //})

  //after(async function cleanupAll () {})

//})

//// scratchpad TODO put this in a gist or the kb or something
    //// make N accounts and send them scrt
    ////const N = 10
    ////const recipients = await Promise.all(
      ////[...Array(N)].map(()=>{
        ////const name = `Agent ${N}`
        ////console.debug(`create agent ${name}`)
        ////return context.network.getAgent(name, { js: true })}))
    ////await context.agent.nextBlock
    ////await fundAgents({           // send 'em scrt for gas fees to create 'em on-chain 
      ////connection: null,          // don't autoconnect
      ////agent:      context.agent, // from the admin's genesis balance
      ////recipients: recipients.reduce((recipients, agent)=>[>ugh<]
        ////Object.assign(recipients, {[agent.address]:{agent, address:agent.address}}), {})})
    //// mint 'em random amount of ASSET
    //////await Promise.all(recipients.map(recipient=>{
      //////const amount = getRandomAmount()
      //////console.debug(`mint ${amount} to ${recipient.name}`)
      //////return asset.mint(amount, asset.agent, recipient.address)
    //////}))
    ////for (const recipient of recipients) {
      ////const amount = getRandomAmount()
      ////console.debug(`mint ${amount} to ${recipient.name}`)
      ////await asset.increaseAllowance(amount, rewardPool.address, recipient)
      ////await asset.mint(amount, asset.agent, recipient.address)
