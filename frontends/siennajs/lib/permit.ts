import { Address, Fee } from './core';

export interface Permit<T> {
    params: {
        permit_name: string,
        allowed_tokens: Address[],
        chain_id: string,
        permissions: T[],
    },
    signature: Signature
}

// This type is case sensitive!
export interface Signature {
    readonly pub_key: Pubkey
    readonly signature: string
}

export interface Pubkey {
    /**
     * Must be: `tendermint/PubKeySecp256k1`
     */
    readonly type: string
    readonly value: any
}

export interface Signer {
    chain_id: string
    signer: Address
    sign<T>(permit_msg: PermitAminoMsg<T>): Promise<Permit<T>>
}

/**
 * Data used for creating a signature as per the SNIP-24 spec:
 * https://github.com/SecretFoundation/SNIPs/blob/master/SNIP-24.md#permit-content---stdsigndoc
 */
// This type is case sensitive!
export interface SignDoc {
    readonly chain_id: string;
    readonly account_number: string;
    readonly sequence: string;
    readonly fee: Fee;
    readonly msgs: readonly AminoMsg[];
    readonly memo: string;
}

export interface AminoMsg {
    readonly type: string;
    readonly value: any;
}

/**
 * Used as the `value` field of the {@link AminoMsg} type.
 */
export interface PermitAminoMsg<T> {
    permit_name: string,
    allowed_tokens: Address[],
    permissions: T[],
}

/**
 * Helper function to create a {@link SignDoc}. All other fields on that type must be constant.
 */
export function create_sign_doc<T>(chain_id: string, permit_msg: PermitAminoMsg<T>): SignDoc {
    return {
        chain_id,
        account_number: "0", // Must be 0
        sequence: "0", // Must be 0
        fee: {
            amount: [{ denom: "uscrt", amount: "0" }], // Must be 0 uscrt
            gas: "1", // Must be 1
        },
        msgs: [
            {
                type: "query_permit", // Must be "query_permit"
                value: permit_msg,
            },
        ],
        memo: "", // Must be empty
    }
}

export class KeplrSigner implements Signer {
    /**
    * @param {string} chain_id - The id of the chain which permits will be signed for.
    * @param {Address} signer - The address which will do the signing and which will be the address used by the contracts.
    * @param keplr - Must be a pre-configured instance.
    */
    constructor(
        readonly chain_id: string,
        readonly signer: Address,
        readonly keplr: any
    ) { }

    /**
     * @param {PermitAminoMsg} permit_msg - Query specific parameters that will be created by the consuming contract.
     */
    async sign<T>(permit_msg: PermitAminoMsg<T>): Promise<Permit<T>> {
        const { signature } = await this.keplr.signAmino(
            this.chain_id,
            this.signer,
            create_sign_doc(this.chain_id, permit_msg),
            {
                preferNoSetFee: true, // Fee must be 0, so hide it from the user
                preferNoSetMemo: true, // Memo must be empty, so hide it from the user
            }
        );
    
        return {
            params: {
                chain_id: this.chain_id,
                allowed_tokens: permit_msg.allowed_tokens,
                permit_name: permit_msg.permit_name,
                permissions: permit_msg.permissions
            },
            signature
        }
    }
}
