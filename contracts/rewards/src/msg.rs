use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};
use cosmwasm_utils::ContractInfo;

use crate::data::RewardPool;

pub(crate) const UPPER_OVERFLOW_MSG: &str = "Upper bound overflow detected.";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub sienna_token: ContractInfo,
    pub reward_pools: Option<Vec<RewardPool>>,
    pub claim_interval: u64,
    pub prng_seed: Binary,
    pub entropy: Binary
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    LockTokens { 
        amount: Uint128,
        lp_token: HumanAddr
    },
    AddPools { 
        pools: Vec<RewardPool>
    },
    RemovePools {
        /// The addresses of the LP tokens of the pools to be removed.
        addresses: Vec<HumanAddr>
    },
    ChangeClaimInterval {
        interval: u64
    },
    Admin(AdminHandleMsg)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Admin(AdminQueryMsg)
}
