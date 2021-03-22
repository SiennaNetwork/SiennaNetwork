use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Binary};

use crate::TokenPair;
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
    pub callback: Callback
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct LpTokenInitMsg {
    pub name: String,
    pub admin: HumanAddr,
    pub symbol: String,
    pub decimals: u8,
    pub callback: Callback
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Used to ask a contract to send back a message.
pub struct Callback {
    /// The message to call.
    pub msg: Binary,
    /// The address of the contract requesting the callback.
    pub contract_addr: HumanAddr,
    /// The code hash the contract requesting the callback.
    pub contract_code_hash: String,
}
