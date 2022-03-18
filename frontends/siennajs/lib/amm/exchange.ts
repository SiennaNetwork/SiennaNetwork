import { 
    Address, Uint128, Fee, create_fee, Decimal, ContractInfo,
} from '../core'
import { 
    TokenPair, add_native_balance_pair, CustomToken,
    add_native_balance, get_token_type, TypeOfToken,
    TokenTypeAmount, TokenPairAmount,
} from './token'
import { SmartContract, Executor, Querier } from '../contract'
import { Snip20Contract } from '../snip20'

import { ExecuteResult, SigningCosmWasmClient } from 'secretjs'

export interface SwapSimulationResponse {
    return_amount: Uint128
    spread_amount: Uint128
    commission_amount: Uint128
}

export interface PairInfo {
    amount_0: Uint128
    amount_1: Uint128
    factory: ContractInfo
    liquidity_token: ContractInfo
    pair: TokenPair
    total_liquidity: Uint128
    contract_version: number
}

export class ExchangeContract extends SmartContract<ExchangeExecutor, ExchangeQuerier> {
    exec(fee?: Fee, memo?: string): ExchangeExecutor {
        return new ExchangeExecutor(
            this.address,
            () => this.query.apply(this),
            this.execute_client,
            fee,
            memo
        )
    }

    query(): ExchangeQuerier {
        return new ExchangeQuerier(this.address, this.query_client)
    }
}

class ExchangeExecutor extends Executor {
    constructor(
        address: Address,
        private querier: () => ExchangeQuerier,
        client?: SigningCosmWasmClient,
        fee?: Fee,
        memo?: string,
    ) {
        super(address, client, fee, memo)
    }

    async provide_liquidity(amount: TokenPairAmount, tolerance?: Decimal): Promise<ExecuteResult> {
        const msg = {
            add_liquidity: {
                deposit: amount,
                slippage_tolerance: tolerance
            }
        }

        const transfer = add_native_balance_pair(amount)

        return this.run(msg, '100000', transfer)
    }

    async withdraw_liquidity(amount: Uint128, recipient: Address): Promise<ExecuteResult> {
        const msg = {
            remove_liquidity: {
                recipient
            }
        }

        const fee = this.fee || create_fee('110000')

        const info = await this.querier().get_pair_info()
        const snip20 = new Snip20Contract(info.liquidity_token.address, this.client)

        return snip20.exec(fee, this.memo).send(this.address, amount, msg)
    }

    async swap(
        amount: TokenTypeAmount,
        recipient?: Address,
        expected_return?: Decimal
    ): Promise<ExecuteResult> {
        if (get_token_type(amount.token) == TypeOfToken.Native) {
            const msg = {
                swap: {
                    offer: amount,
                    to: recipient,
                    expected_return
                }
            }

            const transfer = add_native_balance(amount)
            return this.run(msg, '55000', transfer)
        }

        const msg = {
            swap: {
                to: recipient,
                expected_return
            }
        }

        const fee = this.fee || create_fee('100000')

        const token_addr = (amount.token as CustomToken).custom_token.contract_addr;
        const snip20 = new Snip20Contract(token_addr, this.client)

        return snip20.exec(fee, this.memo).send(this.address, amount.amount, msg)
    }
}

class ExchangeQuerier extends Querier {
    async get_pair_info(): Promise<PairInfo> {
        const msg = 'pair_info' as unknown as object //yeah...

        const result = await this.run(msg) as GetPairInfoResponse
        return result.pair_info
    }

    async simulate_swap(amount: TokenTypeAmount): Promise<SwapSimulationResponse> {
        const msg = {
            swap_simulation: {
                offer: amount
            }
        }

        return this.run(msg)
    }
}

interface GetPairInfoResponse {
    pair_info: PairInfo
}
