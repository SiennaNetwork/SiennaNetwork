//#[cfg(test)]
//mod tests;
mod state;

use lend_shared::core;
use lend_shared::fadroma::{
    admin,
    admin::{assert_admin, Admin},
    cosmwasm_std,
    cosmwasm_std::{HandleResponse, HumanAddr, InitResponse, StdResult},
    derive_contract::*,
    require_admin, Decimal256,
};
use lend_shared::interfaces::interest_model::ConfigResponse;

use state::{load_interest_model, save_interest_model};

#[contract_impl(
    entry,
    path = "lend_shared::interfaces::interest_model",
    component(path = "admin")
)]
pub trait InterestModel {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>,
    ) -> StdResult<InitResponse> {
        let interest_model = core::JumpRateInterest::v1(
            base_rate_year,
            multiplier_year,
            jump_multiplier_year,
            jump_threshold,
            blocks_year,
        )?;
        save_interest_model(&mut deps.storage, &interest_model)?;

        admin::DefaultImpl.new(admin, deps, env)
    }

    #[handle]
    #[require_admin]
    fn update_config(
        base_rate_year: Decimal256,
        multiplier_year: Decimal256,
        jump_multiplier_year: Decimal256,
        jump_threshold: Decimal256,
        blocks_year: Option<u64>,
    ) -> StdResult<HandleResponse> {
        let new_interest_model = core::JumpRateInterest::v1(
            base_rate_year,
            multiplier_year,
            jump_multiplier_year,
            jump_threshold,
            blocks_year,
        )?;
        save_interest_model(&mut deps.storage, &new_interest_model)?;
        Ok(HandleResponse::default())
    }

    #[query("config")]
    fn config() -> StdResult<ConfigResponse> {
        let config = state::load_interest_model(&deps.storage)?;

        Ok(ConfigResponse {
            multiplier_block: config.multiplier_block,
            jump_multiplier_block: config.jump_multiplier_block,
            base_rate_block: config.base_rate_block,
            jump_threshold: config.jump_threshold,
        })
    }

    #[query("borrow_rate")]
    fn borrow_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256,
    ) -> StdResult<Decimal256> {
        let interest_model = load_interest_model(&deps.storage)?;

        Ok(interest_model.borrow_rate(market_size, num_borrows, reserves)?)
    }

    #[query("supply_rate")]
    fn supply_rate(
        market_size: Decimal256,
        num_borrows: Decimal256,
        reserves: Decimal256,
        reserve_factor: Decimal256,
    ) -> StdResult<Decimal256> {
        let interest_model = load_interest_model(&deps.storage)?;

        Ok(interest_model.supply_rate(market_size, num_borrows, reserves, reserve_factor)?)
    }
}
