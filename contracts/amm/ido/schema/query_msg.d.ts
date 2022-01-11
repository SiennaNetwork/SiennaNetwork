/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type QueryMsg =
  | ("status" | "sale_info" | "sale_status")
  | {
      eligibility_info: {
        address: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      admin: QueryMsg1;
      [k: string]: unknown;
    }
  | {
      balance: {
        address: HumanAddr;
        key: string;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      token_info: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type HumanAddr = string;
export type QueryMsg1 = {
  admin: {
    [k: string]: unknown;
  };
  [k: string]: unknown;
};