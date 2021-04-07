import { 
    Address, TokenPair, IdoInitConfig, Pagination, TokenPairAmount,
    Decimal, Uint128, ContractInfo, NativeToken
} from './types.js'
import { ExecuteResult, SigningCosmWasmClient } from 'secretjs'

export const FEES = {
    upload: {
        amount: [{ amount: "2000000", denom: "uscrt" }],
        gas: "2000000",
    },
    init: {
        amount: [{ amount: "500000", denom: "uscrt" }],
        gas: "500000",
    },
    exec: {
        amount: [{ amount: "600000", denom: "uscrt" }],
        gas: "600000",
    },
    send: {
        amount: [{ amount: "80000", denom: "uscrt" }],
        gas: "80000",
    },
}

export interface SmartContract {
    readonly client: SigningCosmWasmClient,
    readonly address: Address
}

export interface GetExchangePairResponse {
    get_exchange_pair: {
        pair: TokenPair;
    }
}

export interface GetExchangeAddressResponse {
    get_exchange_address: {
        address: Address;
    }
}

export interface ListIdosResponse {
    list_idos: {
        idos: Address[];
    }
}

export class FactoryContract implements SmartContract {
    constructor(readonly client: SigningCosmWasmClient, readonly address: Address) { }

    async create_exchange(pair: TokenPair): Promise<ExecuteResult> {
        const msg = {
            create_exchange: {
                pair
            }
        }

        return await this.client.execute(this.address, msg)
    }

    async create_ido(info: IdoInitConfig): Promise<ExecuteResult> {
        const msg = {
            create_ido: {
                info
            }
        }

        return await this.client.execute(this.address, msg)
    }
    
    async get_exchange_pair(exchange_addr: Address): Promise<TokenPair> {
        const msg = {
            get_exchange_pair: {
                exchange_addr
            }
        }

        const result = await this.client.queryContractSmart(this.address, msg) as GetExchangePairResponse
        return result.get_exchange_pair.pair
    }

    async get_exchange_address(pair: TokenPair): Promise<Address> {
        const msg = {
            get_exchange_address: {
                pair
            }
        }

        const result = await this.client.queryContractSmart(this.address, msg) as GetExchangeAddressResponse
        return result.get_exchange_address.address
    }

    async list_idos(pagination: Pagination): Promise<Address[]> {
        const msg = {
            pagination: {
                pagination
            }
        }

        const result = await this.client.queryContractSmart(this.address, msg) as ListIdosResponse
        return result.list_idos.idos
    }
}

export interface GetPairInfoResponse {
    pair_info: TokenPair
}

export interface GetFactoryInfoResponse {
    factory_info: ContractInfo
}

export interface GetPoolResponse {
    pool: TokenPairAmount
}

export interface SwapSimulationResponse {
    return_amount: Uint128,
    spread_amount: Uint128,
    commission_amount: Uint128
}

export class ExchangeContract implements SmartContract {
    constructor(readonly client: SigningCosmWasmClient, readonly address: Address) { }

    async provide_liquidity(amount: TokenPairAmount, tolerance?: Decimal | null): Promise<ExecuteResult> {
        const msg = {
            add_liquidity: {
                deposit: amount,
                slippage_tolerance: tolerance
            }
        }

        return await this.client.execute(this.address, msg)
    }

    async withdraw_liquidity(amount: Uint128, recipient: Address): Promise<ExecuteResult> {
        const msg = {
            remove_liquidity: {
                amount,
                recipient
            }
        }

        return await this.client.execute(this.address, msg)
    }

    async swap(amount: TokenPairAmount): Promise<ExecuteResult> {
        const msg = {
            swap: {
                offer: amount
            }
        }

        return await this.client.execute(this.address, msg,)
    }

    async get_pair_info(): Promise<TokenPair> {
        const msg = 'pair_info' as unknown as object //yeah...

        const result = await this.client.queryContractSmart(this.address, msg) as GetPairInfoResponse
        return result.pair_info
    }

    async get_factory_info(): Promise<ContractInfo> {
        const msg = 'factory_info' as unknown as object

        const result = await this.client.queryContractSmart(this.address, msg) as GetFactoryInfoResponse
        return result.factory_info
    }

    async get_pool(): Promise<TokenPairAmount> {
        const msg = 'pool' as unknown as object

        const result = await this.client.queryContractSmart(this.address, msg) as GetPoolResponse
        return result.pool
    }

    async simulate_swap(amount: TokenPairAmount): Promise<SwapSimulationResponse> {
        const msg = {
            swap_simulation: {
                offer: amount
            }
        }

        return await this.client.queryContractSmart(this.address, msg)
    }
}
