import { SmartContract, Querier } from '../contract'
import { Fee, Address, Pagination, ContractInfo, Decimal256, Uint256 } from '../core'
import { ViewingKeyComponentExecutor } from '../executors/viewing_key_executor'
import { LendAuth } from './auth'
import { PaginatedResponse } from '.'

import { ExecuteResult } from 'secretjs'

export interface Market {
    contract: ContractInfo,
    /**
     * The symbol of the underlying asset. Note that this is the same as the symbol
     * that the oracle expects, not what the actual token has in its storage.
     */
    symbol: string,
    /**
     * The percentage rate at which tokens can be borrowed given the size of the collateral.
     */
    ltv_ratio: Decimal256
}

/**
 * One of the fields will always be 0, depending on the state of the account.
 */
export interface AccountLiquidity {
    /**
     * The USD value borrowable by the user, before it reaches liquidation.
     */
    liquidity: Uint256,
    /**
     * If > 0 the account is currently below the collateral requirement and is subject to liquidation.
     */
    shortfall: Uint256
}

export interface OverseerConfig {
    /**
     * The discount on collateral that a liquidator receives.
     */
    premium: Decimal256,
    /**
     * The percentage of a liquidatable account's borrow that can be repaid in a single liquidate transaction.
     * If a user has multiple borrowed assets, the close factor applies to any single borrowed asset,
     * not the aggregated value of a user’s outstanding borrowing.
     */
    close_factor: Decimal256
}

export type OverseerPermissions = 'account_info'

export class OverseerContract extends SmartContract<OverseerExecutor, OverseerQuerier> {
    exec(fee?: Fee, memo?: string): OverseerExecutor {
        return new OverseerExecutor(
            this.address,
            this.execute_client,
            fee,
            memo
        )
    }

    query(): OverseerQuerier {
        return new OverseerQuerier(this.address, this.query_client)
    }
}

class OverseerExecutor extends ViewingKeyComponentExecutor {
    async enter_markets(markets: Address[]): Promise<ExecuteResult> {
        const msg = {
            enter: {
                markets
            }
        }

        return this.run(msg, '40000')
    }

    async exit_markets(market_address: Address): Promise<ExecuteResult> {
        const msg = {
            exit: {
                market_address
            }
        }

        return this.run(msg, '50000')
    }
}

class OverseerQuerier extends Querier {
    /**
     * Max limit per page is `30`.
     */
    async markets(pagination: Pagination): Promise<PaginatedResponse<Market>> {
        const msg = {
            markets: {
                pagination
            }
        }

        return this.run(msg)
    }

    async market(address: Address): Promise<Market> {
        const msg = {
            market: {
                address
            }
        }

        return this.run(msg)
    }

    async entered_markets(auth: LendAuth): Promise<Market[]> {
        const msg = {
            entered_markets: {
                method: await auth.create_method<OverseerPermissions>(this.address, 'account_info')
            }
        }

        return this.run(msg)
    }

    async current_liquidity(auth: LendAuth): Promise<AccountLiquidity> {
        const msg = {
            account_liquidity: {
                block: (await this.client.getBlock()).header.height,
                method: await auth.create_method<OverseerPermissions>(this.address, 'account_info'),
                market: null,
                redeem_amount: '0',
                borrow_amount: '0'
            }
        }

        return this.run(msg)
    }

    /**
     * The hypothetical liquidity after a redeem operation from a market.
     * 
     * @param method 
     * @param market - The market to redeem from. Must have been entered that market.
     * @param redeem_amount - The amount to redeem.
     */
    async liquidity_after_redeem(
        auth: LendAuth,
        market: Address,
        redeem_amount: Uint256
    ): Promise<AccountLiquidity> {
        const msg = {
            account_liquidity: {
                block: (await this.client.getBlock()).header.height,
                method: await auth.create_method<OverseerPermissions>(this.address, 'account_info'),
                market,
                redeem_amount,
                borrow_amount: '0'
            }
        }

        return this.run(msg)
    }

    /**
     * The hypothetical liquidity after a borrow operation from a market.
     * 
     * @param method 
     * @param market - The market to borrow from. Must have been entered that market.
     * @param borrow_amount - The amount to borrow.
     */
    async liquidity_after_borrow(
        auth: LendAuth,
        market: Address,
        borrow_amount: Uint256
    ): Promise<AccountLiquidity> {
        const msg = {
            account_liquidity: {
                block: (await this.client.getBlock()).header.height,
                method: await auth.create_method<OverseerPermissions>(this.address, 'account_info'),
                market,
                redeem_amount: '0',
                borrow_amount
            }
        }

        return this.run(msg)
    }

    /**
     * The hypothetical amount that will be seized from a liquidation.
     * 
     * @param borrowed - The market that is being liquidated.
     * @param collateral - The slToken collateral to be seized. 
     * @param repay_amount - The liquidation amount.
     */
    async seize_amount(
        borrowed: Address,
        collateral: Address,
        repay_amount: Uint256
    ): Promise<Uint256> {
        const msg = {
            seize_amount: {
                borrowed,
                collateral,
                repay_amount
            }
        }

        return this.run(msg)
    }

    async config(): Promise<OverseerConfig> {
        const msg = {
            config: { }
        }

        return this.run(msg)
    }
}
