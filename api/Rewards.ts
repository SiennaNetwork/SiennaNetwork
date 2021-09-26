import { ContractAPIOptions } from '@fadroma/scrt'
import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { abs } from '../ops/index'
import { randomHex } from '@fadroma/tools'
import { SNIP20 } from './SNIP20'

const BLOCK_TIME = 6 // seconds (on average)
const threshold  = 24 * 60 * 60 / BLOCK_TIME
const cooldown   = 24 * 60 * 60 / BLOCK_TIME

export type RewardsOptions = {
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

  constructor ({ prefix, name, admin, lpToken, rewardToken }: RewardsOptions) {
    super({
      agent:  admin,
      schema: Rewards.schema,
      prefix,
      label:  `SiennaRewards_${name}_Pool`
    })
    Object.assign(this.init.msg, {
      admin:        admin.address,
      lp_token:     lpToken?.link,
      reward_token: rewardToken?.link,
      viewing_key:  ""
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
