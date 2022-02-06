import { Client } from '@hackbg/fadroma'

import { TokenPair, Uint128 } from './schema/handle_msg.d'

export class AMMExchangeClient extends Client {

  async addLiquidity (
    pair:     TokenPair,
    amount_0: Uint128,
    amount_1: Uint128
  ) {
    const result = await this.execute({
      add_liquidity: { deposit:{ pair, amount_0, amount_1 } }
    })
    return result
  }

  async getPairInfo () {
    const { pair_info } = await this.query("pair_info")
    return pair_info
  }

}
