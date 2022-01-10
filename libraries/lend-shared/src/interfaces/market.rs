use fadroma::{
    admin, cosmwasm_std, derive_contract::*, schemars, schemars::JsonSchema, Binary, ContractLink,
    Decimal256, HandleResponse, HumanAddr, InitResponse, StdResult, Uint128,
};

use serde::{Deserialize, Serialize};

pub const VIEWING_KEY: &str = "SiennaLend"; // TODO: Should this be public?

#[interface(component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        // Underlying asset
        underlying_asset: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse>;

    /// Snip20 receiver interface
    #[handle]
    fn receive(from: HumanAddr, msg: Option<Binary>, amount: Uint128) -> StdResult<HandleResponse>;

    #[handle]
    fn register_contracts(
        overseer_contract: ContractLink<HumanAddr>,
        // The contract has the logic for
        // Sienna borrow interest rate
        interest_model: ContractLink<HumanAddr>,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn reduce_reserves(amount: Uint128) -> StdResult<HandleResponse>;

    #[query("config")]
    fn config() -> StdResult<ConfigResponse>;

    #[query("state")]
    fn state(block_height: u64) -> StdResult<StateResponse>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StateResponse {}
