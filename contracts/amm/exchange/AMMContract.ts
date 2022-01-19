import { IAgent, ContractState, AugmentedScrtContract_1_2, randomHex } from "@fadroma/scrt"
import { SNIP20Contract } from '@fadroma/snip20'
import { InitMsg } from './schema/init_msg.d'
import { AMMTransactions } from './AMMTransactions'
import { AMMQueries } from './AMMQueries'

export class AMMContract extends AugmentedScrtContract_1_2<AMMTransactions, AMMQueries> {

  crate = 'exchange'

  name  = 'SiennaAMMExchange'

  initMsg?: InitMsg

  Transactions = AMMTransactions
  Queries      = AMMQueries

  token0?:  SNIP20Contract
  token1?:  SNIP20Contract
  lpToken?: SNIP20Contract

  constructor (options: ContractState & {
    admin?:    IAgent,
    prefix?:   string,
    label?:    string,
    name?:     string,
    symbol?:   string,
    decimals?: number,
    lpToken?:  SNIP20Contract
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

  pairInfo = () => this.q().pair_info()

}

