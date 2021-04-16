use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, HumanAddr, Binary};
use cosmwasm_utils::Callback;

use crate::handle::DisabledMsg;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub name: String,
    pub admin: Option<HumanAddr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<InitConfig>,
    pub callback: Option<Callback>,
    pub disabled_messages: Option<Vec<DisabledMsg>>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct InitialBalance {
    pub address: HumanAddr,
    pub amount: Uint128,
}

/// This type represents optional configuration values which can be overridden.
/// All values are optional and have defaults which are more private by default,
/// but can be overridden if necessary
#[derive(Serialize, Deserialize, JsonSchema, Clone, Default, Debug)]
pub struct InitConfig {
    /// Indicates whether the total supply is public or should be kept secret.
    /// default: False
    pub public_total_supply: Option<bool>,
}

impl InitMsg {
    pub fn config(&self) -> InitConfig {
        self.config.clone().unwrap_or_default()
    }
}

impl InitConfig {
    pub fn public_total_supply(&self) -> bool {
        self.public_total_supply.unwrap_or(false)
    }
}