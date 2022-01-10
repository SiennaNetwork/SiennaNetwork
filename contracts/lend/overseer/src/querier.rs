//TODO: Move to lend-shared
use lend_shared::{
    fadroma::{
        cosmwasm_std::{StdResult, StdError, Querier, QueryRequest, WasmQuery, Binary, to_binary},
        ContractLink, HumanAddr
    },
    interfaces::market::{QueryMsg, QueryResponse, AccountSnapshotResponse}
};


pub fn query_snapshot(
    querier: &impl Querier,
    market: ContractLink<HumanAddr>,
    id: Binary
) -> StdResult<AccountSnapshotResponse> {
    let result: QueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::AccountSnapshot {
            id
        })?
    }))?;

    match result {
        QueryResponse::AccountSnapshot { account_snapshot } => {
            Ok(account_snapshot)
        },
        _ => Err(StdError::generic_err("Expected QueryResponse::AccountSnapshot"))
    }
}
