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
     * The last time that the user claimed their rewards.
     */
    last_claimed: number;
    /**
     * The amount of LP tokens the owner has locked into this contract.
     */
    locked_amount: Uint128;
    /**
     * The owner of this account.
     */
    owner: Address;
    /**
     * A history of submitted tokens that aren't included in the rewards calculations yet.
     */
    pending_balances?: PendingBalance[] | null;
}

export interface PendingBalance {
    amount: Uint128;
    submitted_at: number;
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
  /**
   * The actual amount of rewards that would be claimed.
   */
   actual_claimed: Uint128;
   error?: ClaimError | null;
   /**
    * The total amount of rewards that should be claimed.
    */
   reward_amount: Uint128;
   /**
    * The reward amount that would be claimed for a single portion.
    */
   reward_per_portion: Uint128;
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
