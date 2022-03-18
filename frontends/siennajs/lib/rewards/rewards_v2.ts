import { Address, Uint128, Fee, ContractInfo, ViewingKey } from '../core'
import { SmartContract, Querier } from '../contract'
import { ViewingKeyExecutor } from '../executors/viewing_key_executor'

import { ExecuteResult } from 'secretjs'

export interface RewardPool {
    lp_token: ContractInfo;
    reward_token: ContractInfo;
    /**
     * The current reward token balance that this pool has.
     */
    pool_balance: Uint128;
    /**
     * Amount of rewards already claimed.
     */
    pool_claimed: Uint128;
    /**
     * How many blocks does the user have to wait
     * before being able to claim again.
     */
    pool_cooldown: number;
    /**
     * When liquidity was last updated.
     */
    pool_last_update: number;
    /**
     * The total liquidity ever contained in this pool.
     */
    pool_lifetime: Uint128;
    /**
     * How much liquidity is there in the entire pool right now.
     */
    pool_locked: Uint128;
    /**
     * How many blocks does the user need to have provided liquidity for
     * in order to be eligible for rewards.
     */
    pool_threshold: number;
    /**
     * The time for which the pool was not empty.
     */
    pool_liquid: Uint128;
}

export interface RewardsAccount {
    /**
     * When liquidity was last updated.
     */
    pool_last_update: number;
    /**
     * The total liquidity ever contained in this pool.
     */
    pool_lifetime: Uint128;
    /**
     * How much liquidity is there in the entire pool right now.
     */
    pool_locked: Uint128;
    /**
     * The time period for which the user has provided liquidity.
     */
    user_age: number;
    /**
     * How much rewards can the user claim right now.
     */
    user_claimable: Uint128;
    /**
     * How much rewards has the user ever claimed in total.
     */
    user_claimed: Uint128;
    /**
     * How many blocks does the user needs to wait before being able
     * to claim again.
     */
    user_cooldown: number;
    /**
     * How much rewards has the user actually earned
     * in total as of right now.
     */
    user_earned: Uint128;
    /**
     * When the user's share was last updated.
     */
    user_last_update?: number | null;
    /**
     * The accumulator for every block since the last update.
     */
    user_lifetime: Uint128;
    /**
     * The LP token amount that has been locked by this user.
     */
    user_locked: Uint128;
    /**
     * The user's current share of the pool as a percentage
     * with 6 decimals of precision.
     */
    user_share: Uint128;
}

export class RewardsV2Contract extends SmartContract<RewardsV2Executor, RewardsV2Querier> {
    exec(fee?: Fee, memo?: string): RewardsV2Executor {
        return new RewardsV2Executor(this.address, this.execute_client, fee, memo)
    }

    query(): RewardsV2Querier {
        return new RewardsV2Querier(this.address, this.query_client)
    }
}

class RewardsV2Executor extends ViewingKeyExecutor {
    async claim(): Promise<ExecuteResult> {
        const msg = {
            claim: { }
        }

        return this.run(msg, '80000')
    }

    async lock_tokens(amount: Uint128): Promise<ExecuteResult> {
        const msg = {
            lock: {
                amount
            }
        }

        return this.run(msg, '75000')
    }

    async retrieve_tokens(amount: Uint128,): Promise<ExecuteResult> {
        const msg = {
            retrieve: {
                amount
            }
        }

        return this.run(msg, '75000')
    }
}

class RewardsV2Querier extends Querier {
    async get_pool(at: number): Promise<RewardPool> {
        const msg = {
            pool_info: {
                at
            }
        }

        const result = await this.run(msg) as GetPoolResponse;
        return result.pool_info;
    }

    async get_account(
        address: Address,
        key: ViewingKey,
        at: number
    ): Promise<RewardsAccount> {
        const msg = {
            user_info: {
                address,
                key,
                at
            }
        }

        const result = await this.run(msg) as GetAccountResponse;
        return result.user_info;
    }
}

interface GetAccountResponse {
    user_info: RewardsAccount;
}

interface GetPoolResponse {
    pool_info: RewardPool;
}
