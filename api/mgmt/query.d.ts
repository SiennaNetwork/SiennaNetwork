/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type Query =
  | {
      status: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      schedule: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      history: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      progress: {
        address: HumanAddr;
        time: number;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };
export type HumanAddr = string;
