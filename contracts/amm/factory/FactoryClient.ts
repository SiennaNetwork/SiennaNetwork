import { Agent, Contract, Client } from '@hackbg/fadroma'
import { b64encode } from "@waiting/base64"
import { EnigmaUtils } from "secretjs"
import { AMMExchangeClient, ExchangeInfo } from '@sienna/exchange'

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

export class FactoryClient extends Client {

  /** Return the collection of contract templates
    * (`{ id, code_hash }` structs) that the factory
    * uses to instantiate contracts. */
  async getContracts (): Promise<FactoryInventory> {
    const { config } = await this.query({ get_config: {} })
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
    token_0: TokenType,
    token_1: TokenType
  ) {
    await this.execute({
      create_exchange: {
        pair:    { token_0, token_1 },
        entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString())
      }
    })
  }

  /** Get info about an exchange. */
  async getExchange (
    token_0: TokenType,
    token_1: TokenType
  ): Promise<ExchangeInfo> {
    const {get_exchange_address:{address}} =
      await this.query({
        get_exchange_address: {
          pair: {
            token_0,
            token_1
          }
        }
      })
    return await AMMExchangeClient.get(
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
      const {list_exchanges: {exchanges: list}} = await this.query({
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
