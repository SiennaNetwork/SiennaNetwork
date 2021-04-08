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
    ) {}
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
    ) {}
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

/***********************************************************************************
 * The types below are used in integration tests only and are not needed by the UI.*
 **********************************************************************************/

export class ContractInfo {
    constructor(
        readonly code_hash: string,
        readonly address: Address
    ) {}
}

export class ContractInstantiationInfo {
    constructor(
        readonly code_hash: string,
        readonly id: number
    ) {}
}