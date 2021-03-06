/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type HumanAddr = string;
/**
 * Binary is a wrapper around Vec<u8> to add base64 de/serialization with serde. It also adds some helper methods to help encode inline.
 *
 * This is only needed as serde-json-{core,wasm} has a horrible encoding for Vec<u8>
 */
export type Binary = string;
export type TokenPair = [TokenType, TokenType];
export type TokenType =
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

export interface InitMsg {
  callback: Callback;
  entropy: Binary;
  /**
   * Used by the exchange contract to send back its address to the factory on init
   */
  factory_info: ContractLink;
  /**
   * LP token instantiation info
   */
  lp_token_contract: ContractInstantiationInfo;
  /**
   * The tokens that will be managed by the exchange
   */
  pair: TokenPair;
  prng_seed: Binary;
}
/**
 * Info needed to have the other contract respond.
 */
export interface Callback {
  /**
   * Info about the contract requesting the callback.
   */
  contract: ContractLink;
  /**
   * The message to call.
   */
  msg: Binary;
}
/**
 * Info needed to talk to a contract instance.
 */
export interface ContractLink {
  address: HumanAddr;
  code_hash: string;
}
/**
 * Info needed to instantiate a contract.
 */
export interface ContractInstantiationInfo {
  code_hash: string;
  id: number;
}
