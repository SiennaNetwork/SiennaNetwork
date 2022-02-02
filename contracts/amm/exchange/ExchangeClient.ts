import { TransactionExecutor } from '@hackbg/fadroma'
import { TokenPair, Uint128 } from './schema/handle_msg.d'

export class AMMTransactions extends TransactionExecutor {
  async add_liquidity (
    pair:     TokenPair,
    amount_0: Uint128,
    amount_1: Uint128
  ) {
    const deposit = { pair, amount_0, amount_1 }
    const result = await this.execute({add_liquidity:{deposit}})
    return result
  }
}

import { QueryExecutor } from '@hackbg/fadroma'

export class AMMQueries extends QueryExecutor {
  async pair_info () {
    const { pair_info } = await this.query("pair_info")
    return pair_info
  }
}
