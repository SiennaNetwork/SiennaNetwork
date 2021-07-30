export type Uint128 = string;
export type Address = string;
export type TokenType = CustomToken | NativeToken;
export type Decimal = string;
/**
 * Base64 encoded
 */
export type ViewingKey = string

export class TokenPair {
    constructor(
        readonly token_0: TokenType,
        readonly token_1: TokenType
    ) { }
}

export class TokenPairAmount {
    constructor(
        readonly pair: TokenPair,
        readonly amount_0: Uint128,
        readonly amount_1: Uint128
    ) { }
}

export class TokenTypeAmount {
    constructor(
        readonly token: TokenType,
        readonly amount: Uint128
    ) { }
}

export interface CustomToken {
    custom_token: {
        contract_addr: Address;
        token_code_hash: string;
    };
}

export interface NativeToken {
    native_token: {
        denom: string;
    };
}

export interface TokenInfo {
    name: string,
    symbol: string,
    decimals: number,
    total_supply?: Uint128 | null
}

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

export interface PairInfo {
    amount_0: Uint128;
    amount_1: Uint128;
    factory: ContractInfo;
    liquidity_token: ContractInfo;
    pair: TokenPair;
    total_liquidity: Uint128;
    contract_version: number;
}

export class TokenSaleConfig {
    constructor(
        /**
         * The token that will be used to buy the SNIP20.
         */
        readonly input_token: TokenType,
        /**
         * The total amount that each participant is allowed to buy.
         */
        readonly max_allocation: Uint128,
        /**
         * The minimum amount that each participant is allowed to buy.
         */
        readonly min_allocation: Uint128,
        /**
         * The maximum number of participants allowed.
         */
        readonly max_seats: number,
        /**
         * The price for a single token.
         */
        readonly rate: Uint128,
        readonly sold_token: ContractInfo,
        /**
         * The addresses that are eligible to participate in the sale.
         */
        readonly whitelist: Address[]
    ) {}
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

export enum TypeOfToken {
    Native,
    Custom
}

export function get_token_type(token: TokenType): TypeOfToken {
    const raw = token as Object

    if (raw.hasOwnProperty('native_token')) {
        return TypeOfToken.Native
    }

    return TypeOfToken.Custom
}

export interface Allowance {
    spender: Address,
    owner: Address,
    allowance: Uint128,
    expiration?: number | null
}

export interface ExchangeRate {
    rate: Uint128,
    denom: string
}

export interface RewardPool {
    lp_token: ContractInfo;
    reward_token: ContractInfo;
    /**
     * The current reward token balance that this pool has.
     */
    pool_balance: Uint128;
    /**
     * Amount of rewards already claimed.
     */
    pool_claimed: Uint128;
    /**
     * How many blocks does the user have to wait
     * before being able to claim again.
     */
    pool_cooldown: number;
    /**
     * When liquidity was last updated.
     */
    pool_last_update: number;
    /**
     * The total liquidity ever contained in this pool.
     */
    pool_lifetime: Uint128;
    /**
     * How much liquidity is there in the entire pool right now.
     */
    pool_locked: Uint128;
    /**
     * How many blocks does the user need to have provided liquidity for
     * in order to be eligible for rewards.
     */
    pool_threshold: number;
    /**
     * The time for which the pool was not empty.
     */
    pool_liquid: Uint128;
}

export interface RewardsAccount {
    /**
     * When liquidity was last updated.
     */
    pool_last_update: number;
    /**
     * The total liquidity ever contained in this pool.
     */
    pool_lifetime: Uint128;
    /**
     * How much liquidity is there in the entire pool right now.
     */
    pool_locked: Uint128;
    /**
     * The time period for which the user has provided liquidity.
     */
    user_age: number;
    /**
     * How much rewards can the user claim right now.
     */
    user_claimable: Uint128;
    /**
     * How much rewards has the user ever claimed in total.
     */
    user_claimed: Uint128;
    /**
     * How many blocks does the user needs to wait before being able
     * to claim again.
     */
    user_cooldown: number;
    /**
     * How much rewards has the user actually earned
     * in total as of right now.
     */
    user_earned: Uint128;
    /**
     * When the user's share was last updated.
     */
    user_last_update?: number | null;
    /**
     * The accumulator for every block since the last update.
     */
    user_lifetime: Uint128;
    /**
     * The LP token amount that has been locked by this user.
     */
    user_locked: Uint128;
    /**
     * The user's current share of the pool as a percentage
     * with 6 decimals of precision.
     */
    user_share: Uint128;
}

export class ContractInfo {
    constructor(
        readonly code_hash: string,
        readonly address: Address
    ) { }
}

export class ContractInstantiationInfo {
    constructor(
        readonly code_hash: string,
        readonly id: number
    ) { }
}
