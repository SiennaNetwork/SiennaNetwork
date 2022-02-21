import { Agent, Client, Snip20Client } from '@hackbg/fadroma'
import { LPTokenClient } from '@sienna/api'

import { TokenPair, Uint128 } from './schema/handle_msg.d'

export type AMMVersion = 'v1'|'v2'

export class AMMExchangeClient extends Client {

  static get = getExchange

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

/** An exchange is an interaction between 4 contracts. */
export type ExchangeInfo = {
  /** Shorthand to refer to the whole group. */
  name?: string
  /** One token. */
  TOKEN_0:  Snip20Client|string,
  /** Another token. */
  TOKEN_1:  Snip20Client|string,
  /** The automated market maker/liquidity pool for the token pair. */
  EXCHANGE: AMMExchangeClient,
  /** The liquidity provision token, which is minted to stakers of the 2 tokens. */
  LP_TOKEN: LPTokenClient,
  /** The bare-bones data needed to retrieve the above. */
  raw:      any
}

async function getExchange (
  agent:   Agent,
  address: string,
  token_0: Snip20Contract|TokenType,
  token_1: Snip20Contract|TokenType,
  version = 'v2'
): Promise<ExchangeInfo> {

  const EXCHANGE = new AMMExchangeClient({
    chain:    agent.chain,
    codeId:   await agent.getCodeId(address),
    codeHash: await agent.getCodeHash(address),
    address,
    agent,
  })

  const { TOKEN: TOKEN_0, NAME: TOKEN_0_NAME } = await Snip20Client.fromTokenSpec(agent, token_0)
  const { TOKEN: TOKEN_1, NAME: TOKEN_1_NAME } = await Snip20Client.fromTokenSpec(agent, token_1)
  const name = `${TOKEN_0_NAME}-${TOKEN_1_NAME}`

  const { liquidity_token } = await EXCHANGE.getPairInfo()

  const LP_TOKEN = new LPTokenClient({
    chain:    agent.chain,
    codeId:   await agent.getCodeId(liquidity_token.address),
    codeHash: liquidity_token.code_hash,
    address:  liquidity_token.address,
    agent,
  })

  return {
    raw: { // no methods, just data
      exchange: { address: EXCHANGE.address },
      lp_token: { address: LP_TOKEN.address, code_hash: LP_TOKEN.codeHash },
      token_0,
      token_1,
    },
    name,     // The human-friendly name of the exchange
    EXCHANGE, // The exchange contract
    LP_TOKEN, // The LP token contract
    TOKEN_0,  // One token of the pair
    TOKEN_1,  // The other token of the pair
  }

}
