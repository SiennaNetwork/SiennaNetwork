import { ScrtContract, loadSchemas, Agent, ContractAPIOptions } from "@fadroma/scrt"
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./amm/init_msg.json",
  queryMsg:    "./amm/query_msg.json",
  queryAnswer: "./amm/query_msg_response.json",
  handleMsg:   "./amm/handle_msg.json",
});

export class AMM extends ScrtContract {
  constructor (options: ContractAPIOptions = {}) { super({ ...options, schema }) }

  code = { ...this.code, workspace: abs(), crate: 'exchange' }

  init = { ...this.init, label: 'SiennaAMMExchange', msg: {} }

  pairInfo = () => this.q.pairInfo()
}
