use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use shared::{TokenPair, ContractInstantiationInfo, ContractInfo, IdoInitConfig};

use crate::state::Pagination;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub snip20_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub ido_contract: ContractInstantiationInfo,
    pub sienna_token: ContractInfo
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    /// Instantiates an exchange pair contract
    CreateExchange {
        pair: TokenPair
    },
    /// Instantiates an IDO contract
    CreateIdo {
        info: IdoInitConfig
    },
    /// Used by a newly instantiated exchange contract to register
    /// itself with the factory
    RegisterExchange {
        pair: TokenPair
    },
    /// Used by a newly instantiated IDO contract to register
    /// itself with the factory
    RegisterIdo
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetExchangePair {
        exchange_addr: HumanAddr,
    },
    GetExchangeAddress {
        pair: TokenPair
    },
    ListIdos {
        pagination: Pagination
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
    },
    ListIdos {
        idos: Vec<HumanAddr>
    }
}
