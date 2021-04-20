import { 
    Address, TokenPair, IdoInitConfig, Pagination, TokenPairAmount,
    Decimal, Uint128, ContractInfo, get_token_type, TypeOfToken, 
    TokenInfo, ViewingKey, TokenTypeAmount, Exchange
} from './types.js'
import { ExecuteResult, SigningCosmWasmClient, CosmWasmClient } from 'secretjs'

// These two are not exported in secretjs...
export interface Coin {
    readonly denom: string;
    readonly amount: string;
}

export interface Fee {
    readonly amount: ReadonlyArray<Coin>
    readonly gas: Uint128
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

function create_coin(amount: Uint128): Coin {
    return {
        denom: 'uscrt',
        amount
    }
}

export function create_fee(amount: Uint128, gas?: Uint128 | undefined): Fee {
    if (gas === undefined) {
        gas = amount
    }

    return {
        amount: [{ amount, denom: "uscrt" }],
        gas,
    }
}

export class SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) { }

    protected query_client(): CosmWasmClient | SigningCosmWasmClient {
        if(this.client !== undefined) {
            return this.client
        }

        return this.signing_client
    }
}

export class FactoryContract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async create_exchange(pair: TokenPair, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            create_exchange: {
                pair
            }
        }

        if (fee === undefined) {
            fee = create_fee('700000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async create_ido(info: IdoInitConfig, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            create_ido: {
                info
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
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

export class ExchangeContract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async provide_liquidity(amount: TokenPairAmount, tolerance?: Decimal | null, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            add_liquidity: {
                deposit: amount,
                slippage_tolerance: tolerance
            }
        }

        if (fee === undefined) {
            fee = create_fee('3000000')
        }
        
        const transfer = add_native_balance_pair(amount)
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
    }

    async withdraw_liquidity(amount: Uint128, recipient: Address, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            remove_liquidity: {
                amount,
                recipient
            }
        }

        if (fee === undefined) {
            fee = create_fee('2500000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async swap(amount: TokenTypeAmount, expected_return?: Decimal | null, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            swap: {
                offer: amount,
                expected_return
            }
        }

        if (fee === undefined) {
            fee = create_fee('2400000')
        }

        const transfer = add_native_balance(amount)
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
    }

    async get_pair_info(): Promise<TokenPair> {
        const msg = 'pair_info' as unknown as object //yeah...

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetPairInfoResponse
        return result.pair_info
    }

    async get_factory_info(): Promise<ContractInfo> {
        const msg = 'factory_info' as unknown as object

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetFactoryInfoResponse
        return result.factory_info
    }

    async get_pool(): Promise<TokenPairAmount> {
        const msg = 'pool' as unknown as object

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetPoolResponse
        return result.pool
    }

    async simulate_swap(amount: TokenTypeAmount): Promise<SwapSimulationResponse> {
        const msg = {
            swap_simulation: {
                offer: amount
            }
        }
        
        return await this.query_client().queryContractSmart(this.address, msg)
    }
}

export interface GetAllowanceResponse {
    allowance: Allowance
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

export interface Allowance {
    spender: Address,
    owner: Address,
    allowance: Uint128,
    expiration?: number | null
}

export class Snip20Contract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async increase_allowance(
        spender: Address,
        amount: Uint128,
        expiration?: number | null,
        padding?: string | null,
        fee?: Fee | undefined
    ): Promise<ExecuteResult> {
        const msg = {
            increase_allowance: {
                spender,
                amount,
                expiration,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async get_allowance(owner: Address, spender: Address, key: ViewingKey): Promise<Allowance> {
        const msg = {
            allowance: {
                owner,
                spender,
                key
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetAllowanceResponse
        return result.allowance
    }

    async get_balance(address: Address, key: ViewingKey): Promise<Uint128> {
        const msg = {
            balance: {
                address,
                key
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetBalanceResponse
        return result.balance.amount
    }

    async get_token_info(): Promise<TokenInfo> {
        const msg = {
            token_info: { }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg)
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

    async set_viewing_key(key: ViewingKey, padding?: string | null, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            set_viewing_key: {
                key,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }


        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async deposit(amount: Uint128, padding?: string | null, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            deposit: {
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }


        const transfer = [ create_coin(amount) ]
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
    }

    async transfer(recipient: Address, amount: Uint128, padding?: string | null, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            transfer: {
                recipient,
                amount,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async mint(recipient: Address, amount: Uint128, padding?: string | null, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            mint: {
                recipient,
                amount,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
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
