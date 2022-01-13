mod checks;
mod ops;
mod state;
mod token;

use lend_shared::{
    fadroma::{
        admin,
        admin::{assert_admin, Admin},
        cosmwasm_std,
        cosmwasm_std::{
            log, to_binary, Api, Binary, CosmosMsg, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
        },
        derive_contract::*,
        from_binary, require_admin,
        secret_toolkit::snip20,
        snip20_impl::msg::{InitConfig, InitMsg as Snip20InitMsg},
        Callback, Canonize, ContractInstantiationInfo, ContractLink, Decimal256, Humanize, Permit,
        Uint128, Uint256, BLOCK_SIZE,
    },
    interfaces::{
        market::*,
        overseer::{query_id, OverseerPermissions},
    },
};

use state::{Config, Contracts, GlobalData};

#[contract_impl(path = "lend_shared::interfaces::market", component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        sl_token_info: ContractInstantiationInfo,
        initial_exchange_rate: Decimal256,
        reserve_factor: Decimal256,
        underlying_asset: ContractLink<HumanAddr>,
        overseer_contract: ContractLink<HumanAddr>,
        interest_model_contract: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse> {
        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        };
        let sl_token = ContractLink {
            address: HumanAddr::default(), // Added in RegisterSlToken
            code_hash: sl_token_info.code_hash.clone(),
        };

        Config::save(
            deps,
            &Config {
                initial_exchange_rate,
                reserve_factor,
            },
        )?;

        Contracts::save_overseer(deps, &overseer_contract);
        Contracts::save_interest_model(deps, &interest_model_contract);
        Contracts::save_underlying(deps, &underlying_asset);
        Contracts::save_sl_token(deps, &sl_token);

        let time = env.block.time;
        admin::DefaultImpl.new(admin, deps, env)?;

        let token_info = snip20::token_info_query(
            &deps.querier,
            BLOCK_SIZE,
            underlying_asset.code_hash.clone(),
            underlying_asset.address.clone(),
        )?;

        Ok(InitResponse {
            messages: vec![
                snip20::set_viewing_key_msg(
                    VIEWING_KEY.into(),
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash.clone(),
                    underlying_asset.address.clone(),
                )?,
                snip20::register_receive_msg(
                    self_ref.code_hash.clone(),
                    None,
                    BLOCK_SIZE,
                    underlying_asset.code_hash,
                    underlying_asset.address,
                )?,
                CosmosMsg::Wasm(WasmMsg::Instantiate {
                    code_id: sl_token_info.id,
                    callback_code_hash: sl_token_info.code_hash,
                    send: vec![],
                    label: format!("Interest token for SIENNA Lend: {}", time),
                    msg: to_binary(&Snip20InitMsg {
                        admin: None,
                        name: format!("SIENNA Lend interest token: {}", token_info.name),
                        symbol: format!("sl{}", token_info.symbol),
                        decimals: token_info.decimals,
                        initial_allowances: None,
                        initial_balances: None,
                        prng_seed,
                        config: Some(
                            InitConfig::builder()
                                .public_total_supply()
                                .enable_mint()
                                .build(),
                        ),
                        callback: Some(Callback {
                            msg: to_binary(&HandleMsg::RegisterSlToken {})?,
                            contract: self_ref,
                        }),
                    })?,
                }),
            ],
            log: vec![],
        })
    }

    #[handle]
    fn receive(from: HumanAddr, msg: Option<Binary>, amount: Uint128) -> StdResult<HandleResponse> {
        if msg.is_none() {
            return Err(StdError::generic_err("\"msg\" parameter cannot be empty."));
        }
        match from_binary(&msg.unwrap())? {
            ReceiverCallbackMsg::DepositUnderlying { permit } => {
                if env.message.sender != Contracts::load_underlying(deps)?.address {
                    return Err(StdError::unauthorized());
                }

                let id = query_id(&deps.querier, Contracts::load_overseer(deps)?, permit)?;

                ops::deposit_underlying(deps, env, id, Uint256::from(amount))
            }
            ReceiverCallbackMsg::WithdrawUnderlying { permit } => {
                if env.message.sender != Contracts::load_sl_token(deps)?.address {
                    return Err(StdError::unauthorized());
                }

                let id = query_id(&deps.querier, Contracts::load_overseer(deps)?, permit)?;

                ops::withdraw_underlying(deps, env, id, Uint256::from(amount))
            }
        }
    }

    #[handle]
    fn register_sl_token() -> StdResult<HandleResponse> {
        let mut sl_token = Contracts::load_sl_token(deps)?;

        if sl_token.address != HumanAddr::default() {
            return Err(StdError::unauthorized());
        }

        sl_token.address = env.message.sender;
        Contracts::save_sl_token(deps, &sl_token)?;

        Ok(HandleResponse {
            messages: vec![snip20::register_receive_msg(
                env.contract_code_hash,
                None,
                BLOCK_SIZE,
                sl_token.code_hash,
                sl_token.address,
            )?],
            log: vec![log("action", "register_sl_token")],
            data: None,
        })
    }

    #[handle]
    #[require_admin]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
    ) -> StdResult<HandleResponse> {
        let mut config = Config::load(deps)?;
        if let Some(interest_model) = interest_model {
            Contracts::save_interest_model(deps, &interest_model)?;
        }

        if let Some(reserve_factor) = reserve_factor {
            config.reserve_factor = reserve_factor;
            Config::save(deps, &config)?;
        }

        Ok(HandleResponse::default())
    }

    #[handle]
    fn reduce_reserves(amount: Uint128) -> StdResult<HandleResponse> {
        unimplemented!()
    }

    #[query("config")]
    fn config() -> StdResult<ConfigResponse> {
        unimplemented!()
    }

    #[query("state")]
    fn state() -> StdResult<StateResponse> {
        unimplemented!()
    }

    #[query("borrower")]
    fn borrower(id: Binary) -> StdResult<BorrowerInfoResponse> {
        unimplemented!()
    }

    #[query("borrow_rate_per_block")]
    fn borrow_rate() -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("supply_rate_per_block")]
    fn supply_rate() -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("exchange_rate")]
    fn exchange_rate() -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("borrow_balance")]
    fn borrow_balance(id: Binary) -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("account_snapshot")]
    fn account_snapshot(id: Binary) -> StdResult<AccountSnapshotResponse> {
        unimplemented!()
    }
}
