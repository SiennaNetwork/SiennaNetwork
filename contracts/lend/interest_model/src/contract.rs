//#[cfg(test)]
//mod tests;
mod state;

use lend_shared::fadroma::{
    admin,
    admin::{Admin, assert_admin},
    require_admin,
    derive_contract::*,
    cosmwasm_std,
    cosmwasm_std::{
        StdResult, InitResponse, HandleResponse, HumanAddr
    },
    Decimal256
};
use lend_shared::interfaces::interest_model::ConfigResponse;

use state::{save_config, load_config, Config};

#[contract_impl(
    entry,
    path = "lend_shared::interfaces::interest_model",
    component(path = "admin")
)]
pub trait InterestModel {
    #[init]
    fn new(
        admin:Option<HumanAddr>,
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>
    ) -> StdResult<InitResponse> {
        unimplemented!()
    }

    #[handle]
    fn update_config(
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>
    ) -> StdResult<HandleResponse> {
        unimplemented!()
    }

    #[query("config")]
    fn config() -> StdResult<ConfigResponse> {
        unimplemented!()
    }

    #[query("borrow_rate")]
    fn borrow_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256
    ) -> StdResult<Decimal256> {
        unimplemented!()
    }

    #[query("supply_rate")]
    fn supply_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256,
        reserve_factor: Decimal256
    ) -> StdResult<Decimal256> {
        unimplemented!()
    }
}
