/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type ReceiverCallbackMsg =
  | {
      swap: {
        expected_return?: Uint128 | null;
        recipient?: HumanAddr | null;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      remove_liquidity: {
        recipient: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type Uint128 = string;
export type HumanAddr = string;
