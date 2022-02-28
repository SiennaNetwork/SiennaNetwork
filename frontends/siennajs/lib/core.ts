import { b64encode, b64decode, b64fromBuffer } from "@waiting/base64";
import { Tx, EncryptionUtilsImpl, Coin } from "secretjs";

export type Uint128 = string;
export type Uint256 = string;
export type Address = string;
export type Decimal = string;
export type Decimal256 = string;

/**
 * Base64 encoded
 */
export type ViewingKey = string

export interface Fee {
    readonly amount: ReadonlyArray<Coin>
    readonly gas: Uint128
}

export interface Pagination {
    limit: number;
    start: number;
}

export function decode_data<T>(result: Tx): T {
    const hack = result as any // data field not included in the interface
    const b64string = b64fromBuffer(hack.data)
    
    return JSON.parse(b64decode(b64string))
}

export function create_coin(amount: Uint128): Coin {
    return {
        denom: 'uscrt',
        amount: `${amount}`
    }
}

export function create_fee(amount: Uint128, gas?: Uint128): Fee {
    if (gas === undefined) {
        gas = amount
    }

    return {
        amount: [{ amount: `${amount}`, denom: "uscrt" }],
        gas: `${gas}`,
    }
}

export function create_base64_msg(msg: object): string {
    return b64encode(JSON.stringify(msg))
}

export function create_entropy(): string {
    const rand = EncryptionUtilsImpl.GenerateNewSeed().toString()
    return b64encode(rand)
}

export class ContractInfo {
    constructor(
        readonly code_hash: string,
        readonly address: Address
    ) { }
}

export class ContractInstantiationInfo {
    constructor(
        readonly code_hash: string,
        readonly id: number
    ) { }
}
