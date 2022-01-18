import { ScrtContract_1_2, loadSchemas, IAgent, ContractState } from "@fadroma/scrt"
import { randomHex } from '@hackbg/tools'
import { workspace } from '@sienna/settings'
import { InitMsg } from './schema/init_msg.d'

export type AMMContractOptions = {
}

export class AMMContract extends ScrtContract_1_2 {

  crate = 'exchange'

  name  = 'SiennaAMMExchange'

  initMsg?: InitMsg

  constructor (options: ContractState & {
    admin?:    IAgent,
    prefix?:   string,
    label?:    string,
    name?:     string,
    symbol?:   string,
    decimals?: number,
  } = {}) {
    super(options)
    this.initMsg = {
      callback:          { contract: null, msg: null },
      entropy:           null,
      factory_info:      { address: null, code_hash: null },
      lp_token_contract: { id: null, code_hash: null },
      pair:              null,
      prng_seed:         randomHex(36)
    }
  }

  pairInfo = () => this.q.pairInfo()

}
