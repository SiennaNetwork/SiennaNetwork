import { ScrtContract, loadSchemas, Agent, ContractAPIOptions } from "@fadroma/scrt"
import { randomHex } from '@fadroma/tools'
import { abs } from '../ops/index'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./amm/init_msg.json",
  queryMsg:    "./amm/query_msg.json",
  queryAnswer: "./amm/query_msg_response.json",
  handleMsg:   "./amm/handle_msg.json",
});

export class AMM extends ScrtContract {
  constructor (options: {
    admin?:    Agent,
    prefix?:   string,
    label?:    string,
    name?:     string,
    symbol?:   string,
    decimals?: number,
  } = {}) {
    super({ agent: options?.admin, schema })
    if (options.prefix) this.init.prefix = options.prefix
    this.init.label = options?.label
    Object.assign(this.init.msg, {
      name:      options?.name,
      symbol:    options?.symbol,
      decimals:  options?.decimals,
      prng_seed: randomHex(36)
    })
  }

  code = { ...this.code, workspace: abs(), crate: 'exchange' }

  init = { ...this.init, label: 'SiennaAMMExchange', msg: {} }

  pairInfo = () => this.q.pairInfo()

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new AMM({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }
}
