import assert from 'assert'
import { randomBytes } from 'crypto'

import {SecretNetwork} from '@fadroma/scrt-agent'
import ensureWallets from '@fadroma/scrt-agent/fund.js'

import SNIP20 from './SNIP20.js'
import Rewards from './Rewards.js'

import {abs} from '../ops/lib/index.js'
import RewardsContracts from '../ops/RewardsContracts.js'
import debug from 'debug'
const log = debug('out')

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
    address = context.admin.address,
    key = context.viewkey,
  ) => context.token.balance(address, key)

  const assertBalance = async (amount = 0, address, key) => {
    assert.strictEqual(await balance(address, key), String(amount))
  }

  before(setupAll)

  after(cleanupAll)

  beforeEach(setupEach)

  it('can lock and return tokens', async function () {
    this.timeout(60000)
    const {admin, token, rewards}=context

    await token.mint(100)
    assertBalance(100)

    await token.increaseAllowance(100, rewards.address)
    await rewards.lock(50, token.address)
    assertBalance(50)
    
    await token.decreaseAllowance(100, rewards.address)

    await rewards.retrieve(50, token.address)
    assertBalance(100)
  })

  it('can process claims', async function () {
    this.timeout(120000)
    const { node, network, token, rewards, admin, viewkey } = context

    // Get the genesis account for ALICE and create its agent and viewkey for token
    const { mnemonic, address } = node.genesisAccount('ALICE')
    const alice = await network.getAgent("ALICE", { mnemonic, address })
    const aliceViewkey = (await token.createViewingKey(alice)).key

    // Mint 100 tokens for admin and alice
    await token.mint(100, admin)
    await token.addMinters([alice.address])
    await token.mint(100, alice)
    await assertBalance(100)
    await assertBalance(100, alice.address, aliceViewkey)

    // Mint another 100 tokens for admin
    await token.mint(100, admin)
    await assertBalance(200)

    // Increase allowance for admin and alice to allow them to send tokens to rewards
    await token.increaseAllowance(200, rewards.address)
    await token.increaseAllowance(100, rewards.address, alice)

    // Lock 100 of admin tokens
    await rewards.lock(100, token.address)
    await assertBalance(100)

    // Lock 100 alices tokens
    await rewards.lock(100, token.address, alice)
    await assertBalance(0, alice.address, aliceViewkey)

    // Get total rewards supply that should be 200 now
    const res = await rewards.getTotalRewardsSupply();
    assert.strictEqual(res.total_rewards_supply.amount, '200')

    // Make a claim from admin and expect admin to have 200 tokens
    const res2 = await rewards.claim([token.address])
    log(JSON.stringify(res2, null, 2))
    // fails here, admin now has 300 tokens, which shouldn't happen, 100 is from alice
    await assertBalance(200)
  })

  // it('can be configured', () => {})

  it('can be administrated', async function () {
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
    assertBalance(100)

    await token.increaseAllowance(100, rewards.address)

    await rewards.lock(100, token.address)
    assertBalance(0)

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
      label: `token-${parseInt(Math.random() * 100000)}`,
      codeId: context.tokenCodeId,
      initMsg: ensemble.contracts.TOKEN.initMsg
    }))
    const reward_token = context.token.reference


    // prepare rewards manager config
    const initMsg = {
      ...{...ensemble.contracts.REWARDS.initMsg, claim_interval: 300},
      admin:     context.admin.address,
      entropy:   '',//randomBytes(36).toString('base64'),
      prng_seed: '',//randomBytes(36).toString('base64'),
      reward_token,
    }
    initMsg.reward_pools[0].lp_token = reward_token

    // deploy rewards manager
    context.rewards = await context.admin.instantiate(new Rewards({
      label: `rewards-${parseInt(Math.random() * 100000)}`,
      codeId: context.rewardsCodeId,
      initMsg
    }))

    // create viewing key for admin balance
    context.viewkey = (await context.token.createViewingKey(context.admin)).key
  }

  async function cleanupAll () {
    this.timeout(60000)
    await context.node.terminate()
  }

})
