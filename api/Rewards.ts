import { ContractAPIOptions } from '@fadroma/scrt'
import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { abs } from '../ops/index'
import { randomHex } from '@fadroma/tools'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./rewards/init.json",
  queryMsg:    "./rewards/query.json",
  queryAnswer: "./rewards/response.json",
  handleMsg:   "./rewards/handle.json",
});

const BLOCK_TIME = 6 // seconds (on average)
const threshold  = 24 * 60 * 60 / BLOCK_TIME
const cooldown   = 24 * 60 * 60 / BLOCK_TIME

export class Rewards extends ScrtContract {
  constructor (options: ContractAPIOptions = {}, name: string = '???') {
    super({ ...options, schema, label: `SiennaRewards_${name}_Pool` }) }

  code = { ...super.code, workspace: abs(), crate: 'sienna-rewards' }
  init = { ...super.init, label: 'Rewards', msg: {
    threshold,
    cooldown,
    get viewing_key () { return randomHex(36) } } }

  setProvidedToken = (address: string, code_hash: string, agent = this.instantiator) =>
    this.tx.set_provided_token({address, code_hash}, agent);

  lock = (amount: string, agent: Agent) =>
    this.tx.lock({ amount: String(amount) }, agent);

  retrieve = (amount: string, agent: Agent) =>
    this.tx.retrieve({ amount: String(amount) }, agent);

  claim = (agent: string) =>
    this.tx.claim({}, agent);
}
