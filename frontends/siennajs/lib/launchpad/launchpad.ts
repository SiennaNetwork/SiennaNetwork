import { Address, Uint128, Fee, create_fee, create_coin } from '../core'
import { get_token_type, TypeOfToken, CustomToken, TokenType } from '../amm/token'
import { SmartContract, Querier } from '../contract'
import { ViewingKeyExecutor } from '../executors/viewing_key_executor'
import { Snip20Contract } from '../snip20'

import { SigningCosmWasmClient, ExecuteResult } from 'secretjs'

export type MaybeTokenAddress = Address | null

export interface TokenSettings {
    token_type: TokenType,
    segment: Uint128,
    bounding_period: number,
}

export interface QueryTokenConfig {
    token_type: TokenType,
    segment: Uint128,
    bounding_period: number,
    token_decimals: number,
    locked_balance: Uint128,
}

export interface QueryAccountToken {
    token_type: TokenType,
    balance: Uint128,
    entries: number[],
}

export class LaunchpadContract extends SmartContract<LaunchpadExecutor, LaunchpadQuerier> {
    exec(fee?: Fee, memo?: string): LaunchpadExecutor {
        return new LaunchpadExecutor(
            this.address,
            () => this.query.apply(this),
            this.execute_client,
            fee,
            memo
        )
    }
    
    query(): LaunchpadQuerier {
        return new LaunchpadQuerier(this.address, this.query_client)
    }
}

export class LaunchpadExecutor extends ViewingKeyExecutor {
    tokens?: TokenType[];

    constructor(
        address: Address,
        private querier: () => LaunchpadQuerier,
        client?: SigningCosmWasmClient,
        fee?: Fee,
        memo?: string,
    ) {
        super(address, client, fee, memo)
    }

    private async verify_token_address(address?: Address): Promise<Address | undefined> {
        if (this.tokens === undefined) {
            this.tokens = (await this.querier().info()).map(token => token.token_type);
        }

        for (const token of this.tokens) {
            if (get_token_type(token) == TypeOfToken.Native && !address) {
                return undefined;
            }

            if (
                get_token_type(token) == TypeOfToken.Custom &&
                (token as CustomToken).custom_token.contract_addr === address) {
                return address;
            }
        }

        throw new Error(`Unsupported token address provided for locking`);
    }

    async lock(amount: Uint128, token_address?: Address): Promise<ExecuteResult> {
        token_address = await this.verify_token_address(token_address);

        if (!token_address) {
            const msg = {
                lock: {
                    amount
                }
            }

            return await this.run(msg, "280000", [create_coin(amount)])
        }

        const msg = {
            lock: {}
        }

        const fee = this.fee || create_fee('350000')
        const snip20 = new Snip20Contract(token_address, this.client)
        return snip20.exec(fee, this.memo).send(this.address, amount, msg)
    }

    async unlock(entries: number, token_address?: Address): Promise<ExecuteResult> {
        token_address = await this.verify_token_address(token_address);

        const msg = { unlock: { entries } }

        if (!token_address) {
            return await this.run(msg, "280000")
        }

        const fee = this.fee || create_fee('400000')
        const snip20 = new Snip20Contract(token_address, this.client)
        return snip20.exec(fee, this.memo).send(this.address, '0', msg)
    }

    async admin_add_token(config: TokenSettings) {
        const msg = {
            admin_add_token: {
                config,
            },
        }

        return await this.run(
            msg,
            "3000000"
        )
    }

    async admin_remove_token(index: number, fee?: Fee) {
        const msg = {
            admin_remove_token: {
                index,
            },
        }

        return await this.run(
            msg,
            "3000000"
        )
    }
}

export class LaunchpadQuerier extends Querier {
    async info(): Promise<QueryTokenConfig[]> {
        const msg = "launchpad_info" as unknown as object

        const result = await this.run(msg) as LaunchpadInfoResponse

        return result.launchpad_info
    }

    async user_info(address: Address, key: string): Promise<QueryAccountToken[]> {
        const msg = {
            user_info: {
                address,
                key
            },
        }

        const result = await this.run(msg) as UserInfoResponse

        return result.user_info
    }

    async draw(number: number, tokens: MaybeTokenAddress[]): Promise<Address[]> {
        const msg = {
            draw: {
                tokens,
                number,
                timestamp: parseInt(`${new Date().valueOf() / 1000}`),
            },
        }

        const result = await this.run(msg) as DrawnAddressesResponse

        return result.drawn_addresses
    }
}

interface LaunchpadInfoResponse {
    launchpad_info: QueryTokenConfig[]
}

interface UserInfoResponse {
    user_info: QueryAccountToken[],
}

interface DrawnAddressesResponse {
    drawn_addresses: Address[]
}
