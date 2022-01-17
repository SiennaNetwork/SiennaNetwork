mod querier;
mod state;

use std::ops::Div;

use lend_shared::{
    fadroma::{
        admin,
        admin::{assert_admin, Admin},
        cosmwasm_std,
        cosmwasm_std::{
            log, to_binary, Api, Binary, CosmosMsg, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, Uint128, WasmMsg,
        },
        derive_contract::*,
        require_admin, Callback, ContractInstantiationInfo, ContractLink, Decimal256, Permit,
        Uint256,
    },
    interfaces::{
        market::query_exchange_rate,
        oracle::{
            query_price, Asset, AssetType, HandleMsg as OracleHandleMsg, InitMsg as OracleInitMsg,
        },
        overseer::{AccountLiquidity, Config, HandleMsg, Market, OverseerPermissions, Pagination},
    },
};

use querier::query_snapshot;
use state::{Borrower, BorrowerId, Constants, Contracts, Markets};

#[contract_impl(path = "lend_shared::interfaces::overseer", component(path = "admin"))]
pub trait Overseer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        close_factor: Decimal256,
        premium: Decimal256,
        oracle_contract: ContractInstantiationInfo,
        oracle_source: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse> {
        BorrowerId::set_prng_seed(&mut deps.storage, &prng_seed)?;

        Contracts::save_oracle(
            deps,
            &ContractLink {
                address: HumanAddr::default(), // Added in RegisterOracle
                code_hash: oracle_contract.code_hash.clone(),
            },
        )?;

        Constants {
            close_factor,
            premium,
        }
        .save(&mut deps.storage)?;

        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        };
        Contracts::save_self_ref(deps, &self_ref)?;

        let time = env.block.time;

        let mut result = admin::DefaultImpl.new(admin.clone(), deps, env)?;
        result.messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: oracle_contract.id,
            callback_code_hash: oracle_contract.code_hash,
            send: vec![],
            label: format!("Sienna Lend Oracle: {}", time),
            msg: to_binary(&OracleInitMsg {
                admin: admin,
                source: oracle_source,
                initial_assets: vec![],
                callback: Callback {
                    contract: self_ref,
                    msg: to_binary(&HandleMsg::RegisterOracle {})?,
                },
            })?,
        }));

        Ok(result)
    }

    #[handle]
    fn register_oracle() -> StdResult<HandleResponse> {
        let mut oracle = Contracts::load_oracle(deps)?;

        if oracle.address != HumanAddr::default() {
            return Err(StdError::unauthorized());
        }

        oracle.address = env.message.sender;
        Contracts::save_oracle(deps, &oracle)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![
                log("action", "register_interest_token"),
                log("oracle_address", oracle.address),
            ],
            data: None,
        })
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
                    assets: vec![Asset {
                        address: market.contract.address,
                        symbol: market.symbol,
                    }],
                })?,
            })],
            log: vec![log("action", "whitelist")],
            data: None,
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
            log: vec![log("action", "enter")],
            data: None,
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
            Uint128::zero(),
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
            log: vec![log("action", "exit")],
            data: None,
        })
    }

    #[query("entered_markets")]
    fn entered_markets(permit: Permit<OverseerPermissions>) -> StdResult<Vec<Market<HumanAddr>>> {
        let self_ref = Contracts::load_self_ref(deps)?;
        let borrower = permit.validate_with_permissions(
            deps,
            self_ref.address,
            vec![OverseerPermissions::AccountInfo],
        )?;

        let borrower = Borrower::new(deps, &borrower)?;

        borrower.list_markets(deps)
    }

    #[query("liquidity")]
    fn account_liquidity(
        permit: Permit<OverseerPermissions>,
        market: Option<HumanAddr>,
        redeem_amount: Uint128,
        borrow_amount: Uint128,
    ) -> StdResult<AccountLiquidity> {
        let self_ref = Contracts::load_self_ref(deps)?;
        let borrower = permit.validate_with_permissions(
            deps,
            self_ref.address,
            vec![OverseerPermissions::AccountInfo],
        )?;

        calc_liquidity(
            deps,
            &Borrower::new(deps, &borrower)?,
            market,
            redeem_amount,
            borrow_amount,
        )
    }

    #[query("can_transfer")]
    fn can_transfer(
        permit: Permit<OverseerPermissions>,
        market: HumanAddr,
        amount: Uint128,
    ) -> StdResult<bool> {
        let self_ref = Contracts::load_self_ref(deps)?;
        let borrower = permit.validate_with_permissions(
            deps,
            self_ref.address,
            vec![OverseerPermissions::AccountInfo],
        )?;

        let borrower = Borrower::new(&deps, &borrower)?;

        // If not entered the market then transfer is allowed.
        if borrower.get_market(&deps, &market).is_err() {
            return Ok(true);
        }

        let result = calc_liquidity(deps, &borrower, Some(market), amount, Uint128::zero())?;

        if result.shortfall > Uint256::zero() {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    #[query("id")]
    fn id(permit: Permit<OverseerPermissions>) -> StdResult<Binary> {
        let self_ref = Contracts::load_self_ref(deps)?;
        let borrower = permit.validate_with_permissions(
            deps,
            self_ref.address,
            vec![OverseerPermissions::Id],
        )?;

        Ok(BorrowerId::new(deps, &borrower)?.into())
    }

    #[query("whitelist")]
    fn markets(pagination: Pagination) -> StdResult<Vec<Market<HumanAddr>>> {
        Markets::list(deps, pagination)
    }

    #[query("config")]
    fn config() -> StdResult<Config> {
        let Constants {
            close_factor,
            premium,
        } = Constants::load(&deps.storage)?;

        Ok(Config {
            close_factor,
            premium,
        })
    }
}

/// Determine what the account liquidity would be if the given amounts were redeemed/borrowed.
fn calc_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    borrower: &Borrower,
    target_asset: Option<HumanAddr>,
    redeem_amount: Uint128,
    borrow_amount: Uint128,
) -> StdResult<AccountLiquidity> {
    let oracle = Contracts::load_oracle(deps)?;
    let target_asset = target_asset.unwrap_or_default();

    let mut total_collateral = Uint256::zero();
    let mut total_borrowed = Uint256::zero();

    for market in borrower.list_markets(deps)? {
        let is_target_asset = target_asset == market.contract.address;

        let snapshot = query_snapshot(&deps.querier, market.contract, borrower.clone().id())?;

        let price = query_price(
            &deps.querier,
            oracle.clone(),
            market.symbol.into(),
            "USD".into(),
            None,
        )?;

        let conversion_factor = ((market.ltv_ratio * snapshot.exchange_rate)? * price.rate)?;
        total_collateral = (Uint256::from(snapshot.sl_token_balance)
            .decimal_mul(conversion_factor)?
            + total_collateral)?;
        total_borrowed =
            (Uint256::from(snapshot.borrow_balance).decimal_mul(price.rate)? + total_borrowed)?;

        if is_target_asset {
            total_borrowed =
                (Uint256::from(redeem_amount).decimal_mul(conversion_factor)? + total_borrowed)?;
            total_borrowed =
                (Uint256::from(borrow_amount).decimal_mul(price.rate)? + total_borrowed)?;
        }
    }

    if total_collateral > total_borrowed {
        Ok(AccountLiquidity {
            liquidity: (total_collateral - total_borrowed)?,
            shortfall: Uint256::zero(),
        })
    } else {
        Ok(AccountLiquidity {
            liquidity: Uint256::zero(),
            shortfall: (total_borrowed - total_collateral)?,
        })
    }
}

/// Calculate number of tokens of collateral asset to seize given an underlying amount
fn calc_seize_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    // borrowed token
    borrowed: HumanAddr,
    // collateral token
    collateral: HumanAddr,
    // the amount of borrowed to convert into collateral
    repay_amount: Uint128,
) -> StdResult<Uint256> {
    let premium = Constants::load(&deps.storage)?.premium;

    //  Read oracle prices for borrowed and collateral markets 
    let oracle = Contracts::load_oracle(deps)?;
    let price_borrowed = query_price(
        &deps.querier,
        oracle.clone(),
        AssetType::Address(borrowed),
        "USD".into(),
        None,
    )?;
    let price_collateral = query_price(
        &deps.querier,
        oracle,
        AssetType::Address(collateral.clone()),
        "USD".into(),
        None,
    )?;

    // Get the exchange rate and calculate the number of collateral tokens to seize
    let (_, market) = Markets::get_by_addr(deps, &collateral)?;
    let exchange_rate = query_exchange_rate(&deps.querier, market.contract)?;
    let ratio = ((premium * price_borrowed.rate)? / (price_collateral.rate * exchange_rate)?)?;

    let seize_tokens = (ratio * Decimal256::from_uint256(repay_amount)?)?.round();
    Ok(seize_tokens)
}
