import { 
    Address, TokenPair, Fee, create_fee,
    ContractInstantiationInfo,
} from './core'
import { SmartContract } from './contract'
import { TokenSaleConfig } from './ido'

import {
    ExecuteResult, SigningCosmWasmClient,
    CosmWasmClient, EnigmaUtils
} from 'secretjs'
import { b64encode } from "@waiting/base64";

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
    constructor(
        readonly start: number,
        /**
         * Max is 30.
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

export class FactoryContract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async create_exchange(pair: TokenPair, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            create_exchange: {
                pair,
                entropy: create_entropy()
            }
        }
        
        if (fee === undefined) {
            fee = create_fee('750000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async create_ido(config: TokenSaleConfig, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            create_ido: {
                info: config,
                entropy: create_entropy()
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async set_config(
        snip20_contract: ContractInstantiationInfo | undefined,
        lp_token_contract: ContractInstantiationInfo | undefined,
        pair_contract: ContractInstantiationInfo | undefined,
        ido_contract: ContractInstantiationInfo | undefined,
        exchange_settings: ExchangeSettings | undefined,
        fee?: Fee
    ): Promise<ExecuteResult> {
        const msg = {
            set_config: {
                snip20_contract,
                lp_token_contract,
                pair_contract,
                ido_contract,
                exchange_settings
            }
        }

        if (fee === undefined) {
            fee = create_fee('150000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async get_exchange_address(pair: TokenPair): Promise<Address> {
        const msg = {
            get_exchange_address: {
                pair
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetExchangeAddressResponse
        return result.get_exchange_address.address
    }

    async list_idos(pagination: Pagination): Promise<Address[]> {
        const msg = {
            list_idos: {
                pagination
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as ListIdosResponse
        return result.list_idos.idos
    }

    async list_exchanges(pagination: Pagination): Promise<Exchange[]> {
        const msg = {
            list_exchanges: {
                pagination
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as ListExchangesResponse
        return result.list_exchanges.exchanges
    }

    async get_config(): Promise<FactoryConfig> {
        const msg = {
            get_config: { }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as FactoryGetConfigResponse
        return result.config
    }
}

interface GetExchangeAddressResponse {
    get_exchange_address: {
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

function create_entropy(): string {
    const rand = EnigmaUtils.GenerateNewSeed().toString()
    return b64encode(rand)
}
