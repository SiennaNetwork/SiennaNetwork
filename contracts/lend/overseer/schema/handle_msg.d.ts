/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type HandleMsg =
  | {
      register_oracle: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      whitelist: {
        config: MarketInitConfig;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      register_market: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      enter: {
        markets: HumanAddr[];
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      exit: {
        market_address: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      change_market: {
        ltv_ratio?: Decimal256 | null;
        market: HumanAddr;
        symbol?: string | null;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      set_premium: {
        premium: Decimal256;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      admin: HandleMsg1;
      [k: string]: unknown;
    }
  | {
      auth: HandleMsg1;
      [k: string]: unknown;
    };
/**
 * A fixed-point decimal value with 18 fractional digits, i.e. Decimal256(1_000_000_000_000_000_000) == 1.0 The greatest possible value that can be represented is 115792089237316195423570985008687907853269984665640564039457.584007913129639935 (which is (2^128 - 1) / 10^18)
 */
export type Decimal256 = string;
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
 */
export type Binary = string;
export type HumanAddr = string;
export type HandleMsg1 =
  | {
      change_admin: {
        address: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      accept_admin: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };

export interface MarketInitConfig {
  config: Config;
  entropy: Binary;
  interest_model_contract: ContractLink;
  /**
   * The percentage rate at which tokens can be borrowed given the size of the collateral.
   */
  ltv_ratio: Decimal256;
  prng_seed: Binary;
  /**
   * Symbol of the underlying asset. Must be the same as what the oracle expects.
   */
  token_symbol: string;
  underlying_asset: ContractLink;
  [k: string]: unknown;
}
export interface Config {
  /**
   * Initial exchange rate used when minting the first slTokens (used when totalSupply = 0)
   */
  initial_exchange_rate: Decimal256;
  /**
   * Fraction of interest currently set aside for reserves
   */
  reserve_factor: Decimal256;
  /**
   * Share of seized collateral that is added to reserves
   */
  seize_factor: Decimal256;
}
/**
 * Info needed to talk to a contract instance.
 */
export interface ContractLink {
  address: HumanAddr;
  code_hash: string;
}
