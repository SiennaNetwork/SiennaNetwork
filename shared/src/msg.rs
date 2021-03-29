use std::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Binary, Uint128};

use crate::{TokenPair, TokenType};
use crate::{ContractInfo, ContractInstantiationInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ExchangeInitMsg {
    /// The tokens that will be managed by the exchange
    pub pair: TokenPair,
    /// LP token instantiation info
    pub lp_token_contract: ContractInstantiationInfo,
    /// Used by the exchange contract to
    /// send back its address to the factory on init
    pub factory_info: ContractInfo,
    pub callback: Callback,
    pub sienna_token: ContractInfo
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Used to ask a contract to send back a message.
pub struct Callback {
    /// The message to call.
    pub msg: Binary,
    /// The address of the contract requesting the callback.
    pub contract_addr: HumanAddr,
    /// The code hash of the contract requesting the callback.
    pub contract_code_hash: String,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct IdoInitMsg {
    pub snip20_contract: ContractInstantiationInfo,
    pub info: IdoInitConfig,
    /// Used by the IDO to register itself with the factory.
    pub callback: Callback
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct IdoInitConfig {
    /// The token that will be used to buy the instantiated SNIP20
    pub input_token: TokenType,
    pub rate: Uint128,
    pub snip20_init_info: Snip20TokenInitInfo
}

#[derive(Serialize, Deserialize, JsonSchema)]
/// Used to provide only the essential info
/// to an IDO that instantiates a snip20 token
pub struct Snip20TokenInitInfo {
    pub name: String,
    pub prng_seed: Binary,
    pub symbol: String,
    pub decimals: u8,
    pub config: Option<Snip20InitConfig>
}

// SNIP20
#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Snip20InitMsg {
    pub name: String,
    pub admin: Option<HumanAddr>,
    pub symbol: String,
    pub decimals: u8,
    pub initial_balances: Option<Vec<Snip20InitialBalance>>,
    pub prng_seed: Binary,
    pub config: Option<Snip20InitConfig>,
    pub callback: Option<Callback>
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct Snip20InitialBalance {
    pub address: HumanAddr,
    pub amount: Uint128,
}

/// This type represents optional configuration values which can be overridden.
/// All values are optional and have defaults which are more private by default,
/// but can be overridden if necessary
#[derive(Serialize, Deserialize, JsonSchema, Clone, Default, Debug)]
pub struct Snip20InitConfig {
    /// Indicates whether the total supply is public or should be kept secret.
    /// default: False
    pub public_total_supply: Option<bool>,
}

impl Snip20InitMsg {
    pub fn config(&self) -> Snip20InitConfig {
        self.config.clone().unwrap_or_default()
    }
}

impl Snip20InitConfig {
    pub fn public_total_supply(&self) -> bool {
        self.public_total_supply.unwrap_or(false)
    }
}

impl fmt::Display for IdoInitConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Input token: {}, Rate: {}, Created token: {}({})",
            self.input_token, self.rate, 
            self.snip20_init_info.name, self.snip20_init_info.symbol
        )
    }
}
