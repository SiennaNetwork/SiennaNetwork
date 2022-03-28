import {
    Address, Uint128, ContractInfo, Fee,
    create_fee, create_coin, ViewingKey
} from '../core'
import { get_token_type, TypeOfToken, CustomToken, TokenType } from '../amm/token'
import { SmartContract, Querier } from '../contract'
import { ViewingKeyExecutor } from '../executors/viewing_key_executor'
import { Snip20Contract } from '../snip20'

import { SigningCosmWasmClient, ExecuteResult } from 'secretjs'

export enum SaleType {
    PreLockAndSwap = "PreLockAndSwap",
    PreLockOnly = "PreLockOnly",
    SwapOnly = "SwapOnly",
}

export class TokenSaleConfig {
    constructor(
        /**
         * The token that will be used to buy the SNIP20.
         */
        readonly input_token: TokenType,
        /**
         * The total amount that each participant is allowed to buy.
         */
        readonly max_allocation: Uint128,
        /**
         * The minimum amount that each participant is allowed to buy.
         */
        readonly min_allocation: Uint128,
        /**
         * The maximum number of participants allowed.
         */
        readonly max_seats: number,
        /**
         * The price for a single token.
         */
        readonly rate: Uint128,
        readonly sold_token: ContractInfo,
        /**
         * The addresses that are eligible to participate in the sale.
         */
        readonly whitelist: Address[],
        /**
         * Sale type settings
         */
        readonly sale_type: SaleType | null
    ) {}
}

export interface SaleInfo {
    /**
     * The token that is used to buy the sold SNIP20.
     */
    input_token: TokenType;
    /**
     * The total amount that each participant is allowed to buy.
     */
    max_allocation: Uint128;
    /**
     * The maximum number of participants allowed.
     */
    max_seats: number;
    /**
     * Number of participants currently.
     */
    taken_seats: number;
    /**
     * The minimum amount that each participant is allowed to buy.
     */
    min_allocation: Uint128;
    /**
     * The conversion rate at which the token is sold.
     */
    rate: Uint128;
    /**
     * The token that is being sold.
     */
    sold_token: ContractInfo;
    /**
     * Sale start time.
     */
    start?: number | null;
    /**
     * Sale end time.
     */
    end?: number | null;
}

export interface SaleStatus {
    available_for_sale: Uint128;
    is_active: boolean;
    total_allocation: Uint128;
}

export interface Balance {
    pre_lock_amount: Uint128;
    total_bought: Uint128;
}

export interface EligibilityInfo {
    can_participate: boolean;
}

export class IdoContract extends SmartContract<IdoExecutor, IdoQuerier> {
    exec(fee?: Fee, memo?: string): IdoExecutor {
        return new IdoExecutor(
            this.address,
            () => this.query.apply(this),
            this.execute_client,
            fee,
            memo
        )
    }
    
    query(): IdoQuerier {
        return new IdoQuerier(this.address, this.query_client)
    }
}

export class IdoExecutor extends ViewingKeyExecutor {
    constructor(
        address: Address,
        private querier: () => IdoQuerier,
        client?: SigningCosmWasmClient,
        fee?: Fee,
        memo?: string,
    ) {
        super(address, client, fee, memo)
    }

    async swap(amount: Uint128, recipient?: Address): Promise<ExecuteResult> {
        const info = await this.querier().get_sale_info()

        if (get_token_type(info.input_token) == TypeOfToken.Native) {
            const msg = {
                swap: {
                    amount,
                    recipient,
                }
            }

            const transfer = [ create_coin(amount) ]
            return this.run(msg, '280000', transfer)
        }

        const fee = this.fee || create_fee('350000')

        const msg = {
            swap: {
                recipient
            }
        }

        const token_addr = (info.input_token as CustomToken).custom_token.contract_addr;
        const snip20 = new Snip20Contract(token_addr, this.client)

        return snip20.exec(fee, this.memo).send(this.address, amount, msg)
    }

    async admin_refund(recipient?: Address): Promise<ExecuteResult> {
        const msg = {
            admin_refund: {
                address: recipient
            }
        }

        return this.run(msg, '300000')
    }

    async admin_claim(recipient?: Address): Promise<ExecuteResult> {
        const msg = {
            admin_claim: {
                address: recipient
            }
        }

        return this.run(msg, '300000')
    }

    async admin_add_addresses(addresses: Address[]): Promise<ExecuteResult> {
        const msg = {
            admin_add_addresses: {
                addresses
            }
        }

        return this.run(msg, '300000')
    }

    async activate(
        sale_amount: Uint128,
        end_time: number,
        start_time?: number
    ): Promise<ExecuteResult> {
        const msg = {
            activate: {
                end_time,
                start_time
            }
        }

        const fee = this.fee || create_fee('300000')
        const info = await this.querier().get_sale_info()

        const snip20 = new Snip20Contract(info.sold_token.address, this.client)
        return snip20.exec(fee, this.memo).send(this.address, sale_amount, msg)
    }
}

export class IdoQuerier extends Querier {
    async get_balance(key: ViewingKey, address: Address): Promise<Balance> {
        const msg = {
            balance: {
                address,
                key,
            },
        };

        const result = await this.run(msg) as BalanceResponse
        return result.balance
    }

    async get_sale_info(): Promise<SaleInfo> {
        const msg = 'sale_info' as unknown as object

        const result = await this.run(msg) as SaleInfoResponse
        return result.sale_info
    }

    async get_sale_status(): Promise<SaleStatus> {
        const msg = 'sale_status' as unknown as object

        const result = await this.run(msg) as SaleStatusResponse
        return result.status
    }

    async get_eligibility_info(address: Address): Promise<EligibilityInfo> {
        const msg = {
            eligibility_info: { address },
        };

        const result = await this.run(msg) as EligibilityInfoResponse;

        return result.eligibility;
    }
}

interface SaleInfoResponse {
    sale_info: SaleInfo
}

interface SaleStatusResponse {
    status: SaleStatus
}

interface BalanceResponse {
    balance: Balance;
}

interface EligibilityInfoResponse {
    eligibility: EligibilityInfo;
}
