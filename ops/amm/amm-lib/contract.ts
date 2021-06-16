import { 
    Address, TokenPair, IdoInitConfig, Pagination, TokenPairAmount,
    Decimal, Uint128, get_token_type, TypeOfToken, 
    TokenInfo, ViewingKey, TokenTypeAmount, Exchange, RewardPool,
    RewardsAccount, PairInfo, Allowance, ClaimSimulationResult,
    ExchangeRate, ContractInstantiationInfo, ExchangeSettings
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

export interface FactoryConfig {
    exchange_settings: ExchangeSettings;
    ido_contract: ContractInstantiationInfo;
    lp_token_contract: ContractInstantiationInfo;
    pair_contract: ContractInstantiationInfo;
    snip20_contract: ContractInstantiationInfo;
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
        if (this.client !== undefined) {
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

    async set_config(
        snip20_contract: ContractInstantiationInfo | undefined,
        lp_token_contract: ContractInstantiationInfo | undefined,
        pair_contract: ContractInstantiationInfo | undefined,
        ido_contract: ContractInstantiationInfo | undefined,
        exchange_settings: ExchangeSettings | undefined,
        fee?: Fee | undefined
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

interface GetPairInfoResponse {
    pair_info: PairInfo
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
            fee = create_fee('390000')
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
            fee = create_fee('350000')
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
            fee = create_fee('450000')
        }

        const transfer = add_native_balance(amount)
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
    }

    async get_pair_info(): Promise<PairInfo> {
        const msg = 'pair_info' as unknown as object //yeah...

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetPairInfoResponse
        return result.pair_info
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

interface GetAllowanceResponse {
    allowance: Allowance
}

interface GetBalanceResponse {
    balance: {
        amount: Uint128
    }
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
            token_info: {}
        }

        const result = await this.query_client().queryContractSmart(this.address, msg)
        return result as TokenInfo
    }

    get_exchange_rate(): ExchangeRate {
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


        const transfer = [create_coin(amount)]
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

interface ClaimSimulationResponse {
    claim_simulation: ClaimSimulationResult;
}

interface GetAccountsResponse {
    accounts: RewardsAccount[];
}

interface GetPoolsResponse {
    pools: RewardPool[];
}

interface GetTotalRewardsSupply {
    total_rewards_supply: {
        amount: Uint128;
    }
}

export class RewardsContract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async claim(lp_tokens: Address[], fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            claim: {
                lp_tokens
            }
        }

        if (fee === undefined) {
            fee = create_fee('300000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async claim_simulation(
        address: Address,
        viewing_key: ViewingKey,
        current_time_secs: number,
        lp_tokens: Address[]
    ): Promise<ClaimSimulationResult> {
        const msg = {
            claim_simulation: {
                address,
                current_time: current_time_secs,
                lp_tokens,
                viewing_key
            }
        };

        const result = await this.query_client().queryContractSmart(this.address, msg) as ClaimSimulationResponse;
        return result.claim_simulation;
    }

    async lock_tokens(amount: Uint128, lp_token: Address, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            lock_tokens: {
                amount,
                lp_token
            }
        }

        if (fee === undefined) {
            fee = create_fee('250000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async retrieve_tokens(amount: Uint128, lp_token: Address, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            retrieve_tokens: {
                amount,
                lp_token
            }
        }

        if (fee === undefined) {
            fee = create_fee('250000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async get_pools(): Promise<RewardPool[]> {
        const msg = 'pools' as unknown as object

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetPoolsResponse;
        return result.pools;
    }

    async get_accounts(
        address: Address,
        lp_tokens: Address[],
        viewing_key: ViewingKey
    ): Promise<RewardsAccount[]> {
        const msg = {
            accounts: {
                address,
                lp_tokens,
                viewing_key
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetAccountsResponse;
        return result.accounts;
    }

    async get_total_rewards_supply(): Promise<Uint128> {
        const msg = 'total_rewards_supply' as unknown as object

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetTotalRewardsSupply;
        return result.total_rewards_supply.amount;
    }
}

function add_native_balance_pair(amount: TokenPairAmount): Coin[] | undefined {
    let result: Coin[] | undefined = []

    if (get_token_type(amount.pair.token_0) == TypeOfToken.Native) {
        result.push({
            denom: 'uscrt',
            amount: amount.amount_0
        })
    }
    else if (get_token_type(amount.pair.token_1) == TypeOfToken.Native) {
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
    let result: Coin[] | undefined = []

    if (get_token_type(amount.token) == TypeOfToken.Native) {
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
