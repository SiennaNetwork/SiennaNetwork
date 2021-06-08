use crate::constants::CONFIG_KEY;
use crate::state::Config;
use cosmwasm_std::{
    to_binary, Api, Extern, Querier, QueryRequest, StdError, StdResult, Storage,
    WasmQuery,
};
use scrt_finance::master_msg::{MasterQueryAnswer, MasterQueryMsg};
use secret_toolkit::storage::TypedStore;

pub fn query_pending<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    block: u64,
) -> StdResult<u128> {
    let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY)?;

    let response = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        callback_code_hash: config.master.code_hash,
        contract_addr: config.master.address,
        msg: to_binary(&MasterQueryMsg::Pending {
            spy_addr: config.own_addr,
            block,
        })?,
    }))?;

    match response {
        MasterQueryAnswer::Pending { amount } => Ok(amount.u128()),
        _ => Err(StdError::generic_err(
            "something is wrong with the master contract..",
        )),
    }
}
