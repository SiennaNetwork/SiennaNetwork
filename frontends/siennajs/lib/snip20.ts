import {
    Address, Uint128, Fee, ViewingKey,
    create_coin, create_base64_msg
} from './core'
import { SmartContract, Querier } from './contract'
import { ViewingKeyExecutor } from './executors/viewing_key_executor'
import { Signer, Permit } from './permit'

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

export type Snip20Permit = Permit<'allowance' | 'balance' | 'history' | 'owner'>

export class Snip20 extends SmartContract<Snip20Executor, Snip20Querier> {
    exec(fee?: Fee, memo?: string): Snip20Executor {
        return new Snip20Executor(this.address, this.execute_client, fee, memo)
    }

    query(): Snip20Querier {
        return new Snip20Querier(this.address, this.query_client)
    }
}

export class Snip20Executor extends ViewingKeyExecutor {
    async increase_allowance(
        spender: Address,
        amount: Uint128,
        expiration?: number,
        padding?: string
    ): Promise<ExecuteResult> {
        const msg = {
            increase_allowance: {
                spender,
                amount,
                expiration,
                padding
            }
        }

        return this.run(msg, '50000')
    }

    async deposit(amount: Uint128, padding?: string): Promise<ExecuteResult> {
        const msg = {
            deposit: {
                padding
            }
        }

        const transfer = [create_coin(amount)]
        return this.run(msg, '50000', transfer)
    }

    async transfer(recipient: Address, amount: Uint128, padding?: string): Promise<ExecuteResult> {
        const msg = {
            transfer: {
                recipient,
                amount,
                padding
            }
        }

        return this.run(msg, '50000')
    }

    async send(
        recipient: Address,
        amount: Uint128,
        msg?: object,
        padding?: string
    ): Promise<ExecuteResult> {
        const message = {
            send: {
                recipient,
                amount,
                padding,
                msg: msg ? create_base64_msg(msg) : undefined
            }
        }

        return this.run(message, '50000')
    }

    async mint(recipient: Address, amount: Uint128, padding?: string): Promise<ExecuteResult> {
        const msg = {
            mint: {
                recipient,
                amount,
                padding
            }
        }

        return this.run(msg, '50000')
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
        
        if (result.viewing_key_error) {
            throw new Error(result.viewing_key_error.msg || "Something went wrong with the viewing key")
        }
        
        return result.balance.amount
    }

    async permit_get_balance(signer: Signer): Promise<Uint128> {
        const msg = create_permit_msg(
            { balance: {} },
            await signer.sign({
                permit_name: `SiennaJS permit for ${this.address}`,
                allowed_tokens: [ this.address ],
                permissions: [ 'balance' ]
            })
        )

        const result = await this.run(msg) as GetBalanceResponse
        
        return result.balance.amount
    }

    /**
     * The address of the signer has to correspond to either `owner` or `spender`.
     */
    async permit_get_allowance(signer: Signer, owner: Address, spender: Address): Promise<Allowance> {
        const msg = create_permit_msg(
            {
                allowance: {
                    owner,
                    spender
                }
            },
            await signer.sign({
                permit_name: `SiennaJS permit for ${this.address}`,
                allowed_tokens: [ this.address ],
                permissions: [ 'balance' ]
            })
        )

        const result = await this.run(msg) as GetAllowanceResponse
        
        return result.allowance
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

function create_permit_msg(query: object, permit: Snip20Permit): object {
    return {
        with_permit: {
            query,
            permit
        }
    }
}

interface GetAllowanceResponse {
    allowance: Allowance
}

interface GetBalanceResponse {
    balance: {
        amount: Uint128
    },
    viewing_key_error?: {
        msg?: string,
    }
}

interface GetExchangeRateResponse {
    exchange_rate: ExchangeRate
}

interface GetTokenInfoResponse {
    token_info: TokenInfo
}
