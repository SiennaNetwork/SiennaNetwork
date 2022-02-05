import { Agent, Contract, SNIP20Contract, QueryExecutor, TransactionExecutor } from '@hackbg/fadroma'
import { b64encode }   from "@waiting/base64"
import { EnigmaUtils } from "secretjs"
import { AMMExchangeContract, ExchangeInfo } from '@sienna/exchange'

import { ContractInstantiationInfo } from './schema/init_msg.d'
import { TokenType } from './schema/handle_msg.d'
import { Exchange } from './schema/query_response.d'

export type FactoryInventory = {
  snip20_contract?:    ContractInstantiationInfo
  pair_contract?:      ContractInstantiationInfo
  lp_token_contract?:  ContractInstantiationInfo
  ido_contract?:       ContractInstantiationInfo
  launchpad_contract?: ContractInstantiationInfo
  router_contract?:    ContractInstantiationInfo
}

export class FactoryClient {

  constructor (
    readonly agent:    Agent,
    readonly address:  string,
    readonly codeHash: string
  ) {
    if (!agent) {
      throw new Error('@sienna/amm/FactoryClient: no agent')
    }
  }

  // shims for dual executors:

  /** Return the collection of contract templates
    * (`{ id, code_hash }` structs) that the factory
    * uses to instantiate contracts. */
  async getContracts (): Promise<FactoryInventory> {
    const { config } = await this.agent.query(this, { get_config: {} })
    return {
      snip20_contract:    config.snip20_contract,
      pair_contract:      config.pair_contract,
      lp_token_contract:  config.lp_token_contract,
      ido_contract:       config.ido_contract,
      launchpad_contract: config.launchpad_contract,
    }
  }

  /** Create a liquidity pool, i.e. an instance of the exchange contract,
    * and return info about it from getExchange. */
  async createExchange (
    token_0: SNIP20Contract|TokenType,
    token_1: SNIP20Contract|TokenType
  ): Promise<ExchangeInfo> {
    if (token_0 instanceof SNIP20Contract) token_0 = token_0.asCustomToken
    if (token_1 instanceof SNIP20Contract) token_1 = token_1.asCustomToken
    await this.agent.execute(this, {
      create_exchange: {
        pair:    { token_0, token_1 },
        entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString())
      }
    })
    return await this.getExchange(token_0, token_1)
  }

  /** Get info about an exchange. */
  async getExchange (
    token_0: SNIP20Contract|TokenType,
    token_1: SNIP20Contract|TokenType
  ): Promise<ExchangeInfo> {
    if (token_0 instanceof SNIP20Contract) token_0 = token_0.asCustomToken
    if (token_1 instanceof SNIP20Contract) token_1 = token_1.asCustomToken
    const {get_exchange_address:{address}} =
      await this.agent.query(this, {
        get_exchange_address: {
          pair: {
            token_0,
            token_1
          }
        }
      })
    return await AMMExchangeContract.getExchange(
      this.agent,
      address,
      token_0,
      token_1
    )
  }

  /** Get the full list of raw exchange info from the factory. */
  async listExchanges (): Promise<Exchange[]> {
    const result: Exchange[] = []
    const limit = 30
    let start = 0
    while (true) {
      const {list_exchanges: {exchanges: list}} = await this.agent.query(this, {
        list_exchanges: {
          pagination: { start, limit }
        }
      })
      if (list.length > 0) {
        result.push(...list)
        start += limit
      } else {
        break
      }
    }
    return result
  }

  get exchanges (): Promise<ExchangeInfo[]> {
    return this.listExchanges().then(exchanges=>{
      return Promise.all(
        exchanges.map(({ pair: { token_0, token_1 } }) => {
          return this.getExchange(token_0, token_1)
        })
      )
    })
  }

}

export class FactoryTransactions extends TransactionExecutor {
  create_exchange (token_0: TokenType, token_1: TokenType) {
    return this.execute({ create_exchange: { pair: { token_0, token_1 }, entropy } })
  }
  create_launchpad (tokens: object[]) {
    return this.execute({ create_launchpad: { tokens } })
  }
}
