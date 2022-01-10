mod state;

use lend_shared::{
    fadroma::{
        BLOCK_SIZE,
        Uint256, Decimal256,
        ContractLink,
        Canonize, Humanize,
        Permit,
        Callback,
        admin,
        admin::{Admin, assert_admin},
        require_admin,
        derive_contract::*,
        cosmwasm_std,
        cosmwasm_std::{
            InitResponse, HandleResponse, HumanAddr,
            CosmosMsg, StdResult, WasmMsg, StdError,
            Extern, Storage, Api, Querier, Binary, 
            to_binary, log
        },
        secret_toolkit::snip20
    },
    interfaces::{
        overseer::{OverseerPermissions, AccountLiquidity, Config, Market},
        oracle::{HandleMsg as OracleHandleMsg, PriceAsset}
    }
};

use state::{BorrowerId, Markets, Contracts};

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
        unimplemented!()
    }

    #[handle]
    fn exit(market: HumanAddr) -> StdResult<HandleResponse> {
        unimplemented!()
    }

    #[query("entered_markets")]
    fn entered_markets(
        permit: Permit<OverseerPermissions>
    ) -> StdResult<Vec<ContractLink<HumanAddr>>> {
        unimplemented!()
    }

    #[query("borrow_factor")]
    fn borrow_factor(market: HumanAddr) -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("liquidity")]
    fn account_liquidity(
        permit: Permit<OverseerPermissions>,
    ) -> StdResult<AccountLiquidity> {
        unimplemented!()
    }

    #[query("config")]
    fn config() -> StdResult<Config> {
        unimplemented!()
    }
}
