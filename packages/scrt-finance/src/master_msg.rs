use crate::types::{Schedule, WeightInfo};
use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MasterInitMsg {
    pub gov_token_addr: HumanAddr,
    pub gov_token_hash: String,
    pub minting_schedule: Schedule,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MasterHandleMsg {
    UpdateAllocation {
        spy_addr: HumanAddr,
        spy_hash: String,
        hook: Option<Binary>,
    },

    // Admin commands
    SetWeights {
        weights: Vec<WeightInfo>,
    },
    SetSchedule {
        schedule: Schedule,
    },
    SetGovToken {
        addr: HumanAddr,
        hash: String,
    },
    ChangeAdmin {
        addr: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MasterHandleAnswer {
    Success,
    Failure,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MasterQueryMsg {
    Admin {},
    GovToken {},
    Schedule {},
    SpyWeight { addr: HumanAddr },
    Pending { spy_addr: HumanAddr, block: u64 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum MasterQueryAnswer {
    Admin {
        address: HumanAddr,
    },
    GovToken {
        token_addr: HumanAddr,
        token_hash: String,
    },
    Schedule {
        schedule: Schedule,
    },
    SpyWeight {
        weight: u64,
    },
    Pending {
        amount: Uint128,
    },
}

// #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// #[serde(rename_all = "snake_case")]
// // Duplicating because need a generic one in the master contract, but it has to be in each SPY's HandleMsg
// pub enum CallbackMsg {
//     NotifyAllocation {
//         amount: Uint128,
//         hook: Option<Binary>,
//     },
// }
