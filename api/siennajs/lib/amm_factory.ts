import { Address, TokenPair, Fee, ContractInstantiationInfo, create_entropy } from './core'
import { SmartContract, Executor, Querier } from './contract'
import { TokenSaleConfig } from './ido'
import { TokenSettings } from './launchpad'

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

export class Pagination {
    static readonly MAX_LIMIT = 30;

    constructor(
        readonly start: number,
        /**
         * Max is {@link Pagination.MAX_LIMIT}.
         */
        readonly limit: number
    ) { }
}

export interface FactoryConfig {
    exchange_settings: ExchangeSettings;
    ido_contract: ContractInstantiationInfo;
    lp_token_contract: ContractInstantiationInfo;
    pair_contract: ContractInstantiationInfo;
    snip20_contract: ContractInstantiationInfo;
}

export class AmmFactoryContract extends SmartContract<AmmFactoryExecutor, AmmFactoryQuerier> {
    exec(fee?: Fee, memo?: string): AmmFactoryExecutor {
        return new AmmFactoryExecutor(this.address, this.execute_client, fee, memo)
    }

    query(): AmmFactoryQuerier {
        return new AmmFactoryQuerier(this.address, this.query_client)
    }
}

export class AmmFactoryExecutor extends Executor {
    async create_exchange(pair: TokenPair): Promise<ExecuteResult> {
        const msg = {
            create_exchange: {
                pair,
                entropy: create_entropy()
            }
        }

        return this.run(msg, '750000')
    }

    async create_ido(config: TokenSaleConfig): Promise<ExecuteResult> {
        const msg = {
            create_ido: {
                info: config,
                entropy: create_entropy()
            }
        }

        return this.run(msg, '200000')
    }

    async create_launchpad(tokens: TokenSettings[]): Promise<ExecuteResult> {
        const msg = {
            create_launchpad: {
                tokens,
                entropy: create_entropy()
            }
        }

        return this.run(msg, '200000')
    }

    async set_config(
        snip20_contract: ContractInstantiationInfo | undefined,
        lp_token_contract: ContractInstantiationInfo | undefined,
        pair_contract: ContractInstantiationInfo | undefined,
        launchpad_contract: ContractInstantiationInfo | undefined,
        ido_contract: ContractInstantiationInfo | undefined,
        exchange_settings: ExchangeSettings | undefined
    ): Promise<ExecuteResult> {
        const msg = {
            set_config: {
                snip20_contract,
                lp_token_contract,
                pair_contract,
                launchpad_contract,
                ido_contract,
                exchange_settings
            }
        }

        return this.run(msg, '150000')
    }
}

export class AmmFactoryQuerier extends Querier {
    async get_exchange_address(pair: TokenPair): Promise<Address> {
        const msg = {
            get_exchange_address: {
                pair
            }
        }

        const result = await this.run(msg) as GetExchangeAddressResponse
        return result.get_exchange_address.address
    }

    async get_launchpad_address(): Promise<Address> {
        const msg = "get_launchpad_address" as unknown as object

        const result = await this.run(msg) as GetLaunchpadAddressResponse
        return result.get_launchpad_address.address
    }

    async list_idos(pagination: Pagination): Promise<Address[]> {
        const msg = {
            list_idos: {
                pagination
            }
        }

        const result = await this.run(msg) as ListIdosResponse
        return result.list_idos.idos
    }

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

interface GetLaunchpadAddressResponse {
    get_launchpad_address: {
        address: Address;
    }
} 

interface ListIdosResponse {
    list_idos: {
        idos: Address[];
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
