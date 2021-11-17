/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type HandleMsg =
  | "on_lp_token_init"
  | {
      add_liquidity: {
        deposit: TokenPairAmountFor_HumanAddr;
        /**
         * The amount the price moves in a trading pair between when a transaction is submitted and when it is executed. Transactions that exceed this threshold will be rejected.
         */
        slippage_tolerance?: Decimal | null;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      swap: {
        expected_return?: Uint128 | null;
        /**
         * The token type to swap from.
         */
        offer: TokenTypeAmountFor_HumanAddr;
        to?: HumanAddr | null;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      receive: {
        amount: Uint128;
        from: HumanAddr;
        msg?: Binary | null;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      change_factory: {
        contract: ContractLinkFor_HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type Uint128 = string;
export type TokenPairFor_HumanAddr = [TokenTypeFor_HumanAddr, TokenTypeFor_HumanAddr];
export type TokenTypeFor_HumanAddr =
  | {
      custom_token: {
        contract_addr: HumanAddr;
        token_code_hash: string;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      native_token: {
        denom: string;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type HumanAddr = string;
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal(1_000_000_000_000_000_000) == 1.0
 *
 * The greatest possible value that can be represented is 340282366920938463463.374607431768211455 (which is (2^128 - 1) / 10^18)
 */
export type Decimal = string;
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
 */
export type Binary = string;

export interface TokenPairAmountFor_HumanAddr {
  amount_0: Uint128;
  amount_1: Uint128;
  pair: TokenPairFor_HumanAddr;
  [k: string]: unknown;
}
export interface TokenTypeAmountFor_HumanAddr {
  amount: Uint128;
  token: TokenTypeFor_HumanAddr;
  [k: string]: unknown;
}
/**
 * Info needed to talk to a contract instance.
 */
export interface ContractLinkFor_HumanAddr {
  address: HumanAddr;
  code_hash: string;
  [k: string]: unknown;
}
