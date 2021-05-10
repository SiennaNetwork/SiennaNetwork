use cosmwasm_std::{HumanAddr, Binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};
use cosmwasm_utils::ContractInfo;

use crate::data::RewardPool;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub sienna_token: ContractInfo,
    pub reward_pools: Vec<RewardPool>,
    pub prng_seed: Binary,
    pub entropy: Binary
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Admin(AdminHandleMsg)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Admin(AdminQueryMsg)
}
