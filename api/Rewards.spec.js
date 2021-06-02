import SNIP20 from './SNIP20.js'
import Rewards from './Rewards.js'

import {CONTRACTS} from '../cli/ops.js'
import {abs} from '../cli/root.js'
import {SecretNetwork, SecretNetworkOps as Ops} from '@hackbg/fadroma'

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
  beforeEach(setupEach(state))
  after(cleanupAll(state))

  it('can lock and return tokens', async () => {
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

  it('can process claims', async () => {
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
    const {token, rewards} = await Ops.build({
      token:   { name: 'token',   crate: 'snip20-reference-impl' },
      rewards: { name: 'rewards', crate: 'sienna-rewards' }
    }, {
      workspace: abs(),
      parallel: false
    })
    // run a clean localnet
    const {node, network, builder} = await SecretNetwork.localnet({
      stateBase: abs('artifacts')
    })
    Object.assign(state, { node, network, bulder })
    // and upload them to it
    ;([{codeId: tokenCodeId}, {codeId: rewardsCodeId}] = await Promise.all([
      builder.uploadCached(token),
      builder.uploadCached(rewards)
    ]))
    this.timeout(15000)
  }
}

function setupEach (state = {}) {
  return async function () {
    state.agent = await network.getAgent()
    state.token = await SNIP20.init({
      agent:   state.agent,
      label:   'token',
      codeId:  tokenCodeId,
      initMsg: CONTRACTS.TOKEN.initMsg
    })
    state.rewards = await Rewards.init({
      agent:   state.agent,
      label:   'rewards',
      codeId:  rewardsCodeId,
      initMsg: { reward_token: token.address }
    })
  }
}

function cleanupAll (state = {}) {
  return async function () {
    await state.node.remove()
  }
}

func
