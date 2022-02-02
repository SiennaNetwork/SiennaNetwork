import {
  Scrt_1_2, SNIP20Contract, ContractInfo, Agent, MigrationContext,
  randomHex, colors, bold, Console, timestamp,
  printContract, printToken, printContracts
} from '@hackbg/fadroma'

const console = Console('@sienna/factory/Contract')

import getSettings, { workspace } from '@sienna/settings'

import { AMMExchangeContract, ExchangeInfo } from '@sienna/exchange'
import { AMMSNIP20Contract, deployPlaceholders } from '@sienna/amm-snip20'
import { LPTokenContract } from '@sienna/lp-token'
import { IDOContract } from '@sienna/ido'
import { LaunchpadContract } from '@sienna/launchpad'
import { SiennaSNIP20Contract } from '@sienna/snip20-sienna'

import { InitMsg, ExchangeSettings, ContractInstantiationInfo } from './schema/init_msg.d'
import { TokenType } from './schema/handle_msg.d'
import { QueryResponse, Exchange } from './schema/query_response.d'

export type FactoryInventory = {
  snip20_contract?:    ContractInstantiationInfo
  pair_contract?:      ContractInstantiationInfo
  lp_token_contract?:  ContractInstantiationInfo
  ido_contract?:       ContractInstantiationInfo
  launchpad_contract?: ContractInstantiationInfo
  router_contract?:    ContractInstantiationInfo
}

import { FactoryTransactions, FactoryQueries } from './FactoryApi'
export class AMMFactoryContract extends Scrt_1_2.Contract<FactoryTransactions, FactoryQueries> {
  //workspace    = 'workspace'
  crate        = 'factory'
  version      = 'v2'
  name         = `AMM[${this.version}].Factory`
  Transactions = FactoryTransactions
  Queries      = FactoryQueries

  constructor (options) {
    super(options)
    const { version } = options||{}
    this.version = version
    this.name = `AMM[${this.version}].Factory`
    if (version === 'v1') {
      this.ref = 'a99d8273b4'
    } else if (version === 'v2') {
    } else {
      /* nop */
    }
  }

  static v1 = class AMMFactoryContract_v1 extends AMMFactoryContract {
    version = 'v1'
    name    = `AMM[${this.version}].Factory`
    ref     = 'a99d8273b4'
  }

  static v2 = class AMMFactoryContract_v2 extends AMMFactoryContract {
    version = 'v2'
    name    = `AMM[${this.version}].Factory`
  }

  /** Return the collection of contract templates
    * (`{ id, code_hash }` structs) that the factory
    * uses to instantiate contracts. */
  getContracts (): Promise<FactoryInventory> {
    // type kludge!
    if (this.address) {
      // If this contract has an address query this from the contract state
      return (this.q().get_config()).then((config: FactoryInventory)=>{
        return {
          snip20_contract:    config.snip20_contract,
          pair_contract:      config.pair_contract,
          lp_token_contract:  config.lp_token_contract,
          ido_contract:       config.ido_contract,
          launchpad_contract: config.launchpad_contract,
        }
      })
    } else {
      throw new Error('not deployed yet')
    }
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
  /** Get the full list of raw exchange info from the factory. */
  async listExchanges (): Promise<Exchange[]> {
    const result: Exchange[] = []
    const limit = 30
    let start = 0
    while (true) {
      const list = await this.q().list_exchanges(start, limit)
      if (list.length > 0) {
        result.push(...list)
        start += limit
      } else {
        break
      }
    }
    return result
  }
  /** Get info about an exchange. */
  async getExchange (
    token_0: TokenType,
    token_1: TokenType,
  ): Promise<ExchangeInfo> {
    //console.info(bold('Looking for exchange'))
    //console.info(bold('  between'), JSON.stringify(token_0))
    //console.info(bold('      and'), JSON.stringify(token_1))
    const { agent, prefix, chain } = this
    const { address } = (await this.q(agent).get_exchange_address(token_0, token_1))
    const EXCHANGE = new AMMExchangeContract({
      chain,
      address,
      codeHash: await agent.getCodeHash(address),
      codeId:   await agent.getCodeId(address),
      prefix,
      agent,
    })
    const getTokenName = async TOKEN => {
      let TOKEN_NAME: string
      if (TOKEN instanceof SNIP20Contract) {
        const TOKEN_INFO = await TOKEN.q(agent).tokenInfo()
        return TOKEN_INFO.symbol
      } else {
        return 'SCRT'
      }
    }
    const TOKEN_0      = SNIP20Contract.fromTokenSpec(agent, token_0)
    const TOKEN_0_NAME = await getTokenName(TOKEN_0)
    const TOKEN_1      = SNIP20Contract.fromTokenSpec(agent, token_1)
    const TOKEN_1_NAME = await getTokenName(TOKEN_1)
    const name = `${TOKEN_0_NAME}-${TOKEN_1_NAME}`
    const { liquidity_token } = await EXCHANGE.pairInfo()
    const LP_TOKEN = new LPTokenContract({
      agent, prefix, chain,
      address:  liquidity_token.address,
      codeHash: liquidity_token.code_hash,
      codeId:   await agent.getCodeId(liquidity_token.address),
      agent
    })
    return {
      name,
      EXCHANGE, TOKEN_0, TOKEN_1, LP_TOKEN,
      raw: {
        exchange: { address: EXCHANGE.address },
        token_0,
        token_1,
        lp_token: { address: LP_TOKEN.address, code_hash: LP_TOKEN.codeHash },
      }
    }
  }

  /** Create a liquidity pool, i.e. an instance of the exchange contract. */
  async createExchange (
    token_0: SNIP20Contract|TokenType,
    token_1: SNIP20Contract|TokenType,
    agent = this.agent
  ): Promise<ExchangeInfo> {
    if (token_0 instanceof SNIP20Contract) token_0 = token_0.asCustomToken
    if (token_1 instanceof SNIP20Contract) token_1 = token_1.asCustomToken
    await this.tx(agent).create_exchange(token_0, token_1)
    return await this.getExchange(token_0, token_1, agent)
  }

}
