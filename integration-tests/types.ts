export type Uint128 = string;
export type Address = string;
export type TokenType = CustomToken | NativeToken;

export class TokenPair {
    token_0: TokenType;
    token_1: TokenType;
    amount_0: Uint128;
    amount_1: Uint128;
}

export class TokenPairAmount extends TokenPair {
    amount: Uint128;
}

export class TokenTypeAmount {
    token: TokenType;
    amount: Uint128;
}

export class CustomToken {
    token: {
        contract_addr: Address;
        token_code_hash: string;
    };
}

export class NativeToken {
    native_token: {
        denom: string;
    };
}

export class IdoInitConfig {
    /**
     * This is the token that will be used to buy our token.
     */
    input_token: TokenType;
    /**
     * Check this to understand how the rate is set: https://github.com/SiennaNetwork/sienna-swap-amm/blob/b3dc9b21d8f6c11c32d9282ebc1ad5267aa1fa44/ido/src/contract.rs#L277
     */
    rate: Uint128;
    snip20_init_info: Snip20TokenInitInfo;
}

export class Snip20TokenInitInfo {
    name: string;
    symbol: string;
    /**
     * Must be a base64 encoded string. Otherwise, the tx will fail.
     */
    prng_seed: string;
    /**
     * Max is 18
     */
    decimals: number;
    config?: Snip20InitConfig | null;
}

export class Snip20InitConfig {
    public_total_supply?: boolean | null;
}

export class Pagination {
    start: number;
    /**
     * Max is 30.
     */
    limit: number;
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
    code_hash: string;
    address: Address;
}

export class ContractInstantiationInfo {
    code_hash: string;
    id: number;
}