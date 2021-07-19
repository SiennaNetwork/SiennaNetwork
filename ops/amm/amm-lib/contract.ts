import { 
    Address, TokenPair, TokenSaleConfig, Pagination, TokenPairAmount,
    Decimal, Uint128, TokenInfo, ViewingKey, TokenTypeAmount, Exchange,
    RewardPool, RewardsAccount, PairInfo, Allowance, ExchangeRate,
    ContractInstantiationInfo, ExchangeSettings, TypeOfToken,
    get_token_type
} from './types.js'
import { ExecuteResult, SigningCosmWasmClient, CosmWasmClient } from 'secretjs'
import { b64encode } from '@waiting/base64'

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
            fee = create_fee('630000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async create_ido(config: TokenSaleConfig, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            create_ido: {
                info: config
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

interface GetVersionResponse {
    version: number
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
                recipient
            }
        }

        if (fee === undefined) {
            fee = create_fee('490000')
        }

        const info = await this.get_pair_info()

        const snip20 = new Snip20Contract(info.liquidity_token.address, this.signing_client)
        return await snip20.send(this.address, amount, create_base64_msg(msg), null, fee)
    }

    async swap(
        amount: TokenTypeAmount,
        recipient?: Address | null,
        expected_return?: Decimal | null,
        fee?: Fee | undefined
    ): Promise<ExecuteResult> {
        if (fee === undefined) {
            fee = create_fee('410000')
        }

        if (get_token_type(amount.token) == TypeOfToken.Native) {
            const msg = {
                swap: {
                    offer: amount,
                    recipient,
                    expected_return
                }
            }

            const transfer = add_native_balance(amount)
            return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
        }

        const msg = {
            swap: {
                recipient,
                expected_return
            }
        }

        const token_addr = (amount.token as any).custom_token.contract_addr;
        const snip20 = new Snip20Contract(token_addr, this.signing_client)

        return await snip20.send(this.address, amount.amount, create_base64_msg(msg), null, fee)
    }

    async get_pair_info(): Promise<PairInfo> {
        const msg = 'pair_info' as unknown as object //yeah...

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetPairInfoResponse
        return result.pair_info
    }

    async get_version(): Promise<number> {
        const msg = 'version' as unknown as object

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetVersionResponse
        return result.version
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

interface GetExchangeRateResponse {
    exchange_rate: ExchangeRate
}

interface GetTokenInfoResponse {
    token_info: TokenInfo
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

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetTokenInfoResponse
        return result.token_info
    }

    async get_exchange_rate(): Promise<ExchangeRate> {
        const msg = {
            exchange_rate: { }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetExchangeRateResponse
        return result.exchange_rate
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

    async send(
        recipient: Address,
        amount: Uint128,
        msg: string | null,
        padding?: string | null,
        fee?: Fee | undefined
    ): Promise<ExecuteResult> {
        const message = {
            send: {
                recipient,
                amount,
                padding,
                msg
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, message, undefined, undefined, fee)
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

interface GetAccountResponse {
    user_info: RewardsAccount;
}

interface GetPoolResponse {
    pool_info: RewardPool;
}

export class RewardsContract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async claim(fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            claim: { }
        }

        if (fee === undefined) {
            fee = create_fee('300000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async lock_tokens(amount: Uint128, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            lock: {
                amount
            }
        }

        if (fee === undefined) {
            fee = create_fee('280000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async retrieve_tokens(amount: Uint128, fee?: Fee | undefined): Promise<ExecuteResult> {
        const msg = {
            retrieve: {
                amount
            }
        }

        if (fee === undefined) {
            fee = create_fee('260000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async get_pool(now: number): Promise<RewardPool> {
        const msg = {
            pool_info: {
                now
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetPoolResponse;
        return result.pool_info;
    }

    async get_account(
        address: Address,
        key: ViewingKey,
        now: number
    ): Promise<RewardsAccount> {
        const msg = {
            user_info: {
                address,
                key,
                now
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetAccountResponse;
        return result.user_info;
    }
}

function create_base64_msg(msg: object): string {
    return b64encode(JSON.stringify(msg))
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
