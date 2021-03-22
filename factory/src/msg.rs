use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{TokenPair, ContractInstantiationInfo, ContractInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub lp_token_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub sienna_token: ContractInfo
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Instantiates an exchange pair contract
    CreateExchange {
        pair: TokenPair
    },
    /// Used by a newly instantiated exchange contract to register
    /// itself with the factory
    RegisterExchange {
        pair: TokenPair
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetExchangePair {
        exchange_addr: HumanAddr,
    },
    GetExchangeAddress {
        pair: TokenPair
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryResponse {
    GetExchangePair {
        pair: TokenPair
    },
    GetExchangeAddress {
        address: HumanAddr
    }
}
