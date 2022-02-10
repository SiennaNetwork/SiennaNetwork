use crate::{
    account::{accumulate, Amount, CloseSeal, Volume},
    config::{RewardsConfig, IRewardsConfig},
    errors,
    time_utils::{Clock, IClock, Duration, Moment, DAY},
    Rewards,
};
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// Pool totals
pub struct Total {
    pub clock: Clock,
    /// "When was the last time someone staked or unstaked tokens?"
    /// Set to current time on lock/unlock.
    pub updated: Moment,
    /// "What liquidity is there in the whole pool right now?"
    /// Incremented/decremented on lock/unlock.
    pub staked: Amount,
    /// "What liquidity has this pool contained up to this point?"
    /// Before lock/unlock, if staked > 0, this is incremented
    /// by total.elapsed * total.staked
    pub volume: Volume,
    /// "What amount of rewards is currently available for users?"
    /// Queried from reward token.
    pub budget: Amount,
    /// "What rewards has everyone received so far?"
    /// Incremented on claim.
    pub distributed: Amount,
    /// "what rewards were unlocked for this pool so far?"
    /// computed as balance + claimed.
    pub unlocked: Amount,
    /// "how much must the user wait between claims?"
    /// Configured on init.
    /// Account bondings are reset to this value on claim.
    pub bonding: Duration,
    /// "Is this pool closed, and if so, when and why?"
    /// Set irreversibly via handle method.
    pub closed: Option<CloseSeal>,
}
pub trait ITotal<S, A, Q, C>: Sized
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    /// Load and compute the up-to-date totals for a given moment in time
    fn from_time(core: &C, time: Moment) -> StdResult<Self>;
    /// Load and compute the up-to-date totals for a given `Clock` struct
    fn get(core: &C, clock: Clock) -> StdResult<Self>;
    /// Store values that updated due to the passing of time
    fn commit_elapsed(&self, core: &mut C) -> StdResult<()>;
    /// Store values that updated due to a claim
    fn commit_claim(&mut self, core: &mut C, earned: Amount) -> StdResult<()>;
}
impl Total {
    pub const VOLUME: &'static [u8] = b"/total/volume";
    pub const UPDATED: &'static [u8] = b"/total/updated";
    pub const STAKED: &'static [u8] = b"/total/size";
    pub const CLAIMED: &'static [u8] = b"/total/claimed";
}
impl<S, A, Q, C> ITotal<S, A, Q, C> for Total
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Rewards<S, A, Q>,
{
    fn from_time(core: &C, time: Moment) -> StdResult<Self> {
        Self::get(core, Clock::get(core, time)?)
    }
    fn get(core: &C, clock: Clock) -> StdResult<Self> {
        let mut total = Self::default();
        // # 1. Timestamps
        total.clock = clock;
        total.updated = core.get(Total::UPDATED)?.unwrap_or(total.clock.now);
        if total.clock.now < total.updated {
            return errors::no_time_travel(2);
        }
        // # 2. Liquidity
        // When users lock tokens in the pool, liquidity accumulates.
        // Pool liquidity is internally represented by two variables:
        // * `staked` is the total number of LP tokens that are
        //   currently staked in the pool.
        //   * Incremented and decremented on withdraws and deposits.
        //   * Should be equal to this contract's balance in the
        //     LP token contract.
        // * `volume`. The total amount of liquidity contained by the pool
        //   over its entire lifetime. Liquidity is defined as amount of tokens
        //   multiplied by time.
        //   * Incremented by `elapsed * staked` on deposits and withdrawals.
        //   * Computed as `last_value + elapsed * staked` on queries.
        // > EXAMPLE:
        //   Starting with a new pool, lock 10 LP for 20 moments.
        //   The pool will have a liquidity of 200.
        //   Deposit 10 more; 5 moments later, the liquidity will be 300.
        let last_volume = core.get(Total::VOLUME)?.unwrap_or(Volume::zero());
        let elapsed = total.clock.now - total.updated;
        total.staked = core.get(Total::STAKED)?.unwrap_or(Amount::zero());
        total.volume = accumulate(last_volume, elapsed, total.staked)?;
        let reward_token = RewardsConfig::reward_token(core)?;
        let ref address = RewardsConfig::self_link(core)?.address;
        let ref vk = RewardsConfig::reward_vk(core)?;
        // # 3. Budget
        // * The pool queries its `balance` in reward tokens from the reward token
        //   contract. Rewards are computed on the basis of this balance.
        // * TODO: Couple budget to epoch clock in order to
        //   define a maximum amount of rewards per epoch.
        // * In the case of **single-sided staking** (e.g. staking SIENNA to earn SIENNA)
        //   the value of `staked` is subtracted from this balance in order to separate
        //   the tokens staked by users from the reward budget.
        // * The pool keeps track of how much rewards have been distributed,
        //   in the `distributed` variable which is incremented on successful claims.
        // * The `unlocked` field is equal to `budget + claimed` and is informative.
        //   It should be equal to the sum released from RPT for this total.
        total.budget = reward_token.query_balance(core.querier(), address, vk)?;
        let lp_token = RewardsConfig::lp_token(core)?;
        let is_single_sided = reward_token.link == lp_token.link;
        if is_single_sided {
            total.budget = (total.budget - total.staked)?;
        }
        total.distributed = core.get(Total::CLAIMED)?.unwrap_or(Amount::zero());
        total.unlocked = total.distributed + total.budget;
        // # 4. Throttles
        // * Bonding period: user must wait this much before each claim.
        // * Closing the pool stops its time and makes it
        //   return all funds upon any user action.
        total.bonding = core.get(RewardsConfig::BONDING)?.unwrap_or(DAY);
        total.closed = core.get(RewardsConfig::CLOSED)?;
        Ok(total)
    }
    fn commit_elapsed(&self, core: &mut C) -> StdResult<()> {
        core.set(Self::VOLUME, self.volume)?;
        core.set(Self::UPDATED, self.clock.now)?;
        Ok(())
    }
    fn commit_claim(&mut self, core: &mut C, earned: Amount) -> StdResult<()> {
        self.distributed += earned;
        core.set(Self::CLAIMED, self.distributed)?;
        Ok(())
    }
}
