use crate::{
    account::CloseSeal,
    errors,
    time_utils::{Duration, DAY},
    Rewards,
};
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Reward pool configuration
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RewardsConfig {
    pub lp_token: Option<ContractLink<HumanAddr>>,
    pub reward_token: Option<ContractLink<HumanAddr>>,
    pub reward_vk: Option<String>,
    pub bonding: Option<Duration>,
    pub timekeeper: Option<HumanAddr>,
}
impl RewardsConfig {
    pub const SELF: &'static [u8] = b"/config/self";
    pub const LP_TOKEN: &'static [u8] = b"/config/lp_token";
    pub const REWARD_TOKEN: &'static [u8] = b"/config/reward_token";
    pub const REWARD_VK: &'static [u8] = b"/config/reward_vk";
    pub const CLOSED: &'static [u8] = b"/config/closed";
    pub const BONDING: &'static [u8] = b"/config/bonding";

    pub const TIMEKEEPER: &'static [u8] = b"/config/keeper";
}
pub trait IRewardsConfig<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    /// Commit initial contract configuration to storage.
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>>;
    /// Commit contract configuration to storage.
    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    /// Commit contract configuration to storage.
    fn from_storage(core: &C) -> StdResult<RewardsConfig>;
    /// Get this contract's address (used in queries where Env is unavailable).
    fn self_link(core: &C) -> StdResult<ContractLink<HumanAddr>>;
    /// Get an interface to the LP token.
    fn lp_token(core: &C) -> StdResult<ISnip20>;
    /// Get an interface to the reward token.
    fn reward_token(core: &C) -> StdResult<ISnip20>;
    /// Get the reward viewing key.
    fn reward_vk(core: &C) -> StdResult<String>;
    /// Get the configured bonding period.
    fn bonding(core: &C) -> StdResult<Duration>;
    /// Get the address authorized to increment the epoch
    fn timekeeper(core: &C) -> StdResult<HumanAddr>;
    /// Get the address authorized to increment the epoch
    fn assert_closed(core: &C, env: &Env) -> StdResult<Duration>;
}
impl<S, A, Q, C> IRewardsConfig<S, A, Q, C> for RewardsConfig
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    fn from_storage(core: &C) -> StdResult<Self> {
        Ok(Self {
            lp_token: Some(Self::lp_token(core)?.link),
            reward_token: Some(Self::reward_token(core)?.link),
            reward_vk: None,
            bonding: Some(Self::bonding(core)?),
            timekeeper: Some(Self::timekeeper(core)?),
        })
    }
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        if self.reward_token.is_none() {
            Err(StdError::generic_err(
                "need to provide link to reward token",
            ))
        } else {
            core.set(
                RewardsConfig::SELF,
                &core.canonize(ContractLink {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash.clone(),
                })?,
            )?;
            if self.reward_vk.is_none() {
                self.reward_vk = Some("".into())
            }
            if self.bonding.is_none() {
                self.bonding = Some(DAY)
            }
            if self.timekeeper.is_none() {
                self.timekeeper = Some(env.message.sender.clone())
            }
            self.store(core)
        }
    }
    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>> {
        let RewardsConfig {
            timekeeper,
            lp_token,
            bonding,
            reward_token,
            reward_vk,
        } = self;
        let mut messages = vec![];
        if let Some(lp_token) = lp_token {
            core.set(Self::LP_TOKEN, &core.canonize(lp_token.clone())?)?;
        }
        if let Some(bonding) = bonding {
            core.set(Self::BONDING, &bonding)?;
        }
        if let Some(reward_token) = reward_token {
            core.set(Self::REWARD_TOKEN, &core.canonize(reward_token.clone())?)?;
            if let Some(reward_vk) = reward_vk {
                core.set(Self::REWARD_VK, &reward_vk)?;
                messages.push(ISnip20::attach(reward_token.clone()).set_viewing_key(&reward_vk)?);
            }
        } else if let Some(reward_vk) = reward_vk {
            core.set(Self::REWARD_VK, &reward_vk)?;
            let reward_token = RewardsConfig::reward_token(core)?;
            messages.push(reward_token.set_viewing_key(&reward_vk)?);
        }
        if let Some(timekeeper) = timekeeper {
            core.set(Self::TIMEKEEPER, &core.canonize(timekeeper.clone())?)?;
        }
        Ok(messages)
    }
    fn self_link(core: &C) -> StdResult<ContractLink<HumanAddr>> {
        let link = core
            .get::<ContractLink<CanonicalAddr>>(Self::SELF)?
            .ok_or(StdError::generic_err("no self link"))?;
        Ok(core.humanize(link)?)
    }
    fn lp_token(core: &C) -> StdResult<ISnip20> {
        let link = core
            .get::<ContractLink<CanonicalAddr>>(Self::LP_TOKEN)?
            .ok_or(StdError::generic_err("no lp token"))?;
        Ok(ISnip20::attach(core.humanize(link)?))
    }
    fn reward_token(core: &C) -> StdResult<ISnip20> {
        let link = core
            .get::<ContractLink<CanonicalAddr>>(Self::REWARD_TOKEN)?
            .ok_or(StdError::generic_err("no reward token"))?;
        Ok(ISnip20::attach(core.humanize(link)?))
    }
    fn reward_vk(core: &C) -> StdResult<String> {
        Ok(core
            .get::<ViewingKey>(Self::REWARD_VK)?
            .ok_or(StdError::generic_err("no reward viewing key"))?
            .0)
    }
    fn bonding(core: &C) -> StdResult<Duration> {
        Ok(core
            .get::<Duration>(Self::BONDING)?
            .ok_or(StdError::generic_err("no bonding configured"))?)
    }
    fn timekeeper(core: &C) -> StdResult<HumanAddr> {
        Ok(core.humanize(
            core.get::<CanonicalAddr>(Self::TIMEKEEPER)?
                .ok_or(StdError::generic_err("no timekeeper address"))?,
        )?)
    }
    fn assert_closed(core: &C, env: &Env) -> StdResult<Duration> {
        if let Some((closed, _)) = core.get::<CloseSeal>(RewardsConfig::CLOSED)? {
            if closed <= env.block.time {
                Ok(env.block.time - closed)
            } else {
                errors::no_time_travel(1)
            }
        } else {
            errors::pool_not_closed()
        }
    }
}
