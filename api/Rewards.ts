import type { Contract } from '@fadroma/scrt'
import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { abs } from '../ops/index'
import { randomHex } from '@fadroma/tools'
import { SNIP20, LPToken } from './SNIP20'

const BLOCK_TIME = 6 // seconds (on average)
const threshold  = 24 * 60 * 60 / BLOCK_TIME
const cooldown   = 24 * 60 * 60 / BLOCK_TIME

export type RewardsOptions = {
  codeId?:      number
  codeHash?:    string
  prefix?:      string
  name?:        string
  label?:       string
  admin?:       Agent
  lpToken?:     SNIP20
  rewardToken?: SNIP20
}

export class Rewards extends ScrtContract {

  static schema = loadSchemas(import.meta.url, {
    initMsg:     "./rewards/init.json",
    queryMsg:    "./rewards/query.json",
    queryAnswer: "./rewards/response.json",
    handleMsg:   "./rewards/handle.json",
  })

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new Rewards({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }

  constructor ({
    codeId,
    codeHash,
    prefix,
    label,
    name,
    admin,
    lpToken,
    rewardToken,
  }: RewardsOptions = {}) {
    super({
      agent:  admin,
      schema: Rewards.schema,
      prefix,
      label:  label || `SiennaRewards_${name}_Pool`
    })
    if (codeId)   this.blob.codeId = codeId
    if (codeHash) this.blob.codeHash = codeHash
    Object.assign(this.init.msg, {
      admin: admin?.address,
      viewing_key: randomHex(36)
    })
    Object.defineProperties(this.init.msg, {
      lp_token:     { enumerable: true, get () { return lpToken?.link } },
      reward_token: { enumerable: true, get () { return rewardToken?.link } }
    })
  }

  code = {
    ...this.code,
    workspace: abs(),
    crate: 'sienna-rewards'
  }

  init = {
    ...this.init,
    label: this.init.label||'SiennaRewards',
    msg: {
      threshold,
      cooldown,
      viewing_key: randomHex(36)
    }
  }

  poolInfo = async (
    agent: Agent = this.instantiator
  ) => {
    const { header: { height: at } } = await agent.block
    const { pool_info } = await this.q.pool_info({ at })
    return pool_info
  }

  getRewardToken = async (
    agent: Agent = this.instantiator,
    Class: new()=>Contract = SNIP20
  ) => {
    const { address, code_hash } = (await this.poolInfo(agent)).reward_token
    return LPToken.attach(address, code_hash, agent)
  }

  getLPToken = async (
    agent: Agent = this.instantiator
  ) => {
    const { address, code_hash } = (await this.poolInfo(agent)).lp_token
    return LPToken.attach(address, code_hash, agent)
  }

  setLPToken = (
    address:   string,
    code_hash: string,
    agent: Agent = this.instantiator
  ) =>
    this.tx.set_provided_token({address, code_hash}, agent);

  lock = (
    amount: string,
    agent: Agent = this.instantiator
  ) =>
    this.tx.lock({ amount: String(amount) }, agent);

  retrieve = (
    amount: string,
    agent: Agent = this.instantiator
  ) =>
    this.tx.retrieve({ amount: String(amount) }, agent);

  claim = (
    agent: Agent = this.instantiator
  ) =>
    this.tx.claim({}, agent);

  close = (
    message: string,
    agent: Agent = this.instantiator
  ) =>
    this.tx.closePool({ message }, agent);

}

export class RewardsEmergencyProxy extends ScrtContract {

  static schema = loadSchemas(import.meta.url, {
    initMsg:     "./rewards_emergency_proxy/init.json",
    queryMsg:    "./rewards_emergency_proxy/query.json",
    queryAnswer: "./rewards_emergency_proxy/response.json",
    handleMsg:   "./rewards_emergency_proxy/handle.json",
  })

  code = {
    ...this.code,
    workspace: abs(),
    crate: 'sienna-rewards-emergency-proxy'
  }
}
