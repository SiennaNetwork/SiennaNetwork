import {
    Address, Uint128, Fee, ViewingKey,
    create_coin, create_base64_msg
} from './core'
import { SmartContract, Executor, Querier } from './contract'

import { ExecuteResult } from 'secretjs'

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

export class Snip20Contract extends SmartContract<Snip20Executor, Snip20Querier> {
    exec(fee?: Fee, memo?: string): Snip20Executor {
        return new Snip20Executor(this.address, this.execute_client, fee, memo)
    }

    query(): Snip20Querier {
        return new Snip20Querier(this.address, this.query_client)
    }
}

export class Snip20Executor extends Executor {
    async increase_allowance(
        spender: Address,
        amount: Uint128,
        expiration?: number | null,
        padding?: string | null
    ): Promise<ExecuteResult> {
        const msg = {
            increase_allowance: {
                spender,
                amount,
                expiration,
                padding
            }
        }

        return this.run(msg, '200000')
    }

    async set_viewing_key(key: ViewingKey, padding?: string | null): Promise<ExecuteResult> {
        const msg = {
            set_viewing_key: {
                key,
                padding
            }
        }

        return this.run(msg, '200000')
    }

    async deposit(amount: Uint128, padding?: string | null): Promise<ExecuteResult> {
        const msg = {
            deposit: {
                padding
            }
        }

        const transfer = [create_coin(amount)]
        return this.run(msg, '200000', transfer)
    }

    async transfer(recipient: Address, amount: Uint128, padding?: string | null): Promise<ExecuteResult> {
        const msg = {
            transfer: {
                recipient,
                amount,
                padding
            }
        }

        return this.run(msg, '200000')
    }

    async send(
        recipient: Address,
        amount: Uint128,
        msg?: object | null,
        padding?: string | null
    ): Promise<ExecuteResult> {
        const message = {
            send: {
                recipient,
                amount,
                padding,
                msg: msg ? create_base64_msg(msg) : null
            }
        }

        return this.run(message, '200000')
    }

    async mint(recipient: Address, amount: Uint128, padding?: string | null): Promise<ExecuteResult> {
        const msg = {
            mint: {
                recipient,
                amount,
                padding
            }
        }

        return this.run(msg, '200000')
    }
}

export class Snip20Querier extends Querier {
    async get_allowance(owner: Address, spender: Address, key: ViewingKey): Promise<Allowance> {
        const msg = {
            allowance: {
                owner,
                spender,
                key
            }
        }

        const result = await this.run(msg) as GetAllowanceResponse
        return result.allowance
    }

    async get_balance(key: ViewingKey, address: Address): Promise<Uint128> {
        const msg = {
            balance: {
                address,
                key
            }
        }

        const result = await this.run(msg) as GetBalanceResponse
        return result.balance.amount
    }

    async get_token_info(): Promise<TokenInfo> {
        const msg = {
            token_info: { }
        }

        const result = await this.run(msg) as GetTokenInfoResponse
        return result.token_info
    }

    async get_exchange_rate(): Promise<ExchangeRate> {
        const msg = {
            exchange_rate: { }
        }

        const result = await this.run(msg) as GetExchangeRateResponse
        return result.exchange_rate
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
