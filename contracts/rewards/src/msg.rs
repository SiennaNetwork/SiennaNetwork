use cosmwasm_std::{Binary, HumanAddr, Uint128, StdError};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use fadroma_scrt_callback::ContractInstance;
use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};

use crate::data::{RewardPool, Account};

pub(crate) const OVERFLOW_MSG: &str = "Overflow detected.";
pub(crate) const UNDERFLOW_MSG: &str = "Underflow detected.";

/// Represents a pair that is eligible for rewards.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, JsonSchema)]
pub struct RewardPoolConfig {
    /// The LP token address that the pool will be associated with.
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
        /// The total share of all the pools provided. This is used
        /// as an additional correctness check.
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
    Accounts { 
        address: HumanAddr,
        viewing_key: String
    },
    Pools,
    TotalRewardsSupply,
    Admin(AdminQueryMsg),
    /// Copy of SNIP20 message for Keplr support
    TokenInfo { },
    /// This is only here because of Keplr
    Balance { address: HumanAddr, key: String, },
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgResponse {
    ClaimSimulation(ClaimSimulationResult),
    Accounts(Vec<Account<HumanAddr>>),
    Pools(Vec<RewardPool<HumanAddr>>),
    TotalRewardsSupply {
        amount: Uint128
    },
    /// Copy of SNIP20 message for Keplr support
    TokenInfo {
        name: String,
        symbol: String,
        decimals: u8,
        total_supply: Option<Uint128>
    },
    /// Copy of SNIP20 message for Keplr support
    Balance {
        amount: Uint128,
    },
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct ClaimSimulationResult {
    /// Detailed info about the claim for each reward pool.
    pub results: Vec<ClaimResult>,
    /// The total amount of rewards that should be claimed from all
    /// the supplied pools.
    pub total_rewards_amount: Uint128,
    /// The actual amount of rewards that would be claimed from all
    /// the supplied pools.
    pub actual_claimed: Uint128
}

#[derive(Serialize, Deserialize, JsonSchema, PartialEq, Debug)]
pub struct ClaimResult {
    /// The address of the LP token that the reward pool
    /// corresponds to.
    pub lp_token_addr: HumanAddr,
    /// The total reward amount that would be claimed from this pool.
    pub reward_amount: Uint128,
    /// The reward amount that would be claimed for a single portion.
    pub reward_per_portion: Uint128,
    pub success: bool,
    pub error: Option<ClaimError>
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(tag = "type")]
pub enum ClaimError {
    /// Occurs when the rewards pool is currently empty.
    PoolEmpty,
    /// Occurs when the user has no tokens locked in this pool.
    /// In practice, this can occur when a wrong address was provided to the query.
    AccountZeroLocked,
    /// It is possible for the user's share to be so little, that
    /// the actual reward amount of rewards calculated to be zero.
    /// However, it is highly unlikely in practice.
    AccountZeroReward,
    /// Occurs when the user tries to claim earlier than the designated claim interval.
    EarlyClaim {
        /// In Unix seconds.
        time_to_wait: u64
    }
}

pub(crate) enum GetBalanceError {
    PoolEmpty,
    StdError(StdError)
}

impl From<StdError> for GetBalanceError {
    fn from(err: StdError) -> Self {
        GetBalanceError::StdError(err)
    }
}

impl From<GetBalanceError> for StdError {
    fn from(err: GetBalanceError) -> Self {
        match err {
            GetBalanceError::PoolEmpty => {
                StdError::generic_err(
                    "The reward token pool is currently empty."
                )
            },
            GetBalanceError::StdError(std_err) => std_err
        }
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

impl Into<RewardPool<HumanAddr>> for RewardPoolConfig {
    fn into(self) -> RewardPool<HumanAddr> {
        RewardPool {
            lp_token: self.lp_token,
            share: self.share,
            size: Uint128::zero()
        }
    }
}
