import { ScrtContract, loadSchemas, Agent } from "@fadroma/scrt"
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./amm/init_msg.json",
  queryMsg:    "./amm/query_msg.json",
  queryAnswer: "./amm/query_msg_response.json",
  handleMsg:   "./amm/handle_msg.json",
});

export class AMM extends ScrtContract {
  code = { ...super.code, workspace: abs(), crate: 'exchange' }
  init = { ...super.init, label: 'SiennaAMMExchange', msg: {} }
  constructor (agent: Agent) {
    super(schema, agent) }
  pairInfo = () =>
    this.q.pairInfo()
}
