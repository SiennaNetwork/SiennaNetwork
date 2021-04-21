use cosmwasm_std::{HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// RewardPool is a struct that keeps track of rewards and lockups
#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone)]
pub struct RewardPool {
    pub residue: u128,
    pub inc_token_supply: u128,
    pub acc_reward_per_share: u128,
}

#[derive(Serialize, Deserialize, Eq, PartialEq, Debug, Clone, JsonSchema)]
pub struct TokenInfo {
    pub name: String,
    pub symbol: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UserInfo {
    pub locked: u128,
    pub debt: u128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WeightInfo {
    pub address: HumanAddr,
    pub hash: String,
    pub weight: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SpySettings {
    pub weight: u64,
    pub last_update_block: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema, Copy)]
pub struct ScheduleUnit {
    pub end_block: u64,
    pub mint_per_block: Uint128,
}

pub type Schedule = Vec<ScheduleUnit>;

pub fn sort_schedule(s: &mut Schedule) {
    s.sort_by(|&s1, &s2| s1.end_block.cmp(&s2.end_block))
}
