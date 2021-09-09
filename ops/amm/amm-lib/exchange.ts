import { 
    Address, Uint128, Fee, create_fee, TokenTypeAmount, TokenPairAmount,
    Decimal, ContractInfo, TokenPair, add_native_balance_pair, CustomToken,
    add_native_balance, get_token_type, TypeOfToken, 
} from './core'
import { SmartContract } from './contract'
import { Snip20Contract } from './snip20'

import { ExecuteResult, SigningCosmWasmClient, CosmWasmClient } from 'secretjs'

export interface SwapSimulationResponse {
    return_amount: Uint128,
    spread_amount: Uint128,
    commission_amount: Uint128
}

export interface PairInfo {
    amount_0: Uint128;
    amount_1: Uint128;
    factory: ContractInfo;
    liquidity_token: ContractInfo;
    pair: TokenPair;
    total_liquidity: Uint128;
    contract_version: number;
}

export class ExchangeContract extends SmartContract {
    constructor(
        readonly address: Address,
        readonly signing_client: SigningCosmWasmClient,
        readonly client?: CosmWasmClient | undefined
    ) {
        super(address, signing_client, client)
    }

    async provide_liquidity(amount: TokenPairAmount, tolerance?: Decimal | null, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            add_liquidity: {
                deposit: amount,
                slippage_tolerance: tolerance
            }
        }

        if (fee === undefined) {
            fee = create_fee('530000')
        }

        const transfer = add_native_balance_pair(amount)
        return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
    }

    async withdraw_liquidity(amount: Uint128, recipient: Address, fee?: Fee): Promise<ExecuteResult> {
        const msg = {
            remove_liquidity: {
                recipient
            }
        }

        if (fee === undefined) {
            fee = create_fee('610000')
        }

        const info = await this.get_pair_info()

        const snip20 = new Snip20Contract(info.liquidity_token.address, this.signing_client)
        return await snip20.send(this.address, amount, msg, null, fee)
    }

    async swap(
        amount: TokenTypeAmount,
        recipient?: Address | null,
        expected_return?: Decimal | null,
        fee?: Fee
    ): Promise<ExecuteResult> {
        if (get_token_type(amount.token) == TypeOfToken.Native) {
            if (fee === undefined) {
                fee = create_fee('280000')
            }

            const msg = {
                swap: {
                    offer: amount,
                    recipient,
                    expected_return
                }
            }

            const transfer = add_native_balance(amount)
            return await this.signing_client.execute(this.address, msg, undefined, transfer, fee)
        }

        if (fee === undefined) {
            fee = create_fee('530000')
        }

        const msg = {
            swap: {
                recipient,
                expected_return
            }
        }

        const token_addr = (amount.token as CustomToken).custom_token.contract_addr;
        const snip20 = new Snip20Contract(token_addr, this.signing_client)

        return await snip20.send(this.address, amount.amount, msg, null, fee)
    }

    async get_pair_info(): Promise<PairInfo> {
        const msg = 'pair_info' as unknown as object //yeah...

        const result = await this.query_client().queryContractSmart(this.address, msg) as GetPairInfoResponse
        return result.pair_info
    }

    async simulate_swap(amount: TokenTypeAmount): Promise<SwapSimulationResponse> {
        const msg = {
            swap_simulation: {
                offer: amount
            }
        }

        return await this.query_client().queryContractSmart(this.address, msg)
    }
}

interface GetPairInfoResponse {
    pair_info: PairInfo
}
