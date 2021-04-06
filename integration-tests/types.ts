export type Uint128 = string;
export type Address = string;
export type TokenType = CustomToken | NativeToken;

export class TokenPair {
    constructor(
        readonly token_0: TokenType,
        readonly token_1: TokenType
    ) {}
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
    ) {}
}

export interface CustomToken {
    token: {
        contract_addr: Address;
        token_code_hash: string;
    };
}

export interface NativeToken {
    native_token: {
        denom: string;
    };
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
        readonly name: string,
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

/**
 * Just to show what the response could look like.
 */
export type FactoryQueryResponse =
  | {
      get_exchange_pair: {
        pair: TokenPair;
      };
    }
  | {
      get_exchange_address: {
        address: Address;
      };
    }
  | {
      list_idos: {
        idos: Address[];
      };
    };

export class FactoryContract {
    static HandleMsg = class {
        static create_exchange(pair: TokenPair): object {
            return {
                create_exchange: {
                    pair
                }
            }
        }

        static create_ido(info: IdoInitConfig): object {
            return {
                create_ido: {
                    info
                }
            }
        }
    }

    static QueryMsg = class {
        static get_exchange_pair(exchange_addr: Address): object {
            return {
                get_exchange_pair: {
                    exchange_addr
                }
            }
        }

        static get_exchange_address(pair: TokenPair): object {
            return {
                get_exchange_address: {
                    pair
                }
            }
        }

        static list_idos(pagination: Pagination): object {
            return {
                pagination: {
                    pagination
                }
            }
        }
    }
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