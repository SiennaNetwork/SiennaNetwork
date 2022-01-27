import { Scrt_1_2, ContractState, Agent, randomHex, bold, Console } from "@hackbg/fadroma"
import { SNIP20Contract } from '@fadroma/snip20'

import { AMMContract        } from "@sienna/exchange";
import { AMMSNIP20Contract  } from "@sienna/amm-snip20";
import { LPTokenContract    } from "@sienna/lp-token";
import { IDOContract        } from "@sienna/ido";
import { LaunchpadContract  } from "@sienna/launchpad";

import { InitMsg, ExchangeSettings, ContractInstantiationInfo } from './schema/init_msg.d';
import { TokenType } from './schema/handle_msg.d';
import { QueryResponse, Exchange } from './schema/query_response.d'

export type FactoryInventory = {
  snip20_contract?:    ContractInstantiationInfo
  pair_contract?:      ContractInstantiationInfo
  lp_token_contract?:  ContractInstantiationInfo
  ido_contract?:       ContractInstantiationInfo
  launchpad_contract?: ContractInstantiationInfo
  router_contract?:    ContractInstantiationInfo
}

import { FactoryTransactions } from './FactoryTransactions'
import { FactoryQueries }      from './FactoryQueries'

/** An exchange is an interaction between 4 contracts. */
export type ExchangeInfo = {
  /** Shorthand to refer to the whole group. */
  name?: string
  /** One token. */
  TOKEN_0:  SNIP20Contract|string,
  /** Another token. */
  TOKEN_1:  SNIP20Contract|string,
  /** The automated market maker/liquidity pool for the token pair. */
  EXCHANGE: AMMContract,
  /** The liquidity provision token, which is minted to stakers of the 2 tokens. */
  LP_TOKEN: LPTokenContract,
  /** The bare-bones data needed to retrieve the above. */
  raw:      any
}

const console = Console('@sienna/amm/factory')

export class FactoryContract extends Scrt_1_2.Contract<FactoryTransactions, FactoryQueries> {
  crate        = 'factory'
  name         = 'SiennaAMMFactory'
  Transactions = FactoryTransactions
  Queries      = FactoryQueries
  admin?: Agent
  constructor (options: ContractState & {
    admin?: Agent
    /* AMM config from project settings.
     * First auto-generated type definition
     * finally put to use in the codebase! Hooray */
    exchange_settings?:  ExchangeSettings
    /* Contract contracts (id + codehash)
     * for each contract known by the factory */
    contracts?: FactoryInventory
  } = {}) {
    super(options)
    Object.assign(this.initMsg, {
      prng_seed: randomHex(36),
      exchange_settings: {
        swap_fee:   { nom: 28, denom:  1000 },
        sienna_fee: { nom:  2, denom: 10000 },
        sienna_burner: null,
      }
    })
    if (options.admin) {
      this.creator = options.admin
      this.initMsg.admin = options.admin.address
    }
    if (options.exchange_settings) {
      Object.assign(this.initMsg, { exchange_settings: options.exchange_settings })
    }
    if (options.contracts) {
      this.setContracts(options.contracts)
    }
  }
  /** Return the collection of contract templates
    * (`{ id, code_hash }` structs) that the factory
    * uses to instantiate contracts. */
  getContracts (): Promise<FactoryInventory> {
    // type kludge!
    if (this.address) {
      // If this contract has an address query this from the contract state
      return (this.q().get_config()).then((config: FactoryInventory)=>({
        snip20_contract:    config.snip20_contract,
        pair_contract:      config.pair_contract,
        lp_token_contract:  config.lp_token_contract,
        ido_contract:       config.ido_contract,
        launchpad_contract: config.launchpad_contract,
      }))
    } else {
      // If it's not deployed yet, return the value from the config
      const initMsg: InitMsg = this.initMsg as InitMsg
      return Promise.resolve({
        snip20_contract:    initMsg.snip20_contract    as ContractInstantiationInfo,
        pair_contract:      initMsg.pair_contract      as ContractInstantiationInfo,
        lp_token_contract:  initMsg.lp_token_contract  as ContractInstantiationInfo,
        ido_contract:       initMsg.ido_contract       as ContractInstantiationInfo,
        launchpad_contract: initMsg.launchpad_contract as ContractInstantiationInfo,
      })
    }
  }
  /** Configure the factory's contract templates before deployment. */
  setContracts (contracts: FactoryInventory) {
    if (this.address) {
      throw new Error('Use the config method to reconfigure a live contract.')
    } else {
      for (const [name, contract] of Object.entries(contracts)) {
        this.initMsg[name] = contract
      }
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
    agent = this.creator
  ): Promise<ExchangeInfo> {
    //console.info(bold('Looking for exchange'))
    //console.info(bold('  between'), JSON.stringify(token_0))
    //console.info(bold('      and'), JSON.stringify(token_1))
    const { admin, prefix, chain } = this
    const { address } = await this.q(agent).get_exchange_address(
      token_0, token_1
    )
    // ouch, factory-created contracts have nasty labels
    const label = await agent.getLabel(address)
    const EXCHANGE = new AMMContract({
      chain,
      address,
      codeHash: await agent.getCodeHash(address),
      codeId:   await agent.getCodeId(address),
      prefix,
      admin,
    })

    const TOKEN_0 = SNIP20Contract.fromTokenSpec(agent, token_0)
    let TOKEN_0_NAME: string
    if (TOKEN_0 instanceof SNIP20Contract) {
      const TOKEN_0_INFO = await TOKEN_0.q(agent).tokenInfo()
      TOKEN_0_NAME = TOKEN_0_INFO.symbol
    } else {
      TOKEN_0_NAME = 'SCRT'
    }

    const TOKEN_1 = SNIP20Contract.fromTokenSpec(agent, token_1)
    let TOKEN_1_NAME: string
    if (TOKEN_1 instanceof SNIP20Contract) {
      const TOKEN_1_INFO = await TOKEN_1.q(agent).tokenInfo()
      TOKEN_1_NAME = TOKEN_1_INFO.symbol
    } else {
      TOKEN_1_NAME = 'SCRT'
    }

    const name = `${TOKEN_0_NAME}-${TOKEN_1_NAME}`

    const { liquidity_token } = await EXCHANGE.pairInfo()
    const LP_TOKEN = new LPTokenContract({
      admin:    this.admin,
      prefix:   this.prefix,
      chain:    this.chain,
      address:  liquidity_token.address,
      codeHash: liquidity_token.code_hash,
      codeId:   await this.admin.getCodeId(liquidity_token.address)
    })

    const raw = {
      exchange: {
        address: EXCHANGE.address
      },
      token_0,
      token_1,
      lp_token: {
        address:   LP_TOKEN.address,
        code_hash: LP_TOKEN.codeHash
      },
    }

    return {
      name,
      EXCHANGE, TOKEN_0, TOKEN_1, LP_TOKEN,
      raw
    }
  }

  /** Create a liquidity pool, i.e. an instance of the exchange contract. */
  async createExchange (
    token_0: TokenType,
    token_1: TokenType,
    agent = this.agent
  ): Promise<ExchangeInfo> {
    await this.tx(agent).create_exchange(token_0, token_1)
    return await this.getExchange(token_0, token_1, agent)
  }

  /** Create an instance of the launchpad contract. */
  createLaunchpad (
    tokens: object[],
    agent = this.agent
  ) {
    return this.tx(agent).create_launchpad(tokens)
  }

}
