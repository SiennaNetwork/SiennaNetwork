mod state;
mod querier;

use lend_shared::{
    fadroma::{
        Uint256, Decimal256,
        ContractLink,
        Permit,
        admin,
        admin::{Admin, assert_admin},
        require_admin,
        derive_contract::*,
        cosmwasm_std,
        cosmwasm_std::{
            InitResponse, HandleResponse, HumanAddr,
            CosmosMsg, StdResult, WasmMsg, StdError,
            Extern, Storage, Api, Querier, Binary, 
            Uint128, to_binary, log
        }
    },
    interfaces::{
        overseer::{
            OverseerPermissions, Pagination,
            AccountLiquidity, Config, Market, 
        },
        oracle::{HandleMsg as OracleHandleMsg, PriceAsset, query_price}
    }
};

use state::{Borrower, BorrowerId, Markets, Contracts, Constants};
use querier::query_snapshot;

#[contract_impl(
    path = "lend_shared::interfaces::overseer",
    component(path = "admin")
)]
pub trait Overseer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        close_factor: Decimal256,
        premium: Decimal256
    ) -> StdResult<InitResponse> {
        BorrowerId::set_prng_seed(&mut deps.storage, &prng_seed)?;

        Constants {
            close_factor,
            premium
        }.save(&mut deps.storage)?;

        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        };
        Contracts::save_self_ref(deps, &self_ref)?;

        admin::DefaultImpl.new(admin, deps, env)
    }

    #[handle]
    #[require_admin]
    fn whitelist(market: Market<HumanAddr>) -> StdResult<HandleResponse> {
        market.validate()?;

        Markets::push(deps, &market)?;

        let oracle = Contracts::load_oracle(deps)?;

        Ok(HandleResponse {
            messages: vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: oracle.address,
                callback_code_hash: oracle.code_hash,
                send: vec![],
                msg: to_binary(&OracleHandleMsg::UpdateAssets {
                    assets: vec![PriceAsset {
                        address: market.contract.address,
                        symbol: market.symbol
                    }]
                })?
            })],
            log: vec![
                log("action", "whitelist")
            ],
            data: None
        })
    }

    #[handle]
    fn enter(markets: Vec<HumanAddr>) -> StdResult<HandleResponse> {
        let borrower = Borrower::new(deps, &env.message.sender)?;

        for market in markets {
            let id = Markets::get_id(deps, &market)?;
            borrower.add_market(&mut deps.storage, id)?;
        }

        Ok(HandleResponse {
            messages: vec![],
            log: vec![ log("action", "enter") ],
            data: None
        })
    }

    #[handle]
    fn exit(market_address: HumanAddr) -> StdResult<HandleResponse> {
        let borrower = Borrower::new(deps, &env.message.sender)?;
        let (id, market) = borrower.get_market(deps, &market_address)?;

        // TODO: Maybe calc_liquidity() can be changed to cover this check in order to avoid calling this twice.
        let snapshot = query_snapshot(&deps.querier, market.contract, borrower.clone().id())?;

        if snapshot.borrow_balance != Uint128::zero() {
            return Err(StdError::generic_err("Cannot exit market while borrowing."));
        }

        let liquidity = calc_liquidity(
            deps,
            &borrower,
            Some(market_address),
            snapshot.sl_token_balance,
            Uint128::zero()
        )?;

        if liquidity.shortfall > Uint256::zero() {
            return Err(StdError::generic_err(format!(
                "This account is currently below its target collateral requirement by {}",
                liquidity.shortfall
            )));
        }

        borrower.remove_market(&mut deps.storage, id);

        Ok(HandleResponse {
            messages: vec![],
            log: vec![ log("action", "exit") ],
            data: None
        })
    }

    #[query("entered_markets")]
    fn entered_markets(
        permit: Permit<OverseerPermissions>
    ) -> StdResult<Vec<Market<HumanAddr>>> {
        let self_ref = Contracts::load_self_ref(deps)?;
        let borrower = permit.validate_with_permissions(
            deps,
            self_ref.address,
            vec![ OverseerPermissions::AccountInfo ]
        )?;

        let borrower = Borrower::new(deps, &borrower)?;

        borrower.list_markets(deps)
    }

    #[query("liquidity")]
    fn account_liquidity(
        permit: Permit<OverseerPermissions>,
    ) -> StdResult<AccountLiquidity> {
        let self_ref = Contracts::load_self_ref(deps)?;
        let borrower = permit.validate_with_permissions(
            deps,
            self_ref.address,
            vec![ OverseerPermissions::AccountInfo ]
        )?;

        calc_liquidity(
            deps,
            &Borrower::new(deps, &borrower)?,
            None,
            Uint128::zero(),
            Uint128::zero()
        )
    }

    #[query("whitelist")]
    fn markets(
        pagination: Pagination
    ) -> StdResult<Vec<Market<HumanAddr>>> {
        Markets::list(deps, pagination)
    }

    #[query("config")]
    fn config() -> StdResult<Config> {
        let Constants {
            close_factor,
            premium
        } = Constants::load(&deps.storage)?;

        Ok(Config {
            close_factor,
            premium
        })
    }
}

/// Determine what the account liquidity would be if the given amounts were redeemed/borrowed.
fn calc_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    borrower: &Borrower,
    target_asset: Option<HumanAddr>,
    redeeem_amount: Uint128,
    borrow_amount: Uint128
) -> StdResult<AccountLiquidity> {
    let oracle = Contracts::load_oracle(deps)?;
    let target_asset = target_asset.unwrap_or_default();

    let total_collateral = Uint256::zero();
    let total_borrowed = Uint256::zero();

    for market in borrower.list_markets(deps)? {
        let snapshot = query_snapshot(&deps.querier, market.contract, borrower.clone().id())?;

        let price = query_price(
            &deps.querier,
            oracle.clone(),
            todo!(),
            "USD".into(),
            None
        )?;

        let conversion_factor = ((market.ltv_ratio * snapshot.exchange_rate)? * price.rate)?;
        total_collateral = (Uint256::from(snapshot.sl_token_balance).decimal_mul(conversion_factor)? + total_collateral)?;
        total_borrowed = (Uint256::from(snapshot.borrow_balance).decimal_mul(price.rate)? + total_borrowed)?;

        if target_asset == market.contract.address {
            total_borrowed = (Uint256::from(redeeem_amount).decimal_mul(conversion_factor)? + total_borrowed)?;
            total_borrowed = (Uint256::from(borrow_amount).decimal_mul(price.rate)? + total_borrowed)?;
        }
    }

    if total_collateral > total_borrowed {
        Ok(AccountLiquidity {
            liquidity: (total_collateral - total_borrowed)?,
            shortfall: Uint256::zero()
        })
    } else {
        Ok(AccountLiquidity {
            liquidity: Uint256::zero(),
            shortfall: (total_borrowed - total_collateral)?
        })
    }
}
