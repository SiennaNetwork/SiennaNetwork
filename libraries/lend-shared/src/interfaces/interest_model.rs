use fadroma::{
    schemars,
    admin,
    derive_contract::*,
    cosmwasm_std,
    cosmwasm_std::{
        StdResult, InitResponse, HandleResponse, HumanAddr
    },
    Decimal256
};
use serde::{Deserialize, Serialize};

#[interface(component(path = "admin"))]
pub trait InterestModel {
    #[init]
    fn new(
        admin:Option<HumanAddr>,
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>
    ) -> StdResult<InitResponse>;

    #[handle]
    fn update_config(
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>
    ) -> StdResult<HandleResponse>;

    #[query("config")]
    fn config() -> StdResult<ConfigResponse>;

    #[query("borrow_rate")]
    fn borrow_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256
    ) -> StdResult<Decimal256>;

    #[query("supply_rate")]
    fn supply_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256,
        reserve_factor: Decimal256
    ) -> StdResult<Decimal256>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct ConfigResponse {
    pub base_rate: Decimal256,
    pub interest_multiplier: Decimal256
}
