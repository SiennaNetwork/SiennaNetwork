use amm_shared::fadroma::scrt::{
    Extern, Storage, Api, Querier, StdResult,
    InitResponse, Env, HandleResponse, Binary
};

use amm_shared::snip20_impl as composable_snip20;
use composable_snip20::{snip20_init, snip20_handle, snip20_query, DefaultSnip20Impl};
use composable_snip20::msg::{InitMsg, HandleMsg, QueryMsg};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    snip20_init(deps, env, msg, DefaultSnip20Impl)
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    snip20_handle(deps, env, msg, DefaultSnip20Impl)
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg
) -> StdResult<Binary> {
    snip20_query(deps, msg, DefaultSnip20Impl)
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
