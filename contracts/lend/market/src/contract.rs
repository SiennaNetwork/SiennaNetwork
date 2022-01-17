mod state;

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
        require_admin,
        secret_toolkit::snip20,
        snip20_impl::msg::{InitConfig, InitMsg as Snip20InitMsg},
        Callback, Canonize, ContractInstantiationInfo, ContractLink, Decimal256, Humanize, Permit,
        Uint128, Uint256, BLOCK_SIZE,
    },
    interfaces::{market::*, overseer::OverseerPermissions},
};

use state::{Config};

#[contract_impl(path = "lend_shared::interfaces::market", component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        underlying_asset: ContractLink<HumanAddr>,
        sl_token_info: ContractInstantiationInfo,
    ) -> StdResult<InitResponse> {
        let self_ref = ContractLink {
            address: env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone(),
        };

        let sl_token = ContractLink {
            address: HumanAddr::default(), // Added in RegisterSlToken
            code_hash: sl_token_info.code_hash.clone(),
        };

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
                        name: format!("Sienna Lend: {}", token_info.name),
                        symbol: format!("SL{}", token_info.symbol),
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
        unimplemented!()
    }

    #[handle]
    fn register_sl_token() -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    #[handle]
    fn register_contracts(
        overseer_contract: ContractLink<HumanAddr>,
        interest_model: ContractLink<HumanAddr>,
    ) -> StdResult<HandleResponse> {
        unimplemented!()
    }

    #[handle]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
    ) -> StdResult<HandleResponse> {
        unimplemented!()
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
    fn borrower(
        id: Binary,
    ) -> StdResult<BorrowerInfoResponse> {
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
