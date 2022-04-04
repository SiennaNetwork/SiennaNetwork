use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    account::{accumulate, Amount, Volume},
    config::{RewardsConfig, IRewardsConfig},
    errors,
    total::Total,
};

/// A moment in time, as represented by the current value of env.block.time
pub type Moment = u64;
/// A duration of time, represented as a number of moments
pub type Duration = u64;
/// Seconds in 24 hours
pub const DAY: Duration = 86400;

/// Reward epoch state. Epoch is incremented after each RPT vesting.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Clock {
    /// "For what point in time do the reported values hold true?"
    /// Got from env.block time on transactions, passed by client in queries.
    pub now: Moment,
    /// "What is the current reward epoch?"
    /// Incremented by external periodic call.
    pub number: Moment,
    /// "When did the epoch last increment?"
    /// Set to current time on epoch increment.
    pub started: Moment,
    /// "What was the total pool liquidity at the epoch start?"
    /// Set to `total.volume` on epoch increment.
    pub volume: Volume,
}
impl Clock {
    pub const NUMBER: &'static [u8] = b"/epoch/number";
    pub const START: &'static [u8] = b"/epoch/start";
    pub const VOLUME: &'static [u8] = b"/epoch/volume";
    pub const UNLOCKED: &'static [u8] = b"/epoch/unlocked";
}
pub trait IClock<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    /// Get the current state of the epoch clock.
    fn get(core: &C, now: Moment) -> StdResult<Clock>;
    /// Increment the epoch and commit liquidity so far
    fn increment(core: &mut C, env: &Env, next_epoch: u64) -> StdResult<HandleResponse>;
}
impl<S, A, Q, C> IClock<S, A, Q, C> for Clock
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn get(core: &C, now: Moment) -> StdResult<Clock> {
        let mut clock = Self::default();
        clock.now = now;
        clock.number = core.get(Self::NUMBER)?.unwrap_or(0u64);
        clock.started = core.get(Self::START)?.unwrap_or(0u64);
        clock.volume = core.get(Self::VOLUME)?.unwrap_or(Volume::zero());
        Ok(clock)
    }
    fn increment(core: &mut C, env: &Env, next_epoch: u64) -> StdResult<HandleResponse> {
        if env.message.sender != RewardsConfig::timekeeper(core)? {
            return Err(StdError::unauthorized());
        }
        let epoch: Moment = core.get(Self::NUMBER)?.unwrap_or(0u64);
        if next_epoch != epoch + 1 {
            return errors::invalid_epoch_number(epoch, next_epoch);
        }
        let now = env.block.time;
        let volume = accumulate(
            core.get(Total::VOLUME)?.unwrap_or(Volume::zero()),
            now - core.get(Total::UPDATED)?.unwrap_or(now),
            core.get(Total::STAKED)?.unwrap_or(Amount::zero()),
        )?;
        core.set(Self::NUMBER, next_epoch)?;
        core.set(Self::START, now)?;
        core.set(Self::VOLUME, volume)?;
        Ok(HandleResponse::default())
    }
}
