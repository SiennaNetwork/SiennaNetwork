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
    total_supply?: Uint128 | undefined
}

export interface Exchange {
    pair: TokenPair,
    address: Address
}

export interface PairInfo {
    amount_0: Uint128;
    amount_1: Uint128;
    factory: ContractInfo;
    liquidity_token: ContractInfo;
    pair: TokenPair;
    total_liquidity: Uint128;
}

export class IdoInitConfig {
    constructor(
        /**
         * This is the token that will be used to buy our token.
         */
        readonly input_token: TokenType,
        /**
         * Check this to understand how the rate is set: https://github.com/SiennaNetwork/sienna-swap-amm/blob/b3dc9b21d8f6c11c32d9282ebc1ad5267aa1fa44/ido/src/contract.rs#L277
         */
        readonly rate: Uint128,
        readonly snip20_init_info: Snip20TokenInitInfo
    ) { }
}

export class Snip20TokenInitInfo {
    constructor(
        /**
         * Must be between 3-200 chars length.
         */
        readonly name: string,
        /**
         * Must be between 3-12 chars length, letters only.
         */
        readonly symbol: string,
        /**
         * Must be a base64 encoded string. Otherwise, the tx will fail.
         */
        readonly prng_seed: string,
        /**
         * Max is 18
         */
        readonly decimals: number,
        readonly config?: Snip20InitConfig | null
    ) { }
}

export class Snip20InitConfig {
    public_total_supply?: boolean | null;
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
    /**
     * The reward amount allocated to this pool.
     */
    share: number;
    /**
     * Total amount locked by all participants.
     */
    size: number;
}

export interface RewardsAccount {
    /**
     * The address of the LP token that this account is for.
     */
    lp_token_addr: Address;
    /**
     * The owner of this account.
     */
    owner: Address;
    /**
     * The last time that the user claimed their rewards.
     */
    last_claimed: number;
    /**
     * The amount of LP tokens the owner has locked into this contract.
     */
    locked_amount: number;
}

export type ClaimError =
    | {
        /**
         * Occurs when the rewards pool is currently empty.
         */
        type: "pool_empty";
    }
    | {
        /**
         * Occurs when the user has no tokens locked in this pool. 
         * In practice, this can occur when a wrong address was provided to the query.
         */
        type: "account_zero_locked";
    }
    | {
        /**
         * It is possible for the user's share to be so little, that
         * the actual reward amount of rewards calculated to be zero.
         * However, it is highly unlikely in practice.
         */
        type: "account_zero_reward";
    }
    | {
        /**
         * In Unix seconds.
         */
        time_to_wait: number;
        /**
         * Occurs when the user tries to claim earlier than the designated claim interval.
         */
        type: "early_claim";
    };

export interface ClaimSimulationResult {
    total_rewards_amount: Uint128;
    actual_claimed: Uint128;
    results: ClaimResult[];
}

export interface ClaimResult {
    error?: ClaimError | null;
    lp_token_addr: Address;
    reward_amount: Uint128;
    reward_per_portion: Uint128;
    success: boolean;
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
