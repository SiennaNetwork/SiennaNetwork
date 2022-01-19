import {
  IAgent, ContractState,
  AugmentedScrtContract_1_2, TransactionExecutor, QueryExecutor,
  randomHex
} from "@fadroma/scrt"

import { workspace } from '@sienna/settings'

import { InitMsg } from './schema/init_msg.d'

export class AMMContract extends AugmentedScrtContract_1_2<AMMExecutor, AMMQuerier> {

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

  Transactions = AMMExecutor

  Queries  = AMMQuerier

  pairInfo = () => this.q().pair_info()

}

export class AMMExecutor extends TransactionExecutor {}

export class AMMQuerier extends QueryExecutor {
  pair_info () {
    return this.query({ pair_info: {} })
  }
}
