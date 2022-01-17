use lend_shared::fadroma::{
    storage::{load, save},
    schemars,
    cosmwasm_std::{Storage, StdResult},
    Decimal256
};
use serde::{Deserialize, Serialize};

static KEY_CONFIG: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct Config {
    pub base_rate: Decimal256,
    pub interest_multiplier: Decimal256,
}

pub fn save_config(storage: &mut impl Storage, config: &Config) -> StdResult<()> {
    save(storage, KEY_CONFIG, config)
}

pub fn load_config(storage: &impl Storage) -> StdResult<Config> {
    Ok(load(storage, KEY_CONFIG)?.unwrap())
}
