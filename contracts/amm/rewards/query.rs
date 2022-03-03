use crate::{
    account::{Account, IAccount},
    auth::Auth,
    config::{RewardsConfig, IRewardsConfig},
    time_utils::Moment,
    total::{Total, ITotal},
    Rewards,
};
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RewardsQuery {
    /// Get the current settings of the contract.
    Config,
    /// For a moment in time, report the status of an account, with embedded pool and clock status
    UserInfo {
        at: Moment,
        address: HumanAddr,
        key: String,
    },
    WithPermit{
        query: QueryWithPermit,
        permit: Permit<RewardsPermissions>
    },

    /// For a moment in time, report pool status, with embedded clock status
    PoolInfo { at: Moment },
}
impl<S, A, Q, C> QueryDispatch<S, A, Q, C, RewardsResponse> for RewardsQuery
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    fn dispatch_query(self, core: &C) -> StdResult<RewardsResponse> {
        match self {
            RewardsQuery::Config => RewardsResponse::config(core),
            RewardsQuery::UserInfo { at, address, key } => {
                RewardsResponse::user_info_with_vk(core, at, &address, key)
            }
            RewardsQuery::PoolInfo { at } => RewardsResponse::pool_info(core, at),
            RewardsQuery::WithPermit{query, permit} =>  {
                match query {
                    QueryWithPermit::UserInfo{ at} => RewardsResponse::user_info(core, at, &permit.address)
                }
                
            }
        }
    }
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RewardsResponse {
    UserInfo(Account),
    PoolInfo(Total),
    Config(RewardsConfig),
}
pub trait IRewardsResponse<S, A, Q, C>: Sized
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    /// Populate a response with account + pool + epoch info
    fn user_info_with_vk(core: &C, time: Moment, address: &HumanAddr, key: String) -> StdResult<Self>;
    fn user_info(core: &C, time: Moment, address: &HumanAddr) -> StdResult<Self>;
    /// Populate a response with pool + epoch info
    fn pool_info(core: &C, time: Moment) -> StdResult<Self>;
    /// Populate a response with the contract's configuration
    fn config(core: &C) -> StdResult<Self>;
}
impl<S, A, Q, C> IRewardsResponse<S, A, Q, C> for RewardsResponse
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    fn user_info_with_vk(core: &C, time: Moment, address: &HumanAddr, key: String) -> StdResult<Self> {
        Auth::check_vk(core, address, &key.into())?;
        Ok(RewardsResponse::UserInfo(Account::from_addr(
            core, address, time,
        )?))
    }
    fn user_info(core: &C, time: Moment, address: &HumanAddr) -> StdResult<Self> {
        Ok(RewardsResponse::UserInfo(Account::from_addr(core, address, time)?))
    }

    
    fn pool_info(core: &C, time: Moment) -> StdResult<RewardsResponse> {
        Ok(RewardsResponse::PoolInfo(Total::from_time(core, time)?))
    }
    fn config(core: &C) -> StdResult<RewardsResponse> {
        Ok(RewardsResponse::Config(RewardsConfig::from_storage(core)?))
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    UserInfo{ at: Moment},
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RewardsPermissions {
    UserInfo
}