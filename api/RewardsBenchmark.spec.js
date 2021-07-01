import { randomBytes } from 'crypto'
import { SecretNetwork } from "@fadroma/scrt-agent"
import { gas } from "@fadroma/scrt-agent/gas.js"
import { abs } from "../ops/lib/index.js"
import SNIP20 from "./SNIP20.js"
import RewardsBenchmark from "./RewardsBenchmark.js"

describe("RewardsBenchmark", () => {

  const fees = { upload: gas(20000000)
               , init:   gas(1000000)
               , exec:   gas(1000000)
               , send:   gas(500000) }

  const context = {}

  before(setupAll)
  beforeEach(setupEach)
  after(cleanupAll)

  it("uses a reasonable amount of gas when processing claims", async function () {
    this.timeout(30000)

    console.debug('init asset token:')
    const asset = await context.agent.instantiate(new SNIP20({
      codeId: context.token.id,
      label:  'asset',
      initMsg: { prng_seed: randomBytes(36).toString('hex')
               , name:     "Asset"
               , symbol:   "ASSET"
               , decimals: 18
               , config:
                 { public_total_supply: true
                 , enable_deposit: true
                 , enable_redeem: true
                 , enable_mint: true
                 , enable_burn: true } } }))

    console.debug('init reward token:')
    const reward = await context.agent.instantiate(new SNIP20({
      codeId: context.token.id,
      label:  'reward',
      initMsg: { prng_seed: randomBytes(36).toString('hex')
               , name:     "Reward"
               , symbol:   "REWARD"
               , decimals: 18
               , config:
                 { public_total_supply: true
                 , enable_deposit: true
                 , enable_redeem: true
                 , enable_mint: true
                 , enable_burn: true } } }))

    console.debug('init reward pool:')
    const lending = await context.agent.instantiate(new RewardsBenchmark({
      codeId: context.pool.id,
      label: 'lending',
      initMsg: { provided_token: asset.reference
               , rewarded_token: reward.reference } }))

    
  })

  /// harness

  async function setupAll () {
    this.timeout(60000)
    const localnet = await SecretNetwork.localnet({ stateBase: abs("artifacts") })
    const {node, network, builder, agent} = await localnet.connect()
    agent.API.fees = fees

    const workspace = abs()
    const [ tokenBinary, poolBinary ] = await Promise.all([
      builder.build({workspace, crate: 'snip20-sienna'           }),
      builder.build({workspace, crate: 'sienna-rewards-benchmark'}), ])

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

    Object.assign(context, {
      node, network, builder, agent,
      token: { id: tokenCodeId, code_hash: tokenCodeHash },
      pool:  { id: poolCodeId,  code_hash: poolCodeHash  },
    })
  }

  async function setupEach () {
  }

  async function cleanupAll () {}

})
