import {
    Address, Uint128, Fee, create_fee,
    ViewingKey, create_coin, create_base64_msg
} from './core'
import { SmartContract } from './contract'

import { ExecuteResult, SigningCosmWasmClient, CosmWasmClient } from 'secretjs'

export interface TokenInfo {
    name: string,
    symbol: string,
    decimals: number,
    total_supply?: Uint128 | null
}

export interface Allowance {
    spender: Address,
    owner: Address,
    allowance: Uint128,
    expiration?: number | null
}

export interface ExchangeRate {
    rate: Uint128,
    denom: string
}

export class Snip20Contract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async increase_allowance(
        spender: Address,
        amount: Uint128,
        expiration?: number | null,
        padding?: string | null,
        fee?: Fee
    ): Promise<ExecuteResult> {
        const msg = {
            increase_allowance: {
                spender,
                amount,
                expiration,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async get_allowance(owner: Address, spender: Address, key: ViewingKey): Promise<Allowance> {
        const msg = {
            allowance: {
                owner,
                spender,
                key
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetAllowanceResponse
        return result.allowance
    }

    async get_balance(key: ViewingKey, address?: Address): Promise<Uint128> {
        if (address === undefined) {
            address = this.signing_client.senderAddress
        }

        const msg = {
            balance: {
                address,
                key
            }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetBalanceResponse
        return result.balance.amount
    }

    async get_token_info(): Promise<TokenInfo> {
        const msg = {
            token_info: { }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetTokenInfoResponse
        return result.token_info
    }

    async get_exchange_rate(): Promise<ExchangeRate> {
        const msg = {
            exchange_rate: { }
        }

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetExchangeRateResponse
        return result.exchange_rate
    }

    async set_viewing_key(key: ViewingKey, padding?: string | null, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            set_viewing_key: {
                key,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }


        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async deposit(amount: Uint128, padding?: string | null, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            deposit: {
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }


        const transfer = [create_coin(amount)]
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
    }

    async transfer(recipient: Address, amount: Uint128, padding?: string | null, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            transfer: {
                recipient,
                amount,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }

    async send(
        recipient: Address,
        amount: Uint128,
        msg?: object | null,
        padding?: string | null,
        fee?: Fee
    ): Promise<ExecuteResult> {
        const message = {
            send: {
                recipient,
                amount,
                padding,
                msg: msg ? create_base64_msg(msg) : null
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, message, undefined, undefined, fee)
    }

    async mint(recipient: Address, amount: Uint128, padding?: string | null, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            mint: {
                recipient,
                amount,
                padding
            }
        }

        if (fee === undefined) {
            fee = create_fee('200000')
        }

        return await this.signing_client.execute(this.address, msg, undefined, undefined, fee)
    }
}

interface GetAllowanceResponse {
    allowance: Allowance
}

interface GetBalanceResponse {
    balance: {
        amount: Uint128
    }
}

interface GetExchangeRateResponse {
    exchange_rate: ExchangeRate
}

interface GetTokenInfoResponse {
    token_info: TokenInfo
}
