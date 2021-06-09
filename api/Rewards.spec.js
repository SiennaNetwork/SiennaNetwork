import { randomBytes } from 'crypto'

import SNIP20 from './SNIP20.js'
import Rewards from './Rewards.js'

import {abs} from '../ops/root.js'
import RewardsContracts from '../ops/RewardsContracts.js'
import {SecretNetwork} from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'

import {assert} from 'chai'

const contracts = new RewardsContracts()

describe('Rewards', () => {

  const state = {
    node:          null,
    network:       null,
    tokenCodeId:   null,
    rewardsCodeId: null,
    admin:         null,
    token:         null,
    rewards:       null
  }

  before(setupAll)

  after(cleanupAll)

  beforeEach(setupEach)

  it('can lock and return tokens', async function () {
    this.timeout(60000)
    const {admin, token, rewards}=state

    await token.mint(admin, 100)
    assert(await token.balance(admin)   === 100)
    assert(await token.balance(rewards) ===   0)

    await rewards.lock(admin, 50)
    assert(await token.balance(admin)   ===  50)
    assert(await token.balance(rewards) ===  50)

    await rewards.unlock(admin, 50)
    assert(await token.balance(admin)   === 100)
    assert(await token.balance(rewards) ===   0)
  })

  it('can process claims', async function () {
    this.timeout(60000)
    const {admin, token, rewards}=state

    await rewards.claim(admin)
    expect(await token.balance(admin)   ===   0)

    await token.mint(admin, 100)
    assert(await token.balance(admin)   === 100)
    assert(await token.balance(rewards) ===   0)
    await rewards.lock(admin, 100)

    await token.interval()
    await rewards.claim(admin)
    expect(await token.balance(admin)   ===   2)

    await token.interval()
    await rewards.claim(admin)
    expect(await token.balance(admin)   ===   4)

    await rewards.unlock(admin, 50)
    expect(await token.balance(admin)   ===  54)

    await token.interval()
    await rewards.claim(admin)
    expect(await token.balance(admin)   ===  55)

    await rewards.unlock(admin, 50)
    expect(await token.balance(admin)   === 105)

    await token.interval()
    await rewards.claim(admin)
    expect(await token.balance(admin)   === 105)
  })

  it('can be configured', () => {})

  it('can be administrated', () => {})

  it('is protected by a viewing key', () => {})

  async function setupAll () {
    this.timeout(60000)
    // before each test run, compile fresh versions of the contracts
    const {TOKEN: tokenBinary, REWARDS: rewardsBinary} = await contracts.build({
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

  async function setupEach () {
    this.timeout(60000)
    //state.agent = await state.network.getAgent()
    console.log('\ndeploying instance of token')
    state.token = await SNIP20.init({
      agent:   state.admin,
      label:   'token',
      codeId:  state.tokenCodeId,
      initMsg: contracts.contracts.TOKEN.initMsg
    })
    console.log('\ndeploying instance of rewards')
    const reward_token = { address: state.token.address, code_hash: state.token.codeHash }
    const initMsg = {
      ...contracts.contracts.REWARDS.initMsg,
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

  async function cleanupAll () {
    await state.node.terminate()
  }

})
