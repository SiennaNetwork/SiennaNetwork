import { randomBytes }   from 'crypto'
import { SecretNetwork } from '@fadroma/scrt-agent'
import { gas }           from '@fadroma/scrt-agent/gas.js'
import { bignum }        from '@fadroma/utilities'
//import fundAgents        from '@fadroma/scrt-agent/fund.js'
import { abs }           from '../ops/lib/index.js'
import SNIP20            from './SNIP20.js'
import RewardsBenchmark  from './RewardsBenchmark.js'

describe("RewardsBenchmark", () => {

  const fees = { upload: gas(10000000)
               , init:   gas(10000000)
               , exec:   gas(10000000)
               , send:   gas(10000000) }

  const context = {}

  before(async function setupAll () {
    this.timeout(600000)
    const T0 = + new Date()

    // connect to a localnet with a large number of predefined agents
    const numberOfAgents = 20
    const agentNames = [...Array(numberOfAgents)].map((_,i)=>`Agent${i}`)
    const localnet = SecretNetwork.localnet({
      stateBase:       abs("artifacts"),
      genesisAccounts: ["ADMIN", ...agentNames]
    })
    const {node, network, builder, agent} = await localnet.connect()
    const agents = await Promise.all(agentNames.map(name=>
      network.getAgent(name, { mnemonic: node.genesisAccount(name).mnemonic })))
    console.log({agents})
    agent.API.fees = fees

    const T1 = + new Date()
    console.debug(`connecting took ${T1 - T0}msec`)

    // build the contracts
    const workspace = abs()
    const [ tokenBinary, poolBinary ] = await Promise.all([
      builder.build({workspace, crate: 'amm-snip20'              }),
      builder.build({workspace, crate: 'sienna-rewards-benchmark'}), ])

    const T2 = + new Date()
    console.debug(`building took ${T2 - T1}msec`)

    // upload the contracts
    const {
      codeId:           tokenCodeId,
      originalChecksum: tokenCodeHash,
    } = await builder.uploadCached(tokenBinary)
    await agent.nextBlock

    const {
      codeId:           poolCodeId,
      originalChecksum: poolCodeHash
    } = await builder.uploadCached(poolBinary)
    await agent.nextBlock

    const T3 = + new Date()
    console.debug(`uploading took ${T3 - T2}msec`)
    console.debug(`total preparation time: ${T3 - T0}msec`)

    Object.assign(context, {
      node, network, builder, agent, agents,
      token: { id: tokenCodeId, code_hash: tokenCodeHash },
      pool:  { id: poolCodeId,  code_hash: poolCodeHash  },
    })
  })

  beforeEach(async function setupEach () {})

  it("uses a reasonable amount of gas when processing claims", async function () {
    this.timeout(600000)

    const T0 = + new Date()

    console.debug('init reward token:')
    const rewardToken = await context.agent.instantiate(new SNIP20({
      codeId: context.token.id,
      label:  'RewardToken',
      initMsg: { prng_seed: randomBytes(36).toString('hex')
               , name:     "RewardToken"
               , symbol:   "REWARD"
               , decimals: 18
               , config:
                 { public_total_supply: true
                 , enable_deposit: true
                 , enable_redeem: true
                 , enable_mint: true
                 , enable_burn: true } } }))

    console.debug('init reward pool:')
    const rewardPool = await context.agent.instantiate(new RewardsBenchmark({
      codeId: context.pool.id,
      label: 'RewardPool',
      initMsg: { rewarded_token: rewardToken.reference
               , viewing_key:    "" } }))

    console.debug('mint reward budget:')
    await rewardToken.mint("500000000000000000000", undefined, rewardPool.address)

    console.debug('init asset token:')
    const lpToken = await context.agent.instantiate(new SNIP20({
      codeId: context.token.id,
      label:  'LPToken',
      initMsg: { prng_seed: randomBytes(36).toString('hex')
               , name:     "LPToken"
               , symbol:   "LPTOKE"
               , decimals: 18
               , initial_balances:
                 [ { address: context.agent.address, amount: '1000000000000000000' }
                 , ...context.agents.map(user=>({
                     address: user.address,
                     amount: '1000000000000000000'
                   }))]
               , initial_allowances:
                 [ { owner: context.agent.address
                   , spender: rewardPool.address
                   , amount: '1000000000000000000' }
                 , ...context.agents.map(user=>({
                   owner:   user.address,
                   spender: rewardPool.address,
                   amount: '1000000000000000000'
                 })) ]
               , config:
                 { public_total_supply: true
                 , enable_deposit: true
                 , enable_redeem: true
                 , enable_mint: true
                 , enable_burn: true } } }))

    console.debug('set asset token in rewards pool:')
    await rewardPool.setProvidedToken(lpToken.reference.address, lpToken.reference.code_hash)

    const T1 = + new Date()
    console.info(`instantiation took ${T1 - T0}msec`)

    const getRandomAmount = () => bignum(String(Math.floor(Math.random()*1000000)))
    await rewardPool.lock(getRandomAmount()).catch(console.error)
    await rewardPool.claim().catch(console.error)
    await rewardPool.retrieve(getRandomAmount()).catch(console.error)

    const T2 = + new Date()

    // K*N times have a random user do a random operation (lock/retrieve random amount or claim)
    const actions = [
      recipient => {
        const amount = getRandomAmount()
        console.debug(`${recipient}: lock ${amount}`)
        return rewardPool.lock("100", recipient)
      },
      recipient => {
        const amount = getRandomAmount()
        console.debug(`${recipient.name}: retrieve ${amount}`)
        return rewardPool.retrieve("1", recipient)
      },
      recipient => {
        console.debug(`${recipient.name}: claim`)
        return rewardPool.claim(recipient)
      },
      () => {}
    ]
    const pickRandom = arr => arr[Math.floor(Math.random()*arr.length)]
    for (let i = 0; i < 1000000; i++) {
      const action    = pickRandom(actions)
      const recipient = pickRandom(context.agents)
      // track average and maximum gas cost
      try {
        console.debug(await action(recipient.address))
      } catch (e) {
        console.warn(e)
      }
    }

    const T3 = + new Date()
    console.info(`benchmark took ${T3 - T2}msec`)
    
  })

  after(async function cleanupAll () {})

})

// scratchpad TODO put this in a gist or the kb or something
    // make N accounts and send them scrt
    //const N = 10
    //const recipients = await Promise.all(
      //[...Array(N)].map(()=>{
        //const name = `Agent ${N}`
        //console.debug(`create agent ${name}`)
        //return context.network.getAgent(name, { js: true })}))
    //await context.agent.nextBlock
    //await fundAgents({           // send 'em scrt for gas fees to create 'em on-chain 
      //connection: null,          // don't autoconnect
      //agent:      context.agent, // from the admin's genesis balance
      //recipients: recipients.reduce((recipients, agent)=>[>ugh<]
        //Object.assign(recipients, {[agent.address]:{agent, address:agent.address}}), {})})
    // mint 'em random amount of ASSET
    ////await Promise.all(recipients.map(recipient=>{
      ////const amount = getRandomAmount()
      ////console.debug(`mint ${amount} to ${recipient.name}`)
      ////return asset.mint(amount, asset.agent, recipient.address)
    ////}))
    //for (const recipient of recipients) {
      //const amount = getRandomAmount()
      //console.debug(`mint ${amount} to ${recipient.name}`)
      //await asset.increaseAllowance(amount, rewardPool.address, recipient)
      //await asset.mint(amount, asset.agent, recipient.address)
    //}
