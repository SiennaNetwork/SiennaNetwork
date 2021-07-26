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
    pool_balance: Uint128;
    pool_claimed: Uint128;
    pool_cooldown: number;
    pool_last_update: number;
    pool_lifetime: Uint128;
    pool_locked: Uint128;
    pool_threshold: number;
}

export interface RewardsAccount {
    pool_last_update: number;
    pool_lifetime: Uint128;
    pool_locked: Uint128;
    user_age: number;
    user_claimable: Uint128;
    user_claimed: Uint128;
    user_cooldown: number;
    user_earned: Uint128;
    user_last_update?: number | null;
    user_lifetime: Uint128;
    user_locked: Uint128;
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
