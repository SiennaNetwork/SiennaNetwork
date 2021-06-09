import assert from 'assert'
import { randomBytes } from 'crypto'

import {SecretNetwork} from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'

import SNIP20 from './SNIP20.js'
import Rewards from './Rewards.js'

import {abs} from '../ops/root.js'
import RewardsContracts from '../ops/RewardsContracts.js'

const ensemble = new RewardsContracts()

describe('Rewards', () => {

  const context = {
    node:          null,
    network:       null,
    tokenCodeId:   null,
    rewardsCodeId: null,
    admin:         null,
    viewkey:       null,
    token:         null,
    rewards:       null
  }

  const balance = (
    address = context.admin.address,
    key     = context.viewkey
  ) => context.token.balance(address, key)

  before(setupAll)

  after(cleanupAll)

  beforeEach(setupEach)

  it('can lock and return tokens', async function () {
    this.timeout(60000)
    const {admin, token, rewards}=context

    await token.mint(100)
    assert.equal(await balance(), 100)
    //assert(await token.balance(rewards) ===   0)

    await token.increaseAllowance(100, rewards.address)
    await rewards.lock(50, token.address)
    assert.equal(await balance(), 50)
    //assert(await token.balance(rewards) ===  50)
    await token.decreaseAllowance(100, rewards.address)

    await rewards.retrieve(50, token.address)
    assert.equal(await balance(), 100)
    //assert(await token.balance(rewards) ===   0)
  })

  it('can process claims', async function () {
    this.timeout(60000)
    const {token, rewards, admin, viewkey}=context

    await rewards.claim(admin)
    assert.equal(await balance(), 0)

    await token.mint(admin, 100)
    assert(await balance(), 100)
    //assert(await token.balance(rewards) ===   0)
    await rewards.lock(100, token.address)

    await token.interval()
    await rewards.claim(admin)
    assert.equal(await balance(), 2)

    await token.interval()
    await rewards.claim(admin)
    assert.equal(await balance(), 4)

    await rewards.retrieve(50, token.address)
    assert.equal(await balance(), 54)

    await token.interval()
    await rewards.claim(admin)
    assert.equal(await balance(), 55)

    await rewards.retrieve(50, token.address)
    assert.equal(await balance(), 105)

    await token.interval()
    await rewards.claim(admin)
    assert.equal(await balance(), 105)
  })

  it('can be configured', () => {})

  it('can be administrated', () => {})

  it('is protected by a viewing key', () => {})

  async function setupAll () {
    this.timeout(60000)

    // before each test run, compile fresh versions of the contracts
    const {TOKEN: tokenBinary, REWARDS: rewardsBinary} = await ensemble.build({
      workspace: abs(),
      parallel: false
    })

    // run a clean localnet
    const {node, network, agent, builder} = await SecretNetwork.localnet({
      stateBase: abs('artifacts')
    })
    await agent.nextBlock
    Object.assign(context, { node, network, admin: agent, builder })

    // and upload them to it
    const {codeId: tokenCodeId}   = await builder.uploadCached(tokenBinary)
    const {codeId: rewardsCodeId} = await builder.uploadCached(rewardsBinary)
    Object.assign(context, { tokenCodeId, rewardsCodeId })
  }

  async function setupEach () {
    this.timeout(60000)

    // deploy token
    context.token = await context.admin.instantiate(new SNIP20({
      label: 'token',
      codeId: context.tokenCodeId,
      initMsg: ensemble.contracts.TOKEN.initMsg
    }))
    const reward_token = context.token.reference

    // prepare rewards manager config
    const initMsg = {
      ...ensemble.contracts.REWARDS.initMsg,
      admin:     context.admin.address,
      entropy:   '',//randomBytes(36).toString('base64'),
      prng_seed: '',//randomBytes(36).toString('base64'),
      reward_token,
    }
    initMsg.reward_pools[0].lp_token = reward_token

    // deploy rewards manager
    context.rewards = await context.admin.instantiate(new Rewards({
      label: 'rewards',
      codeId: context.rewardsCodeId,
      initMsg
    }))

    // create viewing key for admin balance
    context.viewkey = (await context.token.createViewingKey(context.admin)).key
  }

  async function cleanupAll () {
    await context.node.terminate()
  }

})
