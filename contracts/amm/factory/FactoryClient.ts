import { Agent, Contract, Client, Snip20Client } from '@hackbg/fadroma'
import { b64encode } from "@waiting/base64"
import { EnigmaUtils } from "secretjs"
import { AMMExchangeClient, ExchangeInfo } from '@sienna/exchange'

import { ContractInstantiationInfo } from './schema/init_msg.d'
import { TokenType, TokenPair, ContractStatusLevel, HumanAddr } from './schema/handle_msg.d'
import { Exchange } from './schema/query_response.d'

export type AMMFactoryTemplates = {
  pair_contract:       ContractInstantiationInfo
  lp_token_contract:   ContractInstantiationInfo
  snip20_contract?:    ContractInstantiationInfo
  ido_contract?:       ContractInstantiationInfo
  launchpad_contract?: ContractInstantiationInfo
  router_contract?:    ContractInstantiationInfo
}

export type AMMVersion = "v1"|"v2"

export abstract class AMMFactoryClient extends Client {

  abstract readonly version: AMMVersion
  static "v1" = class AMMFactoryClient_v1 extends AMMFactoryClient {
    readonly version = "v1" as AMMVersion
  }
  static "v2" = class AMMFactoryClient_v2 extends AMMFactoryClient {
    readonly version = "v2" as AMMVersion
  }

  setStatus (
    level:        ContractStatusLevel,
    new_address?: HumanAddr,
    reason:       string = ""
  ) {
    return this.execute({
      set_status: { level, new_address, reason }
    })
  }

  /** Return the collection of contract templates
    * (`{ id, code_hash }` structs) that the factory
    * uses to instantiate contracts. */
  async getContracts (): Promise<AMMFactoryTemplates> {
    const { config } = await this.query({ get_config: {} })
    return {
      snip20_contract:    config.snip20_contract,
      pair_contract:      config.pair_contract,
      lp_token_contract:  config.lp_token_contract,
      ido_contract:       config.ido_contract,
      launchpad_contract: config.launchpad_contract,
    }
  }

  /** Create a liquidity pool, i.e. an instance of the AMMExchange contract */
  createExchange (
    token_0: TokenType,
    token_1: TokenType
  ) {
    const msg = {
      create_exchange: {
        pair:    { token_0, token_1 },
        entropy: b64encode(EnigmaUtils.GenerateNewSeed().toString())
      }
    }
    return this.execute(msg)
  }

  /** Creates multiple exchanges in the same transaction. */
  async createExchanges (input: {

    templates: AMMFactoryTemplates

    pairs: Array<{
      name?: string,
      pair: {
        token_0: Snip20Client|TokenType,
        token_1: Snip20Client|TokenType
      }
    }>

  }): Promise<Array<{

    name?: string,

    token_0: Snip20Client|TokenType,
    token_1: Snip20Client|TokenType

  }>> {

    const {
      templates = await this.getContracts(),
      pairs,
    } = input

    if (pairs.length === 0) {
      console.warn('Creating 0 exchanges.')
      return []
    }

    const newPairs = []

    await this.agent.bundle().wrap(async bundle=>{
      const bundledThis = this.switchAgent(bundle)
      for (const pair of pairs) {
        let token_0 = pair.pair?.token_0 || pair.raw?.token_0
        let token_1 = pair.pair?.token_1 || pair.raw?.token_1
        if (token_0 instanceof Snip20Client) token_0 = token_0.asCustomToken
        if (token_1 instanceof Snip20Client) token_1 = token_1.asCustomToken
        const exchange = await bundledThis.createExchange(token_0, token_1)
        newPairs.push(pair)
      }
    })

    return newPairs
  }

  /** Creates multiple exchanges in the same transaction. */
  async createExchanges (input: {
    templates: AMMFactoryTemplates
    pairs: { name?: string, TOKEN_0: Snip20Client, TOKEN_1: Snip20Client }[]
  }): Promise<{ name?: string, TOKEN_0: Snip20Client, TOKEN_1: Snip20Client }[]> {

    const {
      templates = await this.getContracts(),
      pairs,
    } = input

    if (pairs.length === 0) {
      console.warn('Creating 0 exchanges.')
      return []
    }

    const newPairs = []

    await this.agent.bundle().wrap(async bundle=>{
      const bundledThis = this.client(bundle)
      console.log(this, bundledThis)
      for (const { name, TOKEN_0, TOKEN_1 } of pairs) {
        const exchange = await bundledThis.createExchange(
          TOKEN_0.asCustomToken,
          TOKEN_1.asCustomToken
        )
        newPairs.push({name, TOKEN_0, TOKEN_1})
      }
    })

    return newPairs
  }

  /** Get info about an exchange. */
  async getExchange (
    token_0: TokenType,
    token_1: TokenType
  ): Promise<ExchangeInfo> {
    const {get_exchange_address:{address}} = await this.query({
      get_exchange_address: { pair: { token_0, token_1 } }
    })
    return await AMMExchangeClient.get(this.agent, address, token_0, token_1)
  }

  /** Get the full list of raw exchange info from the factory. */
  async listExchanges (): Promise<{
    address: string,
    pair: {
      token_0: TokenType,
      token_1: TokenType
    }
  }[]> {
    const result = []
    const limit = 30
    let start = 0
    while (true) {
      const msg = { list_exchanges: { pagination: { start, limit } } }
      const {list_exchanges: {exchanges: list}} = await this.query(msg)
      if (list.length > 0) {
        result.push(...list)
        start += limit
      } else {
        break
      }
    }
    return result
  }

  async listExchangesFull (): Promise<ExchangeInfo[]> {
    const exchanges = await this.listExchanges()
    return Promise.all(
      exchanges.map((info) => {
        const { pair: { token_0, token_1 } } = info
        const address = info.address || info.contract.address
        return AMMExchangeClient.get(this.agent, address, token_0, token_1)
      })
    )
  }

}
