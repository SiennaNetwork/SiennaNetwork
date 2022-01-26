import { Agent, ContractState, randomHex, Scrt_1_2 } from "@hackbg/fadroma"
import { SNIP20Contract } from '@fadroma/snip20'

import { AMMTransactions } from './AMMTransactions'
import { AMMQueries } from './AMMQueries'

import { InitMsg } from './schema/init_msg.d'
import { TokenType, TokenPair, ContractLink } from './schema/query_msg_response.d'

export class AMMContract extends Scrt_1_2.Contract<AMMTransactions, AMMQueries> {

  crate = 'exchange'
  name  = 'SiennaAMMExchange'

  Transactions = AMMTransactions
  Queries      = AMMQueries

  initMsg?: InitMsg
  token_0?: TokenType
  token_1?: TokenType
  lpToken?: SNIP20Contract

  constructor (options: ContractState & {
    admin?:    Agent,
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

  async populate () {
    const pairInfo = await this.pairInfo()
    const { pair: { token_0, token_1 }, liquidity_token } = pairInfo
    this.token_0  = token_0
    this.token_1  = token_1
    this.lpToken = new SNIP20Contract(liquidity_token)
    return this
  }

  pairInfo = (): Promise<{ pair: TokenPair, liquidity_token: ContractLink }> => {
    return this.q().pair_info()
  }

}
