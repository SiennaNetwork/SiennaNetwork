// For running this test from the root directory of Sienna repository type a command:
// ./api-test sienna-test/router.spec.mjs
import { Address, Uint128, Fee, create_fee, Decimal } from '../core'
import { get_token_type, TypeOfToken, CustomToken, TokenTypeAmount, add_native_balance } from './token'
import { SmartContract, Querier, Executor } from '../contract'
import { Snip20Contract } from '../snip20'

import { SigningCosmWasmClient, ExecuteResult } from 'secretjs'
import { Hop } from './hop'
import { ExchangeContract } from './exchange'

export class RouterContract extends SmartContract<RouterExecutor, RouterQuerier> {
    exec(fee?: Fee, memo?: string): RouterExecutor {
        return new RouterExecutor(
            this.address,
            () => this.query.apply(this),
            this.execute_client,
            fee,
            memo
        )
    }
    
    query(): RouterQuerier {
        return new RouterQuerier(this.address, this.query_client)
    }
}

class RouterExecutor extends Executor {
    constructor(
        address: Address,
        private querier: () => RouterQuerier,
        client?: SigningCosmWasmClient,
        fee?: Fee,
        memo?: string,
    ) {
        super(address, client, fee, memo)
    }

    async swap(
        hops: Hop[],
        amount: Uint128,
        recipient: Address,
        expected_return?: Decimal,
    ): Promise<ExecuteResult> {
        if (hops.length === 0) {
            throw new Error("You have to provide hops for the swap to happen")
        }

        const first_hop = hops[0]

        if (hops.length === 1) {
            return (new ExchangeContract(first_hop.pair_address, this.client, this.client))
                .exec()
                .swap(new TokenTypeAmount(first_hop.from_token, amount), recipient, expected_return)
        }

        if (get_token_type(first_hop.from_token) == TypeOfToken.Native) {
            const msg = {
                hops,
                to: recipient,
                expected_return,
            }

            const transfer = add_native_balance(new TokenTypeAmount(first_hop.from_token, amount))
            return this.run(msg, '280000', transfer)
        }

        const msg = {
            hops,
            to: recipient,
            expected_return,
        }

        const fee = this.fee || create_fee('1500000')

        const token_addr = (first_hop.from_token as CustomToken).custom_token.contract_addr;
        const snip20 = new Snip20Contract(token_addr, this.client)

        return snip20.exec(fee, this.memo).send(this.address, amount, msg)
    }
}

class RouterQuerier extends Querier {
    async supported_tokens(): Promise<Address[]> {
        const result = await this.run({ supported_tokens: {} }) as Address[];
        
        return result;        
    }
}
