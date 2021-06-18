import assert from 'assert'
import { randomBytes } from 'crypto'

import {SecretNetwork} from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'
import {gas} from '@fadroma/scrt-agent/gas.js'

import SNIP20 from './SNIP20.js'
import Rewards from './Rewards.js'

import {abs} from '../ops/lib/index.js'
import RewardsContracts from '../ops/RewardsContracts.js'
import debug from 'debug'
const log = debug('out')

const fees = {
  upload: gas(20000000),
  init:   gas(1000000),
  exec:   gas(1000000),
  send:   gas( 500000),
}

const wait = (n) => new Promise((done) => setTimeout(done, n * 1000))

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
    address = '',
    key = '',
    token = 'token'
  ) => context[token].balance(address, key)

  const assertAdminBalance = async (amount = 0) => {
    assert.strictEqual(await balance(context.admin.address, context.viewkey, 'token'), String(amount))
  }

  const assertAliceBalance = async (amount = 0) => {
    assert.strictEqual(await balance(context.alice.address, context.aliceViewKey, 'token'), String(amount))
  }

  const assertAdminBalanceReward = async (amount = 0) => {
    assert.strictEqual(await balance(context.admin.address, context.viewkeyFoo, 'rewardToken'), String(amount))
  }

  const assertAliceBalanceReward = async (amount = 0) => {
    assert.strictEqual(await balance(context.alice.address, context.aliceViewKeyFoo, 'rewardToken'), String(amount))
  }

  before(setupAll)

  after(cleanupAll)

  beforeEach(setupEach)

  it('can lock and return tokens', async function () {
    this.timeout(60000)
    const {token, rewards}=context

    await token.mint(100)
    assertAdminBalance(100)

    await token.increaseAllowance(100, rewards.address)
    await rewards.lock(50, token.address)
    assertAdminBalance(50)
    
    await token.decreaseAllowance(100, rewards.address)

    await rewards.retrieve(50, token.address)
    assertAdminBalance(100)
  })

  it('can process claims', async function () {
    this.timeout(120000)
    const {
      token,
      rewardToken,
      rewards,
      admin,
    } = context
    // Mind lp token to admin and assert he has lp token balance and no reward token balance
    await token.mint(100000000)
    await assertAdminBalance(100000000)
    await assertAdminBalanceReward(0)

    // Increase allowance and lock tokens for admin
    await token.increaseAllowance(100000000, rewards.address)
    await rewards.lock(100000000, token.address)

    // Admin should now have zero balance on both tokens
    await assertAdminBalance(0)
    await assertAdminBalanceReward(0)

    // Mint reward tokens into rewards contract
    await rewardToken.mint(100000000, admin, rewards.address)

    // Total rewards supply should be now the same as minted
    const res = await rewards.getTotalRewardsSupply();
    assert.strictEqual(res.total_rewards_supply.amount, '100000000')

    // After claiming, admin should have balance in reward tokens 
    // and still have zero balance in lp token
    await rewards.claim([token.address])
    assertAdminBalance(0)
    assertAdminBalanceReward(100000000)
  })

  it('can be administrated and configured', async function () {
    this.timeout(60000)
    const { token, rewards, admin, viewkey, node, network } = context

    const { mnemonic, address } = node.genesisAccount('ALICE')
    const alice = await network.getAgent("ALICE", { mnemonic, address })

    const res = await rewards.admin
    assert.strictEqual(res.address, admin.address)
    
    await rewards.changeAdmin({ address: alice.address })

    const res1 = await rewards.admin
    assert.strictEqual(res1.address, alice.address)
  })

  it('is protected by a viewing key', async function () {
    this.timeout(60000)
    const { token, rewards, admin, viewkey } = context
    
    await token.mint(100, admin)
    assertAdminBalance(100)

    await token.increaseAllowance(100, rewards.address)

    await rewards.lock(100, token.address)
    assertAdminBalance(0)

    // Create viewkey for admin rewards
    const viewkeyNew = (await rewards.createViewingKey(admin)).key

    const timestamp = parseInt((new Date()).valueOf() / 1000);
    await rewards.simulate(admin.address, timestamp, [token.address], viewkeyNew)
    
    const acc = await rewards.getAccounts(admin.address, [token.address], viewkeyNew)
    const totalLocked = acc.accounts.map(i => parseInt(i.locked_amount)).reduce((t, i) => t + i, 0);
    assert.strictEqual(totalLocked, 100)

    try {
      // I'm using the viewkey from context here because that one should get unauthorized error
      await rewards.getAccounts(admin.address, [token.address], viewkey)

      // this is supposed to fail because we didn't get error on the call abouve
      assert.strictEqual(true, false)
    }
    catch (e) {
      assert.strictEqual(e.message, 'query contract failed: encrypted: {"unauthorized":{}} (HTTP 500)')
    }
  })

  async function setupAll () {
    this.timeout(120000)

    const {SIENNA: rewardTokenBinary, LPTOKEN: tokenBinary, REWARDS: rewardsBinary} = await ensemble.build({
      workspace: abs(),
      parallel: false
    })

    const localnet = await SecretNetwork.localnet({
      stateBase: abs('artifacts')
    });

    // run a clean localnet
    const { node, network, builder, agent } = await localnet.connect()
    await agent.nextBlock
    agent.API.fees = fees;
    Object.assign(context, { node, network, admin: agent, builder })

    // Get the genesis account for ALICE and create its agent and viewkey for token
    const { mnemonic, address } = node.genesisAccount('ALICE')
    const alice = await network.getAgent("ALICE", { mnemonic, address })
    alice.API.fees = fees;
    Object.assign(context, { alice })

    // and upload them to it
    const {codeId: tokenCodeId}   = await builder.uploadCached(tokenBinary)
    const {codeId: rewardTokenCodeId}   = await builder.uploadCached(rewardTokenBinary)
    const {codeId: rewardsCodeId} = await builder.uploadCached(rewardsBinary)
    Object.assign(context, { tokenCodeId, rewardTokenCodeId, rewardsCodeId })
  }

  async function setupEach () {
    this.timeout(120000)

    context.token = await context.admin.instantiate(new SNIP20({
      label: `token-${parseInt(Math.random() * 100000)}`,
      codeId: context.tokenCodeId,
      initMsg: ensemble.contracts.LPTOKEN.initMsg
    }))

    context.rewardToken = await context.admin.instantiate(new SNIP20({
      label: `reward-token-${parseInt(Math.random() * 100000)}`,
      codeId: context.rewardTokenCodeId,
      initMsg: ensemble.contracts.SIENNA.initMsg
    }))


    // prepare rewards manager config
    const initMsg = {
      ...ensemble.contracts.REWARDS.initMsg,
      claim_interval: 1,
      admin:     context.admin.address,
      entropy:   '',//randomBytes(36).toString('base64'),
      prng_seed: '',//randomBytes(36).toString('base64'),
      reward_token: context.rewardToken.reference,
    }
    // Override the init message to only have one token in the pool 
    // that is different then the reward_token
    initMsg.reward_pools = [
      { ...initMsg.reward_pools[0], lp_token: context.token.reference }
    ]

    // deploy rewards manager
    context.rewards = await context.admin.instantiate(new Rewards({
      label: `rewards-${parseInt(Math.random() * 100000)}`,
      codeId: context.rewardsCodeId,
      initMsg
    }))

    // create viewing key for admin balance
    context.aliceViewkey = (await context.token.createViewingKey(context.alice)).key
    context.viewkey = (await context.token.createViewingKey(context.admin)).key
    context.aliceViewkeyFoo = (await context.rewardToken.createViewingKey(context.alice)).key
    context.viewkeyFoo = (await context.rewardToken.createViewingKey(context.admin)).key
  }

  async function cleanupAll () {
    this.timeout(120000)
    await context.node.terminate()
  }

})
