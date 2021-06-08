import { randomBytes } from 'crypto'

import SNIP20 from './SNIP20.js'
import Rewards from './Rewards.js'

import {abs} from '../ops/root.js'
import RewardsContracts from '../ops/RewardsContracts.js'
import {SecretNetwork} from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'

describe('Rewards', () => {

  const state = {
    node:          null,
    network:       null,
    tokenCodeId:   null,
    rewardsCodeId: null,
    agent:         null,
    token:         null,
    rewards:       null
  }

  before(setupAll(state))

  after(cleanupAll(state))

  beforeEach(setupEach(state))

  it('can lock and return tokens', async function () {
    this.timeout(60000)
    const {agent, token, rewards}=state

    await token.mint(agent, 100)
    assert(await token.balance(agent)   === 100)
    assert(await token.balance(rewards) ===   0)

    await rewards.lock(agent, 50)
    assert(await token.balance(agent)   ===  50)
    assert(await token.balance(rewards) ===  50)

    await rewards.unlock(agent, 50)
    assert(await token.balance(agent)   === 100)
    assert(await token.balance(rewards) ===   0)
  })

  it('can process claims', async function () {
    this.timeout(60000)
    const {agent, token, rewards}=state

    await rewards.claim(agent)
    expect(await token.balance(agent)   ===   0)

    await token.mint(agent, 100)
    assert(await token.balance(agent)   === 100)
    assert(await token.balance(rewards) ===   0)
    await rewards.lock(agent, 100)

    await token.interval()
    await rewards.claim(agent)
    expect(await token.balance(agent)   ===   2)

    await token.interval()
    await rewards.claim(agent)
    expect(await token.balance(agent)   ===   4)

    await rewards.unlock(agent, 50)
    expect(await token.balance(agent)   ===  54)

    await token.interval()
    await rewards.claim(agent)
    expect(await token.balance(agent)   ===  55)

    await rewards.unlock(agent, 50)
    expect(await token.balance(agent)   === 105)

    await token.interval()
    await rewards.claim(agent)
    expect(await token.balance(agent)   === 105)
  })

  it('can be configured', () => {})

  it('can be administrated', () => {})

  it('is protected by a viewing key', () => {})
})

function setupAll (state = {}) {
  return async function () {
    this.timeout(60000)
    // before each test run, compile fresh versions of the contracts
    const {TOKEN: tokenBinary, REWARDS: rewardsBinary} = await RewardsContracts.build({
      workspace: abs(),
      parallel: false
    })
    // run a clean localnet
    const {node, network, agent, builder} = await SecretNetwork.localnet({
      stateBase: abs('artifacts')
    })
    await agent.nextBlock
    Object.assign(state, { node, network, admin: agent, builder })
    // and upload them to it
    const {codeId: tokenCodeId}   = await builder.uploadCached(tokenBinary)
    const {codeId: rewardsCodeId} = await builder.uploadCached(rewardsBinary)
    Object.assign(state, { tokenCodeId, rewardsCodeId })
  }
}

function setupEach (state = {}) {
  return async function () {
    this.timeout(60000)
    //state.agent = await state.network.getAgent()
    console.log('\ndeploying instance of token')
    state.token = await SNIP20.init({
      agent:   state.admin,
      label:   'token',
      codeId:  state.tokenCodeId,
      initMsg: RewardsContracts.contracts.TOKEN.initMsg
    })
    console.log('\ndeploying instance of rewards')
    const reward_token = { address: state.token.address, code_hash: state.token.codeHash }
    const initMsg = {
      ...RewardsContracts.contracts.REWARDS.initMsg,
      admin: state.admin.address,
      reward_token,
      entropy:   randomBytes(36).toString('base64'),
      prng_seed: randomBytes(36).toString('base64')
    }
    console.log({initMsg})
    state.rewards = await Rewards.init({
      agent:   state.admin,
      label:   'rewards',
      codeId:  state.rewardsCodeId,
      initMsg
    })
    console.log('ready')
  }
}

function cleanupAll (state = {}) {
  return async function () {
    await state.node.terminate()
  }
}
