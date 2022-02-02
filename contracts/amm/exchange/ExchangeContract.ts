import { timestamp, randomHex, Scrt_1_2, SNIP20Contract } from "@hackbg/fadroma"
import { InitMsg } from './schema/init_msg.d'
import { AMMTransactions, AMMQueries } from './ExchangeClient'
import { TokenType, TokenPair, ContractLink } from './schema/query_msg_response.d'
import { LPTokenContract } from '@sienna/lp-token'
import { workspace } from '@sienna/settings'

/** An exchange is an interaction between 4 contracts. */
export type ExchangeInfo = {
  /** Shorthand to refer to the whole group. */
  name?: string
  /** One token. */
  TOKEN_0:  SNIP20Contract|string,
  /** Another token. */
  TOKEN_1:  SNIP20Contract|string,
  /** The automated market maker/liquidity pool for the token pair. */
  EXCHANGE: AMMExchangeContract,
  /** The liquidity provision token, which is minted to stakers of the 2 tokens. */
  LP_TOKEN: LPTokenContract,
  /** The bare-bones data needed to retrieve the above. */
  raw:      any
}

export class AMMExchangeContract extends Scrt_1_2.Contract<AMMTransactions, AMMQueries> {
  workspace = workspace
  crate     = 'exchange'
  name      = 'AMMExchange'
  initMsg?: InitMsg = {
    callback:          { contract: null, msg: null },
    entropy:           null,
    factory_info:      { address: null, code_hash: null },
    lp_token_contract: { id: null, code_hash: null },
    pair:              null,
    prng_seed:         randomHex(36),
  }
  Transactions = AMMTransactions
  Queries      = AMMQueries

  token_0?: TokenType
  token_1?: TokenType
  lpToken?: SNIP20Contract

  constructor (options) {
    super(options)
    const { version } = options||{}
    if (version === 'v1') {
      this.ref    = 'a99d8273b4'
      this.suffix = `@v1+${timestamp()}`
    } else if (version === 'v2') {
      this.suffix = `@v2+${timestamp()}`
    } else {
      /* nop */
    }
  }

  get info (): Promise<any/*ExchangeInfo*/> {
    throw new Error('todo')
  }

  //async populate () {
    //const pairInfo = await this.pairInfo()
    //const { pair: { token_0, token_1 }, liquidity_token } = pairInfo
    //this.token_0  = token_0
    //this.token_1  = token_1
    //this.lpToken = new SNIP20Contract(liquidity_token)
    //return this
  //}

  pairInfo = (): Promise<{ pair: TokenPair, liquidity_token: ContractLink }> => {
    return this.q().pair_info()
  }

}
