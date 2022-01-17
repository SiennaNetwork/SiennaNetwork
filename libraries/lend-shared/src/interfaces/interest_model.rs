use fadroma::{
    admin, cosmwasm_std,
    cosmwasm_std::{
        to_binary, HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, WasmQuery,
    },
    derive_contract::*,
    schemars, ContractLink, Decimal256, QueryRequest,
};
use serde::{Deserialize, Serialize};

#[interface(component(path = "admin"))]
pub trait InterestModel {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>,
    ) -> StdResult<InitResponse>;

    #[handle]
    fn update_config(
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>,
    ) -> StdResult<HandleResponse>;

    #[query("config")]
    fn config() -> StdResult<ConfigResponse>;

    #[query("borrow_rate")]
    fn borrow_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256,
    ) -> StdResult<Decimal256>;

    #[query("supply_rate")]
    fn supply_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256,
        reserve_factor: Decimal256,
    ) -> StdResult<Decimal256>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct ConfigResponse {
    pub base_rate: Decimal256,
    pub interest_multiplier: Decimal256,
}

pub fn query_borrow_rate(
    querier: &impl Querier,
    interest_model: ContractLink<HumanAddr>,
    market_size: Decimal256,
    num_borrows: Decimal256,
    reserves: Decimal256,
) -> StdResult<Decimal256> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: interest_model.address,
        callback_code_hash: interest_model.code_hash,
        msg: to_binary(&QueryMsg::BorrowRate {
            market_size,
            num_borrows,
            reserves,
        })?,
    }))?;

    match result {
        QueryResponse::BorrowRate { borrow_rate } => Ok(borrow_rate),
        _ => Err(StdError::generic_err("Expecting Queryresponse::BorrowRate")),
    }
}
