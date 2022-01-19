import type { IAgent } from "@fadroma/scrt";
import { AugmentedScrtContract_1_2, ContractState } from "@fadroma/scrt";
import { randomHex } from "@hackbg/tools";

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

export class FactoryContract extends AugmentedScrtContract_1_2<FactoryTransactions, FactoryQueries> {

  crate        = 'factory'

  name         = 'SiennaAMMFactory'

  Transactions = FactoryTransactions

  Queries      = FactoryQueries

  constructor (options: ContractState & {

    admin?: IAgent

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
        swap_fee: { nom: 28, denom: 1000 },
        sienna_fee: { nom: 2, denom: 10000 },
        sienna_burner: null,
      },
    })

    if (options.admin) {
      this.instantiator = options.admin
      this.initMsg.admin = options.admin.address
    }

    if (options.exchange_settings) {
      Object.assign(this.initMsg, { exchange_settings: options.exchange_settings })
    }

    if (options.contracts) {
      this.contracts = options.contracts
    }

  }

  get contracts (): Promise<FactoryInventory> {
    // type kludge!
    if (this.address) {
      // If this contract has an address query this from the contract state
      return (this.query({'get_config':{}}) as Promise<QueryResponse>).then(response=>{
        const config: FactoryInventory = response.config
        return {
          snip20_contract:    config.snip20_contract,
          pair_contract:      config.pair_contract,
          lp_token_contract:  config.lp_token_contract,
          ido_contract:       config.ido_contract,
          launchpad_contract: config.launchpad_contract,
          router_contract:    config.router_contract
        }
      })
    } else {
      // If it's not deployed yet, return the value from the config
      const initMsg: InitMsg = this.initMsg as InitMsg
      return Promise.resolve({
        snip20_contract:    initMsg.snip20_contract    as ContractInstantiationInfo,
        pair_contract:      initMsg.pair_contract      as ContractInstantiationInfo,
        lp_token_contract:  initMsg.lp_token_contract  as ContractInstantiationInfo,
        ido_contract:       initMsg.ido_contract       as ContractInstantiationInfo,
        launchpad_contract: initMsg.launchpad_contract as ContractInstantiationInfo,
        router_contract:    initMsg.router_contract    as ContractInstantiationInfo
      })
    }
  }

  set contracts (contracts: FactoryInventory|Promise<FactoryInventory>) {
    if (this.address) {
      throw new Error('Use the config method to reconfigure a live contract.')
    } else {
      Promise.resolve(contracts).then(contracts=>{
        for (const contract of Object.keys(contracts)) {
          this.initMsg[contract] = {
            id:        contracts[contract].codeId,
            code_hash: contracts[contract].codeHash,
          }
        }
      })
    }
  }

  get exchanges (): Promise<AMMContract[]> {
    return this.listExchanges().then(exchanges=>
      Promise.all(exchanges.map(async ({ contract, pair }) => {
        const { lp_token } = await this.getExchange(
          pair.token_0,
          pair.token_1
        )
        const lpToken = new LPTokenContract({
          address:  lp_token.address,
          codeHash: lp_token.code_hash
        })
        const exchange = new AMMContract({
          address:  contract.address,
          codeHash: contract.code_hash,
          lpToken
        })
        return exchange
      }))
    )
  }

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
  async getExchange (token_0: TokenType, token_1: TokenType, agent = this.instantiator) {
    const {address} = await this.q(agent).get_exchange_address(token_0, token_1)
    const exchange  = new AMMContract({address, admin: agent, instantiator: agent})
    const {pair_info:{liquidity_token:lp_token}} = await exchange.pairInfo()
    return { exchange: exchange.link, lp_token, token_0, token_1 }
  }

  /** Create a liquidity pool, i.e. an instance of the exchange contract. */
  async createExchange (token_0: TokenType, token_1: TokenType, agent = this.instantiator) {
    await this.tx(agent).create_exchange(token_0, token_1)
    return await this.getExchange(token_0, token_1, agent);
  }

  /** Create an instance of the launchpad contract. */
  createLaunchpad (tokens: object[], agent = this.instantiator) {
    return this.tx(agent).create_launchpad(tokens)
  }

}
