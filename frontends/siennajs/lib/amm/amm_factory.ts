import { Address, Fee, ContractInstantiationInfo, Pagination, create_entropy } from '../core'
import { TokenPair } from './token'
import { SmartContract, Executor, Querier } from '../contract'

import { ExecuteResult } from 'secretjs'

export interface Exchange {
    pair: TokenPair,
    address: Address
}

export interface ExchangeSettings {
    sienna_burner?: Address | undefined;
    sienna_fee: ExchangeFee;
    swap_fee: ExchangeFee;
}

export interface ExchangeFee {
    denom: number;
    nom: number;
}

export interface FactoryConfig {
    lp_token_contract: ContractInstantiationInfo;
    pair_contract: ContractInstantiationInfo;
    exchange_settings: ExchangeSettings;
}

export class AmmFactoryContract extends SmartContract<AmmFactoryExecutor, AmmFactoryQuerier> {
    exec(fee?: Fee, memo?: string): AmmFactoryExecutor {
        return new AmmFactoryExecutor(this.address, this.execute_client, fee, memo)
    }

    query(): AmmFactoryQuerier {
        return new AmmFactoryQuerier(this.address, this.query_client)
    }
}

class AmmFactoryExecutor extends Executor {
    async create_exchange(pair: TokenPair): Promise<ExecuteResult> {
        const msg = {
            create_exchange: {
                pair,
                entropy: create_entropy()
            }
        }

        return this.run(msg, '300000')
    }
}

class AmmFactoryQuerier extends Querier {
    async get_exchange_address(pair: TokenPair): Promise<Address> {
        const msg = {
            get_exchange_address: {
                pair
            }
        }

        const result = await this.run(msg) as GetExchangeAddressResponse
        return result.get_exchange_address.address
    }

    /**
     * Max limit per page is `30`.
     */
    async list_exchanges(pagination: Pagination): Promise<Exchange[]> {
        const msg = {
            list_exchanges: {
                pagination
            }
        }

        const result = await this.run(msg) as ListExchangesResponse
        return result.list_exchanges.exchanges
    }

    async get_config(): Promise<FactoryConfig> {
        const msg = {
            get_config: { }
        }

        const result = await this.run(msg) as FactoryGetConfigResponse
        return result.config
    }
}

interface GetExchangeAddressResponse {
    get_exchange_address: {
        address: Address;
    }
}

interface ListExchangesResponse {
    list_exchanges: {
        exchanges: Exchange[];
    }
}

interface FactoryGetConfigResponse {
    config: FactoryConfig
}
