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
      contract_status: {
        status: ContractStatusLevel;
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
        total?: number | null;
        txs: Tx[];
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      transaction_history: {
        total?: number | null;
        txs: RichTx[];
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
export type ContractStatusLevel = "normal_run" | "stop_all_but_redeems" | "stop_all";
export type HumanAddr = string;
export type TxAction =
  | {
      transfer: {
        from: HumanAddr;
        recipient: HumanAddr;
        sender: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      mint: {
        minter: HumanAddr;
        recipient: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      burn: {
        burner: HumanAddr;
        owner: HumanAddr;
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      deposit: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    }
  | {
      redeem: {
        [k: string]: unknown;
      };
      [k: string]: unknown;
    };

export interface Tx {
  block_height?: number | null;
  block_time?: number | null;
  coins: Coin;
  from: HumanAddr;
  id: number;
  memo?: string | null;
  receiver: HumanAddr;
  sender: HumanAddr;
}
export interface Coin {
  amount: Uint128;
  denom: string;
  [k: string]: unknown;
}
export interface RichTx {
  action: TxAction;
  block_height: number;
  block_time: number;
  coins: Coin;
  id: number;
  memo?: string | null;
}