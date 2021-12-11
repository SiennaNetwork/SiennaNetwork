/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type QueryAnswer =
  | {
      token_info: {
        decimals: number;
        name: string;
        symbol: string;
        total_supply?: Uint128 | null;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      exchange_rate: {
        denom: string;
        rate: Uint128;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      allowance: {
        allowance: Uint128;
        expiration?: number | null;
        owner: HumanAddr;
        spender: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      balance: {
        amount: Uint128;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      transfer_history: {
        txs: Tx[];
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      viewing_key_error: {
        msg: string;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      minters: {
        minters: HumanAddr[];
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type Uint128 = string;
export type HumanAddr = string;

export interface Tx {
  coins: Coin;
  from: HumanAddr;
  id: number;
  receiver: HumanAddr;
  sender: HumanAddr;
  [k: string]: unknown;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}