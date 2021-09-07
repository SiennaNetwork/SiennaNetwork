import {
    Address, Uint128, ContractInfo, TokenType, Fee, create_fee,
    get_token_type, TypeOfToken, CustomToken, create_coin, ViewingKey
} from './core'
import { SmartContract } from './contract'
import { Snip20Contract } from './snip20'

import { SigningCosmWasmClient, CosmWasmClient, ExecuteResult } from 'secretjs'

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
        readonly whitelist: Address[]
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

export class IdoContract extends SmartContract {
    private input_token?: TokenType;

    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async swap(
        amount: Uint128,
        recipient?: Address | null,
        fee?: Fee
    ) : Promise<ExecuteResult> {

        if (this.input_token === undefined) {
            const info = await this.get_sale_info()
            this.input_token = info.input_token;
        }

        if (get_token_type(this.input_token) == TypeOfToken.Native) {
            if (fee === undefined) {
                fee = create_fee('280000')
            }

            const msg = {
                swap: {
                    amount,
                    recipient,
                }
            }

            return await this.signing_client.execute(this.address, msg, undefined, [ create_coin(amount) ], fee)
        }

        if (fee === undefined) {
            fee = create_fee('3500000')
        }

        const msg = {
            swap: {
                recipient
            }
        }

        const token_addr = (this.input_token as CustomToken).custom_token.contract_addr;
        const snip20 = new Snip20Contract(token_addr, this.signing_client)

        return await snip20.send(this.address, amount, msg, null, fee)
    }

    async refund(recipient?: Address | null, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            admin_refund: {
                address: recipient
            }
        }

        if (fee === undefined) {
            fee = create_fee('3000000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async claim(recipient?: Address | null, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            admin_claim: {
                address: recipient
            }
        }

        if (fee === undefined) {
            fee = create_fee('3000000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async add_addresses(addresses: Address[], fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            admin_add_addresses: {
                addresses
            }
        }

        if (fee === undefined) {
            fee = create_fee('3000000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async activate(
        sale_amount: Uint128,
        end_time: number,
        start_time?: number,
        fee?: Fee
    ): Promise<ExecuteResult> {
        if (fee === undefined) {
            fee = create_fee('3000000')
        }

        const msg = {
            activate: {
                end_time,
                start_time
            }
        }

        const info = await this.get_sale_info()

        const snip20 = new Snip20Contract(info.sold_token.address, this.signing_client)
        return await snip20.send(this.address, sale_amount, msg, null, fee)
    }

    async get_balance(key: ViewingKey, address?: Address): Promise<Uint128> {
        // It's the same interface as SNIP20
        const snip20 = new Snip20Contract(this.address, this.signing_client, this.client)
        return await snip20.get_balance(key, address)
    }

    async get_sale_info(): Promise<SaleInfo> {
        const msg = 'sale_info' as unknown as object

        const result = await this.query_client().queryContractSmart(this.address, msg) as SaleInfoResponse
        return result.sale_info
    }

    async get_sale_status(): Promise<SaleStatus> {
        const msg = 'sale_status' as unknown as object

        const result = await this.query_client().queryContractSmart(this.address, msg) as SaleStatusResponse
        return result.status
    }
}

interface SaleInfoResponse {
    sale_info: SaleInfo
}

interface SaleStatusResponse {
    status: SaleStatus
}
