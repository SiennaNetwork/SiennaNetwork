import { 
    Address, TokenPair, IdoInitConfig, Pagination, TokenPairAmount,
    Decimal, Uint128, ContractInfo, get_token_type, TypeOfToken, 
    TokenInfo, ViewingKey, TokenTypeAmount, Exchange
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

/**
 * This only exists because they didn't bother to
 * export it in secretjs for some reason...
 */
export interface Coin {
    readonly denom: string;
    readonly amount: string;
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

export interface ListExchangesResponse {
    list_exchanges: {
        exchanges: Exchange[];
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

        return await this.client.execute(this.address, msg, undefined, undefined, {
            amount: [{ amount: "700000", denom: "uscrt" }],
            gas: "700000",
        })
    }

    async create_ido(info: IdoInitConfig): Promise<ExecuteResult> {
        const msg = {
            create_ido: {
                info
            }
        }

        return await this.client.execute(this.address, msg)
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
            list_idos: {
                pagination
            }
        }

        const result = await this.client.queryContractSmart(this.address, msg) as ListIdosResponse
        return result.list_idos.idos
    }

    async list_exchanges(pagination: Pagination): Promise<Exchange[]> {
        const msg = {
            list_exchanges: {
                pagination
            }
        }

        const result = await this.client.queryContractSmart(this.address, msg) as ListExchangesResponse
        return result.list_exchanges.exchanges
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

        const transfer = add_native_balance_pair(amount)
        return await this.client.execute(this.address, msg, undefined, transfer)
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

    async swap(amount: TokenTypeAmount, expected_return?: Decimal | null): Promise<ExecuteResult> {
        const msg = {
            swap: {
                offer: amount,
                expected_return
            }
        }

        const transfer = add_native_balance(amount)
        return await this.client.execute(this.address, msg, undefined, transfer)
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

    async simulate_swap(amount: TokenTypeAmount): Promise<SwapSimulationResponse> {
        const msg = {
            swap_simulation: {
                offer: amount
            }
        }
        
        return await this.client.queryContractSmart(this.address, msg)
    }
}

export interface GetAllowanceResponse {
    spender: Address,
    owner: Address,
    allowance: Uint128,
    expiration?: number | undefined
}

export interface GetExchangeRateResponse {
    rate: Uint128,
    denom: string
}

export interface GetBalanceResponse {
    balance: {
        amount: Uint128
    }
}

export class Snip20Contract implements SmartContract {
    constructor(readonly client: SigningCosmWasmClient, readonly address: Address) { }

    async increase_allowance(
        spender: Address,
        amount: Uint128,
        expiration?: number | null,
        padding?: string | null
    ): Promise<ExecuteResult> {
        const msg = {
            increase_allowance: {
                spender,
                amount,
                expiration,
                padding
            }
        }

        return await this.client.execute(this.address, msg)
    }

    async get_allowance(owner: Address, spender: Address, key: ViewingKey): Promise<GetAllowanceResponse> {
        const msg = {
            allowance: {
                owner,
                spender,
                key
            }
        }

        const result = await this.client.queryContractSmart(this.address, msg)
        return result as GetAllowanceResponse
    }

    async get_balance(address: Address, key: ViewingKey): Promise<Uint128> {
        const msg = {
            balance: {
                address,
                key
            }
        }

        const result = await this.client.queryContractSmart(this.address, msg) as GetBalanceResponse
        return result.balance.amount
    }

    async get_token_info(): Promise<TokenInfo> {
        const msg = {
            token_info: { }
        }

        const result = await this.client.queryContractSmart(this.address, msg)
        return result as TokenInfo
    }

    get_exchange_rate(): GetExchangeRateResponse {
        /*
        const msg = {
            exchange_rate: { }
        }

        const result = await this.client.queryContractSmart(this.address, msg)
        return result as GetExchangeRateResponse
        */
        // This is hardcoded in the contract
        return {
            rate: "1",
            denom: "uscrt"
        }
    }

    async set_viewing_key(key: ViewingKey, padding?: string | null): Promise<ExecuteResult> {
        const msg = {
            set_viewing_key: {
                key,
                padding
            }
        }

        return await this.client.execute(this.address, msg)
    }

    async deposit(amount: Uint128, padding?: string | null): Promise<ExecuteResult> {
        const msg = {
            deposit: {
                padding
            }
        }

        const transfer = [ coin(amount) ]
        return await this.client.execute(this.address, msg, undefined, transfer)
    }
}

function add_native_balance_pair(amount: TokenPairAmount): Coin[] | undefined {
    let result: Coin[] | undefined = [ ]

    if(get_token_type(amount.pair.token_0) == TypeOfToken.Native) {
        result.push({
            denom: 'uscrt',
            amount: amount.amount_0
        })
    } 
    else if(get_token_type(amount.pair.token_1) == TypeOfToken.Native) {
        result.push({
            denom: 'uscrt',
            amount: amount.amount_1
        })
    } else {
        result = undefined
    }

    return result
}

function add_native_balance(amount: TokenTypeAmount): Coin[] | undefined {
    let result: Coin[] | undefined = [ ]

    if(get_token_type(amount.token) == TypeOfToken.Native) {
        result.push({
            denom: 'uscrt',
            amount: amount.amount
        })
    } 
    else {
        result = undefined
    }

    return result
}

function coin(amount: Uint128): Coin {
    return {
        denom: 'uscrt',
        amount
    }
}