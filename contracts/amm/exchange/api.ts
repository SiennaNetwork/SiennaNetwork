import { ScrtContract_1_2, loadSchemas, Agent, ContractAPIOptions } from "@fadroma/scrt"
import { randomHex } from '@hackbg/tools'
import { workspace } from '@sienna/settings'

export const schema = loadSchemas(import.meta.url, {
  initMsg:     "./schema/init_msg.json",
  queryMsg:    "./schema/query_msg.json",
  queryAnswer: "./schema/query_msg_response.json",
  handleMsg:   "./schema/handle_msg.json",
});

export type AMMContractOptions = {
  admin?:    Agent,
  prefix?:   string,
  label?:    string,
  name?:     string,
  symbol?:   string,
  decimals?: number,
}

export class AMMContract extends ScrtContract_1_2 {

  constructor (options: AMMContractOptions = {}) {
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

  code = { ...this.code, workspace, crate: 'exchange' }

  init = { ...this.init, label: 'SiennaAMMExchange', msg: {} }

  pairInfo = () => this.q.pairInfo()

  static attach = (
    address:  string,
    codeHash: string,
    agent:    Agent
  ) => {
    const instance = new AMMContract({ admin: agent })
    instance.init.agent = agent
    instance.init.address = address
    instance.blob.codeHash = codeHash
    return instance
  }
}
