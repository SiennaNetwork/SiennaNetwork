use std::ops::RangeInclusive;
use amm_shared::fadroma::scrt::{
    Api, Binary, Env, Extern, HandleResponse,
    HumanAddr, InitResponse, Querier,
    StdError, StdResult, Storage, Uint128,
};

use amm_shared::snip20_impl as composable_snip20;

use composable_snip20::msg::{HandleMsg, InitMsg, QueryMsg};
use composable_snip20::{
    snip20_handle, snip20_init, snip20_query,
    Snip20, SymbolValidation, batch
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    snip20_init(deps, env, msg, LpTokenImpl)
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    snip20_handle(deps, env, msg, LpTokenImpl)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    snip20_query(deps, msg, LpTokenImpl)
}

struct LpTokenImpl;

impl Snip20 for LpTokenImpl {
    fn symbol_validation(&self) -> SymbolValidation {
        SymbolValidation {
            length: 3..=18,
            allow_upper: true,
            allow_lower: true,
            allow_numeric: false,
            allowed_special: Some(vec![b'-']),
        }
    }

    fn name_range(&self) -> RangeInclusive<usize> {
        3..=200
    }

    fn burn_from<S: Storage, A: Api, Q: Querier>(
        &self,
        _deps: &mut Extern<S, A, Q>,
        _env: Env,
        _owner: HumanAddr,
        _amount: Uint128,
        _memo: Option<String>,
    ) -> StdResult<HandleResponse> {
        Err(StdError::generic_err("This method has been disabled."))
    }

    fn batch_burn_from<S: Storage, A: Api, Q: Querier>(
        &self,
        _deps: &mut Extern<S, A, Q>,
        _env: Env,
        _actions: Vec<batch::BurnFromAction>
    ) -> StdResult<HandleResponse> {
        Err(StdError::generic_err("This method has been disabled."))
    }
}

#[cfg(target_arch = "wasm32")]
mod wasm {
    use amm_shared::fadroma::scrt::cosmwasm_std::{
        do_handle, do_init, do_query, ExternalApi, ExternalQuerier, ExternalStorage,
    };

    #[no_mangle]
    extern "C" fn init(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_init(
            &super::init::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn handle(env_ptr: u32, msg_ptr: u32) -> u32 {
        do_handle(
            &super::handle::<ExternalStorage, ExternalApi, ExternalQuerier>,
            env_ptr,
            msg_ptr,
        )
    }

    #[no_mangle]
    extern "C" fn query(msg_ptr: u32) -> u32 {
        do_query(
            &super::query::<ExternalStorage, ExternalApi, ExternalQuerier>,
            msg_ptr,
        )
    }

    // Other C externs like cosmwasm_vm_version_1, allocate, deallocate are available
    // automatically because we `use cosmwasm_std`.
}
