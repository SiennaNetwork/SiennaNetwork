import { ContractAPIOptions } from '@fadroma/scrt'
import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { abs } from '../ops/index'
import { randomHex } from '@fadroma/tools'
import { SNIP20 } from './SNIP20'

const BLOCK_TIME = 6 // seconds (on average)
const threshold  = 24 * 60 * 60 / BLOCK_TIME
const cooldown   = 24 * 60 * 60 / BLOCK_TIME

export type RewardsOptions = {
  codeId?:      number
  codeHash?:    string
  prefix?:      string
  name?:        string
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

  constructor ({
    codeId,
    codeHash,
    prefix,
    name,
    admin,
    lpToken,
    rewardToken,
  }: RewardsOptions) {
    super({
      agent:  admin,
      schema: Rewards.schema,
      prefix,
      label:  `SiennaRewards_${name}_Pool`
    })
    if (codeId)   this.blob.codeId = codeId
    if (codeHash) this.blob.codeHash = codeHash
    Object.assign(this.init.msg, {
      admin: admin.address,
      viewing_key:  ""
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

  setProvidedToken = (address: string, code_hash: string, agent = this.instantiator) =>
    this.tx.set_provided_token({address, code_hash}, agent);

  lock = (amount: string, agent: Agent) =>
    this.tx.lock({ amount: String(amount) }, agent);

  retrieve = (amount: string, agent: Agent) =>
    this.tx.retrieve({ amount: String(amount) }, agent);

  claim = (agent: string) =>
    this.tx.claim({}, agent);
}
