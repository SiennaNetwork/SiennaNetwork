/* tslint:disable */
/**
 * This file was automatically generated by json-schema-to-typescript.
 * DO NOT MODIFY IT BY HAND. Instead, modify the source JSONSchema file,
 * and run json-schema-to-typescript to regenerate this file.
 */

export type HumanAddr = string;
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

export interface InitMsg {
  admin?: HumanAddr | null;
  close_factor: Decimal256;
  entropy: Binary;
  market_contract: ContractInstantiationInfo;
  oracle_contract: ContractInstantiationInfo;
  oracle_source: ContractLink;
  premium: Decimal256;
  prng_seed: Binary;
}
/**
 * Info needed to instantiate a contract.
 */
export interface ContractInstantiationInfo {
  code_hash: string;
  id: number;
}
/**
 * Info needed to talk to a contract instance.
 */
export interface ContractLink {
  address: HumanAddr;
  code_hash: string;
}
