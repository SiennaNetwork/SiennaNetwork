use cosmwasm_std::{Binary, HumanAddr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use fadroma_scrt_callback::ContractInstance;
use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};

use crate::data::{RewardPool, Account};

pub(crate) const OVERFLOW_MSG: &str = "Upper bound overflow detected.";

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
/// Represents a pair that is eligible for rewards.
pub struct RewardPoolConfig {
    pub lp_token: ContractInstance<HumanAddr>,
    /// The reward amount allocated to this pool.
    pub share: Uint128,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub reward_token: ContractInstance<HumanAddr>,
    pub reward_pools: Option<Vec<RewardPoolConfig>>,
    pub claim_interval: u64,
    pub prng_seed: Binary,
    pub entropy: Binary
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    LockTokens { 
        amount: Uint128,
        lp_token: HumanAddr
    },
    RetrieveTokens {
        amount: Uint128,
        lp_token: HumanAddr
    },
    Claim {
        /// The addresses of the LP tokens pools to claim from.
        lp_tokens: Vec<HumanAddr>
    },
    ChangePools {
        total_share: Uint128, 
        pools: Vec<RewardPoolConfig>
    },
    CreateViewingKey {
        entropy: String,
        padding: Option<String>,
    },
    SetViewingKey {
        key: String,
        padding: Option<String>,
    },
    Admin(AdminHandleMsg)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    ClaimSimulation {
        /// The addresses of the LP tokens pools to claim from.
        lp_tokens: Vec<HumanAddr>,
        viewing_key: String,
        address: HumanAddr,
        /// Unix time in seconds.
        current_time: u64
    },
    Pools,
    Accounts { 
        address: HumanAddr,
        viewing_key: String,
        /// The addresses of the LP tokens pools to get the accounts for.
        lp_tokens: Vec<HumanAddr>
    },
    /// This is only here because of Keplr
    TokenInfo { },
    Admin(AdminQueryMsg)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgResponse {
    ClaimSimulation(ClaimSimulationResult),
    Accounts(Vec<Account<HumanAddr>>),
    Pools(Vec<RewardPool<HumanAddr>>),
    TokenInfo {
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: Option<Uint128>
    }
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct ClaimSimulationResult {
    pub results: Vec<ClaimResult>,
    pub total_rewards_amount: Uint128,
    pub actual_claimed: Uint128
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct ClaimResult {
    pub lp_token_addr: HumanAddr,
    pub reward_amount: Uint128,
    pub reward_per_portion: Uint128,
    pub success: bool,
    pub error: Option<ClaimError>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum ClaimError {
    PoolEmpty,
    AccountZeroLocked,
    AccountZeroReward,
    EarlyClaim {
        time_to_wait: u64
    }
}

impl ClaimResult {
    pub fn success(lp_token_addr: HumanAddr, reward_amount: Uint128, reward_per_portion: Uint128) -> Self {
        Self {
            lp_token_addr,
            reward_amount,
            reward_per_portion,
            success: true,
            error: None
        }
    }

    pub fn error(lp_token_addr: HumanAddr, error: ClaimError) -> Self {
        Self {
            lp_token_addr,
            reward_amount: Uint128::zero(),
            reward_per_portion: Uint128::zero(),
            success: false,
            error: Some(error)
        }
    }
}

impl Into<RewardPool<HumanAddr
>> for RewardPoolConfig {
    fn into(self) -> RewardPool<HumanAddr> {
        RewardPool {
            lp_token: self.lp_token,
            share: self.share,
            size: Uint128::zero()
        }
    }
}
