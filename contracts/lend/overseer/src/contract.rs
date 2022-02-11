mod state;

use lend_shared::{
    core::{AuthenticatedUser, MasterKey},
    fadroma::{
        admin,
        admin::{assert_admin, Admin},
        auth, cosmwasm_std,
        cosmwasm_std::{
            log, to_binary, Api, Binary, CosmosMsg, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
        },
        derive_contract::*,
        require_admin,
        Callback, ContractInstantiationInfo, ContractLink, Decimal256, Humanize, Uint256,
    },
    interfaces::{
        market::{query_account, query_exchange_rate, InitMsg as MarketInitMsg, MarketAuth},
        oracle::{
            query_price, Asset, AssetType, HandleMsg as OracleHandleMsg, InitMsg as OracleInitMsg,
        },
        overseer::{
            AccountLiquidity, Config, HandleMsg, Market, MarketInitConfig, OverseerAuth,
            OverseerPermissions, Pagination,
        },
    },
};

use state::{Account, Constants, Contracts, Markets, Whitelisting};

const QUOTE_SYMBOL: &str = "USD";

#[contract_impl(
    entry,
    path = "lend_shared::interfaces::overseer",
    component(path = "admin"),
    component(path = "auth", skip(query))
)]
pub trait Overseer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        entropy: Binary,
        close_factor: Decimal256,
        premium: Decimal256,
        market_contract: ContractInstantiationInfo,
        oracle_contract: ContractInstantiationInfo,
        oracle_source: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse> {
        MasterKey::new(&env, prng_seed.as_slice(), entropy.as_slice()).save(&mut deps.storage)?;

        Contracts::save_oracle(
            deps,
            &ContractLink {
                address: HumanAddr::default(), // Added in RegisterOracle
                code_hash: oracle_contract.code_hash.clone(),
            },
        )?;

        Constants::save(&mut deps.storage, &Config::new(premium, close_factor)?)?;

        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        };
        Contracts::save_self_ref(deps, &self_ref)?;

        Whitelisting::save_market_contract(&mut deps.storage, &market_contract)?;

        let time = env.block.time;

        let mut result = admin::DefaultImpl.new(admin.clone(), deps, env)?;
        result.messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: oracle_contract.id,
            callback_code_hash: oracle_contract.code_hash,
            send: vec![],
            label: format!("Sienna Lend Oracle: {}", time),
            msg: to_binary(&OracleInitMsg {
                admin,
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
    fn whitelist(config: MarketInitConfig) -> StdResult<HandleResponse> {
        let info = Whitelisting::load_market_contract(&deps.storage)?;
        let label = format!(
            "Sienna Lend {} market with overseer: {}",
            config.token_symbol, env.contract.address
        );

        let market = Market {
            contract: ContractLink {
                address: HumanAddr::default(),
                code_hash: info.code_hash.clone(),
            },
            symbol: config.token_symbol,
            ltv_ratio: config.ltv_ratio,
        };
        market.validate()?;

        Whitelisting::set_pending(&mut deps.storage, &market)?;

        Ok(HandleResponse {
            messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
                label,
                code_id: info.id,
                callback_code_hash: info.code_hash,
                send: vec![],
                msg: to_binary(&MarketInitMsg {
                    admin: env.message.sender,
                    prng_seed: config.prng_seed,
                    entropy: config.entropy,
                    interest_model_contract: config.interest_model_contract,
                    key: MasterKey::load(&deps.storage)?,
                    config: config.config,
                    underlying_asset: config.underlying_asset,
                    callback: Callback {
                        contract: ContractLink {
                            address: env.contract.address,
                            code_hash: env.contract_code_hash,
                        },
                        msg: to_binary(&HandleMsg::RegisterMarket {})?,
                    },
                })?,
            })],
            log: vec![log("action", "whitelist")],
            data: None,
        })
    }

    #[handle]
    fn register_market() -> StdResult<HandleResponse> {
        let mut market = Whitelisting::pop_pending(&mut deps.storage)?;
        let oracle = Contracts::load_oracle(deps)?;

        market.contract.address = env.message.sender;

        Markets::push(deps, &market)?;

        let log_address = market.contract.address.to_string();

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
            log: vec![
                log("action", "register_market"),
                log("market_address", log_address),
            ],
            data: None,
        })
    }

    #[handle]
    fn enter(markets: Vec<HumanAddr>) -> StdResult<HandleResponse> {
        let account = Account::new(&deps.api, &env.message.sender)?;

        let ids = markets.iter()
            .map(|x| Markets::get_id(deps, x))
            .collect::<StdResult<Vec<u64>>>()?;
            
        account.add_markets(&mut deps.storage, ids)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("action", "enter")],
            data: None,
        })
    }

    #[handle]
    fn exit(market_address: HumanAddr) -> StdResult<HandleResponse> {
        let account = Account::new(&deps.api, &env.message.sender)?;
        let (id, market) = account.get_market(deps, &market_address)?;

        let method = MarketAuth::Internal {
            address: env.message.sender,
            key: MasterKey::load(&deps.storage)?,
        };

        // TODO: Maybe calc_liquidity() can be changed to cover this check in order to avoid calling this twice.
        let snapshot = query_account(
            &deps.querier,
            market.contract,
            method.clone(),
            None, // None because we only check if borrows balance is zero here.
        )?;

        if snapshot.borrow_balance != Uint256::zero() {
            return Err(StdError::generic_err("Cannot exit market while borrowing."));
        }

        let liquidity = calc_liquidity(
            deps,
            &account,
            method,
            Some(market_address),
            Some(env.block.height),
            snapshot.sl_token_balance,
            Uint256::zero(),
        )?;

        if liquidity.shortfall > Uint256::zero() {
            return Err(StdError::generic_err(format!(
                "This account is currently below its target collateral requirement by {}",
                liquidity.shortfall
            )));
        }

        account.remove_market(&mut deps.storage, id)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("action", "exit")],
            data: None,
        })
    }

    #[handle]
    #[require_admin]
    fn change_market(
        market: HumanAddr,
        ltv_ratio:  Option<Decimal256>,
        symbol: Option<String>
    ) -> StdResult<HandleResponse>{
        let (_, stored_market) = Markets::get_by_addr(deps, &market)?;

        let update_oracle = symbol.is_some();
        let symbol = symbol.unwrap_or(stored_market.symbol);

        let ltv_ratio = if let Some(ltv_ratio) = ltv_ratio {
            let price = query_price(
                &deps.querier,
                Contracts::load_oracle(deps)?,
                symbol.clone().into(),
                QUOTE_SYMBOL.into(),
                None,
            )?;
    
            // Can't set collateral factor if the price is 0
            if price.rate == Decimal256::zero() {
                return Err(StdError::generic_err(
                    "Cannot set LTV ratio if the price is 0",
                ));
            }
            
            ltv_ratio
        } else {
            stored_market.ltv_ratio
        };

        Markets::update(deps, &market, |mut m| {
            m.ltv_ratio = ltv_ratio;
            m.validate()?;

            m.symbol = symbol.clone();

            Ok(m)
        })?;

        let messages = if update_oracle {
            let oracle = Contracts::load_oracle(deps)?;

            vec![CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: oracle.address,
                callback_code_hash: oracle.code_hash,
                send: vec![],
                msg: to_binary(&OracleHandleMsg::UpdateAssets {
                    assets: vec![Asset {
                        address: stored_market.contract.address,
                        symbol,
                    }],
                })?,
            })]
        } else {
            vec![]
        };

        Ok(HandleResponse {
            messages,
            log: vec![],
            data: None
        })
    }

    #[handle]
    #[require_admin]
    fn change_config(
        premium_rate: Option<Decimal256>,
        close_factor: Option<Decimal256>
    ) -> StdResult<HandleResponse> {
        let mut constants = Constants::load(&deps.storage)?;

        if let Some(premium_rate) = premium_rate {
            constants.premium = premium_rate;
        }

        if let Some(close_factor) = close_factor {
            constants.set_close_factor(close_factor)?;
        }

        Constants::save(&mut deps.storage, &constants)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![
                log("action", "change_config"),
                log("premium_rate", constants.premium),
                log("close_factor", constants.close_factor())
            ],
            data: None
        })
    }

    #[query]
    fn entered_markets(method: OverseerAuth) -> StdResult<Vec<Market<HumanAddr>>> {
        let account = Account::authenticate(
            deps,
            method,
            OverseerPermissions::AccountInfo,
            Contracts::load_self_ref,
        )?;

        account.list_markets(deps)
    }

    #[query]
    fn account_liquidity(
        method: OverseerAuth,
        market: Option<HumanAddr>,
        block: Option<u64>,
        redeem_amount: Uint256,
        borrow_amount: Uint256,
    ) -> StdResult<AccountLiquidity> {
        let account = Account::authenticate(
            deps,
            method,
            OverseerPermissions::AccountInfo,
            Contracts::load_self_ref,
        )?;

        calc_liquidity(
            deps,
            &account,
            // This is ugly
            MarketAuth::Internal {
                key: MasterKey::load(&deps.storage)?,
                address: account.0.humanize(&deps.api)?,
            },
            market,
            block,
            redeem_amount,
            borrow_amount,
        )
    }

    #[query]
    fn can_transfer_internal(
        key: MasterKey,
        address: HumanAddr,
        market: HumanAddr,
        block: u64,
        amount: Uint256,
    ) -> StdResult<bool> {
        MasterKey::check(&deps.storage, &key)?;

        let account = Account::new(&deps.api, &address)?;

        // If not entered the market then transfer is allowed.
        if account.get_market(&deps, &market).is_err() {
            return Ok(true);
        }

        let result = calc_liquidity(
            deps,
            &account,
            MarketAuth::Internal { key, address },
            Some(market),
            Some(block),
            amount,
            Uint256::zero(),
        )?;

        if result.shortfall > Uint256::zero() {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    #[query]
    fn seize_amount(
        borrowed: HumanAddr,
        collateral: HumanAddr,
        repay_amount: Uint256,
    ) -> StdResult<Uint256> {
        let premium = Constants::load(&deps.storage)?.premium;

        //  Read oracle prices for borrowed and collateral markets
        let oracle = Contracts::load_oracle(deps)?;
        let price_borrowed = query_price(
            &deps.querier,
            oracle.clone(),
            AssetType::Address(borrowed),
            QUOTE_SYMBOL.into(),
            None,
        )?;
        let price_collateral = query_price(
            &deps.querier,
            oracle,
            AssetType::Address(collateral.clone()),
            QUOTE_SYMBOL.into(),
            None,
        )?;
    
        // Get the exchange rate and calculate the number of collateral tokens to seize
        let (_, market) = Markets::get_by_addr(deps, &collateral)?;
        let exchange_rate = query_exchange_rate(&deps.querier, market.contract, None)?;
        let ratio = ((premium * price_borrowed.rate)? / (price_collateral.rate * exchange_rate)?)?;
    
        repay_amount.decimal_mul(ratio)
    }

    #[query]
    fn markets(pagination: Pagination) -> StdResult<Vec<Market<HumanAddr>>> {
        Markets::list(deps, pagination)
    }

    #[query]
    fn market(address: HumanAddr) -> StdResult<Market<HumanAddr>> {
        let (_, market) = Markets::get_by_addr(deps, &address)?;

        Ok(market)
    }

    #[query]
    fn config() -> StdResult<Config> {
        Constants::load(&deps.storage)
    }
}

/// Determine what the account liquidity would be if the given amounts were redeemed/borrowed.
fn calc_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    account: &Account,
    method: MarketAuth,
    target_asset: Option<HumanAddr>,
    block: Option<u64>,
    redeem_amount: Uint256,
    borrow_amount: Uint256,
) -> StdResult<AccountLiquidity> {
    let oracle = Contracts::load_oracle(deps)?;

    let mut total_collateral = Uint256::zero();
    let mut total_borrowed = Uint256::zero();

    let markets = account.list_markets(deps)?;
    
    if markets.len() == 0 {
        return Err(StdError::generic_err("Not entered in any markets."));
    }

    let target_asset = if let Some(asset) = target_asset {
        if !markets.iter().any(|x| x.contract.address == asset) {
            return Err(StdError::generic_err(format!("Not entered in market: {}", asset)));
        }

        asset
    } else {
        HumanAddr::default()
    };

    for market in markets {
        let is_target_asset = target_asset == market.contract.address;

        let snapshot = query_account(&deps.querier, market.contract, method.clone(), block)?;

        let price = query_price(
            &deps.querier,
            oracle.clone(),
            market.symbol.into(),
            QUOTE_SYMBOL.into(),
            None,
        )?;

        let conversion_factor = ((market.ltv_ratio * snapshot.exchange_rate)? * price.rate)?;

        total_collateral = (Uint256::from(snapshot.sl_token_balance)
            .decimal_mul(conversion_factor)?
            + total_collateral)?;
        total_borrowed =
            (Uint256::from(snapshot.borrow_balance).decimal_mul(price.rate)? + total_borrowed)?;

        if is_target_asset {
            total_borrowed = (redeem_amount.decimal_mul(conversion_factor)? + total_borrowed)?;
            total_borrowed = (borrow_amount.decimal_mul(price.rate)? + total_borrowed)?;
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
