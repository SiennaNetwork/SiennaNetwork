use cosmwasm_std::{
    debug_print, to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    QueryResult, StdError, StdResult, Storage,
};

use crate::{
    msg::{HandleMsg, InitMsg, QueryMsg},
    state,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    Ok(HandleResponse::default())
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetAddress { key } => query_get_address(&deps, &key),
        QueryMsg::GetUint { key } => query_get_unit(&deps, &key),
    }
}

pub fn query_get_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: &[u8],
) -> QueryResult {
    match state::get_address(&deps.storage, key) {
        Some(data) => Ok(data),
        None => Err(StdError::generic_err(
            "Сouldn't find address in address storage",
        )),
    }
}

pub fn query_get_unit<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    key: &[u8],
) -> QueryResult {
    match state::get_uint(&deps.storage, key) {
        Some(data) => Ok(data),
        None => Err(StdError::generic_err("Сouldn't find in address storage")),
    }
}
