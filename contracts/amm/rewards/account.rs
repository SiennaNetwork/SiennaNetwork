use crate::{
    config::{IRewardsConfig, RewardsConfig},
    errors,
    gov::{
        config::{GovernanceConfig, IGovernanceConfig},
        user::{IUser, User},
    },
    time_utils::{Duration, Moment},
    total::{ITotal, Total},
};
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
/// Account status
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Account {
    /// "What is the overall state of the pool?"
    /// Passed at instantiation.
    pub total: Total,
    /// "When did this user's liquidity amount last change?"
    /// Set to current time on update.
    pub updated: Moment,
    /// "How much time has passed since the user updated their stake?"
    /// Computed as `current time - updated`
    pub elapsed: Duration,
    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    pub staked: Amount,
    /// What portion of the pool is currently owned by this user?
    /// Computed as user.staked / pool.staked
    pub pool_share: (Amount, Amount),
    /// How much liquidity has this user provided since they first appeared?
    /// Incremented on update by staked * elapsed if staked > 0
    pub volume: Volume,
    /// What was the volume of the pool when the user entered?
    /// Set to `total.volume` on initial deposit.
    pub starting_pool_volume: Volume,
    /// How much has `total.volume` grown, i.e. how much liquidity
    /// has accumulated in the pool since this user entered?
    /// Used as basis of reward share calculation.
    pub accumulated_pool_volume: Volume,
    /// What portion of all the liquidity accumulated since this user's entry
    /// is due to this particular user's stake? Computed as user.volume / pool.volume
    pub reward_share: (Volume, Volume),
    /// How much rewards were already unlocked when the user entered?
    /// Set to `total.unlocked` on initial deposit.
    pub starting_pool_rewards: Amount,
    /// How much has `total.unlocked` grown, i.e. how much rewards
    /// have been unlocked since this user entered?
    /// Multiply this by the reward share to compute earnings.
    pub accumulated_pool_rewards: Amount,
    /// How much rewards has this user earned?
    /// Computed as user.reward_share * pool.unlocked
    pub earned: Amount,
    /// How many units of time remain until the user can claim?
    /// Decremented on update, reset to pool.bonding on claim.
    pub bonding: Duration,
    /// Passed around internally, not presented to user.
    #[serde(skip)]
    pub address: HumanAddr,
    /// Passed around internally, not presented to user.
    #[serde(skip)]
    pub id: CanonicalAddr,
}
pub trait IAccount<S, A, Q, C>: Sized
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    /// Get the transaction initiator's account at current time
    fn from_env(core: &C, env: &Env) -> StdResult<Self>;
    /// Get the transaction initiator's account at specified time
    fn from_addr(core: &C, address: &HumanAddr, time: Moment) -> StdResult<Self>;
    /// Get an account with up-to-date values
    fn get(core: &C, total: Total, address: &HumanAddr) -> StdResult<Self>;
    /// Reset the user's liquidity conribution
    fn reset(&mut self, core: &mut C) -> StdResult<()>;
    /// Check if a deposit is possible, then perform it
    fn deposit(&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse>;
    /// Check if a withdrawal is possible, then perform it.
    fn withdraw(&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse>;
    /// Check if a claim is possible, then perform it
    fn claim(&mut self, core: &mut C) -> StdResult<HandleResponse>;
    /// Return the user's stake if trying to interact with a closed pool
    fn force_exit(&mut self, core: &mut C, when: Moment, why: String) -> StdResult<HandleResponse>;
    /// Store the values that were updated by the passing of time
    fn commit_elapsed(&mut self, core: &mut C) -> StdResult<()>;
    /// Store the results of a deposit
    fn commit_deposit(&mut self, core: &mut C, amount: Amount) -> StdResult<()>;
    /// Store the results of a withdrawal
    fn commit_withdrawal(&mut self, core: &mut C, amount: Amount) -> StdResult<()>;
    /// Store the results of a claim
    fn commit_claim(&mut self, core: &mut C) -> StdResult<()>;
}
impl<S, A, Q, C> IAccount<S, A, Q, C> for Account
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn from_env(core: &C, env: &Env) -> StdResult<Self> {
        Self::from_addr(core, &env.message.sender, env.block.time)
    }
    fn from_addr(core: &C, address: &HumanAddr, time: Moment) -> StdResult<Self> {
        Self::get(core, Total::from_time(core, time)?, address)
    }
    fn get(core: &C, total: Total, address: &HumanAddr) -> StdResult<Self> {
        let id = core.canonize(address.clone())?;
        let get_time = |key, default: u64| -> StdResult<u64> {
            Ok(core.get_ns(key, &id.as_slice())?.unwrap_or(default))
        };
        let get_amount = |key, default: Amount| -> StdResult<Amount> {
            Ok(core.get_ns(key, &id.as_slice())?.unwrap_or(default))
        };
        let get_volume = |key, default: Volume| -> StdResult<Volume> {
            Ok(core.get_ns(key, &id.as_slice())?.unwrap_or(default))
        };
        let mut account = Self::default();
        // 1. Timestamps
        //    Each user earns rewards as a function of their liquidity contribution over time.
        //    The following points and durations in time are stored for each user:
        //    * `updated` is the time of last update (deposit, withdraw or claim by this user)
        account.updated = get_time(Self::UPDATED, total.clock.now)?;
        if total.clock.now < account.updated {
            return errors::no_time_travel(3);
        }
        // 2. Liquidity and liquidity share
        //    * `staked` is the number of LP tokens staked by this user in this pool.
        //    * The user's **momentary share** is defined as `staked / total.staked`.
        //    * `volume` is the volume liquidity contributed by this user.
        //      It is incremented by `staked` for every moment elapsed.
        //    * The user's **volume share** is defined as `volume / total.volume`.
        //      It represents the user's overall contribution, and should move in the
        //      direction of the user's momentary share.
        account.staked = get_amount(Self::STAKED, Amount::zero())?;
        account.pool_share = (account.staked, total.staked);
        let last_volume = get_volume(Self::VOLUME, Volume::zero())?;
        account.elapsed = total.clock.now - account.updated;
        account.volume = accumulate(last_volume, account.elapsed, account.staked)?;
        account.starting_pool_volume = get_volume(Self::ENTRY_VOL, total.clock.volume)?;
        if account.starting_pool_volume > total.volume {
            return errors::no_time_travel(4);
        }
        account.accumulated_pool_volume = (total.volume - account.starting_pool_volume)?;
        // 3. Rewards claimable
        //    The `earned` rewards are a portion of the rewards unlocked since the epoch
        //    in which the user entered the pool.
        account.starting_pool_rewards = get_amount(Self::ENTRY_REW, total.unlocked)?;
        if account.starting_pool_rewards > total.unlocked {
            return errors::no_time_travel(5);
        }
        account.accumulated_pool_rewards = (total.unlocked - account.starting_pool_rewards)?;
        account.reward_share = (account.volume, account.accumulated_pool_volume);
        account.earned = if account.reward_share.1 == Volume::zero() {
            Amount::zero()
        } else {
            let reward = Volume::from(account.accumulated_pool_rewards)
                .multiply_ratio(account.reward_share.0, account.reward_share.1)?
                .low_u128();
            u128::min(total.budget.0, reward).into()
        };
        // 4. Bonding period
        // This decrements by `elapsed` if `staked > 0`.
        account.bonding = get_time(Self::BONDING, total.bonding)?;
        if account.staked > Amount::zero() {
            account.bonding = account.bonding.saturating_sub(account.elapsed)
        };
        // These are used above, then moved into the account struct at the end
        account.id = id;
        account.total = total;
        account.address = address.clone();
        Ok(account)
    }
    fn reset(&mut self, core: &mut C) -> StdResult<()> {
        self.starting_pool_volume = if self.staked == Amount::zero() {
            // If starting from scratch amidst an epoch,
            // count from the start of the epoch.
            self.total.clock.volume
        } else {
            // If reset is due to claim, there is no gap in liquidity provision,
            // therefore count from the current moment.
            self.total.volume
        };
        core.set_ns(
            Self::ENTRY_VOL,
            self.id.as_slice(),
            self.starting_pool_volume,
        )?;
        self.starting_pool_rewards = self.total.unlocked;
        core.set_ns(
            Self::ENTRY_REW,
            self.id.as_slice(),
            self.starting_pool_rewards,
        )?;
        self.bonding = self.total.bonding;
        core.set_ns(Self::BONDING, self.id.as_slice(), self.bonding)?;
        self.volume = Volume::zero();
        core.set_ns(Self::VOLUME, self.id.as_slice(), self.volume)?;
        self.updated = self.total.clock.now;
        core.set_ns(Self::UPDATED, self.id.as_slice(), self.updated)?;
        Ok(())
    }
    fn deposit(&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = self.total.closed {
            let when = when.clone();
            let why = why.clone();
            return self.force_exit(core, when, why);
        } else {
            self.commit_deposit(core, amount)?;
            let lp_token = RewardsConfig::lp_token(core)?;
            let self_link = RewardsConfig::self_link(core)?;
            HandleResponse::default().msg(lp_token.transfer_from(
                &self.address,
                &self_link.address,
                amount,
            )?)
        }
    }
    fn withdraw(&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = self.total.closed {
            let when = when.clone();
            let why = why.clone();
            self.force_exit(core, when, why)
        } else if self.staked < amount {
            errors::withdraw(self.staked, amount)
        } else if self.total.staked < amount {
            errors::withdraw_fatal(self.total.staked, amount)
        } else {
            // testing pub/sub between traits. Message string will be enum, not a string
            // let polls = core.broadcast::<Moment, Vec<u64>>("get_active_polls", self.total.clock.now)?;

            // TODO probably must go into commit_withdrawal since other methods call it directly
            let user = User::get(core, &self.address, self.total.clock.now)?;
            let threshold = GovernanceConfig::threshold(core)?;
            if !user.can_unstake(self.staked.u128(), threshold.into(), amount.u128()) {
                errors::unstake_disallowed()?
            }

            self.commit_withdrawal(core, amount)?;
            let mut response = HandleResponse::default();
            // If all tokens were withdrawn
            if self.staked == Amount::zero() {
                // And if there is some reward claimable
                if self.earned > Amount::zero() && self.bonding == 0 {
                    // Also transfer rewards
                    self.commit_claim(core)?;
                    let reward_token = RewardsConfig::reward_token(core)?;
                    response = response
                        .msg(reward_token.transfer(&self.address, self.earned)?)?
                        .log("reward", &self.earned.to_string())?;
                } else {
                    // If bonding is not over yet just reset even if some rewards were earned
                    self.reset(core)?;
                }
            }
            // Transfer withdrawn stake
            response.msg(RewardsConfig::lp_token(core)?.transfer(&self.address, amount)?)
        }
    }
    fn claim(&mut self, core: &mut C) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = self.total.closed {
            let when = when.clone();
            let why = why.clone();
            self.force_exit(core, when, why)
        } else if self.bonding > 0 {
            errors::claim_bonding(self.bonding)
        } else if self.total.budget == Amount::zero() {
            errors::claim_pool_empty()
        } else if self.earned == Amount::zero() {
            errors::claim_zero_claimable()
        } else {
            self.commit_claim(core)?;
            HandleResponse::default()
                .msg(RewardsConfig::reward_token(core)?.transfer(&self.address, self.earned)?)?
                .log("reward", &self.earned.to_string())
        }
    }
    fn force_exit(&mut self, core: &mut C, when: Moment, why: String) -> StdResult<HandleResponse> {
        let response = HandleResponse::default()
            .msg(RewardsConfig::lp_token(core)?.transfer(&self.address, self.staked)?)?
            .msg(RewardsConfig::reward_token(core)?.transfer(&self.address, self.earned)?)?
            .log("close_time", &format!("{}", when))?
            .log("close_reason", &format!("{}", why))?;
        self.commit_withdrawal(core, self.staked)?;
        self.commit_claim(core)?;
        Ok(response)
    }
    fn commit_elapsed(&mut self, core: &mut C) -> StdResult<()> {
        self.total.commit_elapsed(core)?;
        if self.staked == Amount::zero() {
            self.reset(core)?;
        } else {
            core.set_ns(Self::BONDING, self.id.as_slice(), self.bonding)?;
            core.set_ns(Self::VOLUME, self.id.as_slice(), self.volume)?;
            core.set_ns(Self::UPDATED, self.id.as_slice(), self.total.clock.now)?;
        }
        Ok(())
    }
    fn commit_deposit(&mut self, core: &mut C, amount: Amount) -> StdResult<()> {
        let user = User::get(core, &self.address, self.total.clock.now)?;
        user.active_polls.into_iter().for_each(|poll_id| {
            User::increase_vote_power(
                core,
                poll_id,
                &self.address,
                amount,
                self.total.clock.now,
            )
            .expect("Increasing voting power failed");
        });

        self.commit_elapsed(core)?;
        self.staked += amount;
        core.set_ns(Self::STAKED, self.id.as_slice(), self.staked)?;
        self.total.staked += amount;
        core.set(Total::STAKED, self.total.staked)?;
        Ok(())
    }
    fn commit_withdrawal(&mut self, core: &mut C, amount: Amount) -> StdResult<()> {
        self.commit_elapsed(core)?;
        self.staked = (self.staked - amount)?;
        core.set_ns(Self::STAKED, self.id.as_slice(), self.staked)?;
        self.total.staked = (self.total.staked - amount)?;
        core.set(Total::STAKED, self.total.staked)?;
        Ok(())
    }
    fn commit_claim(&mut self, core: &mut C) -> StdResult<()> {
        if self.earned > Amount::zero() {
            self.reset(core)?;
            self.total.commit_claim(core, self.earned)?;
        }
        Ok(())
    }
}
impl Account {
    pub const ENTRY_VOL: &'static [u8] = b"/user/entry_vol/";
    pub const ENTRY_REW: &'static [u8] = b"/user/entry_rew/";
    pub const STAKED: &'static [u8] = b"/user/current/";
    pub const UPDATED: &'static [u8] = b"/user/updated/";
    pub const VOLUME: &'static [u8] = b"/user/volume/";
    pub const BONDING: &'static [u8] = b"/user/bonding/";
}

/// Amount of funds
pub type Amount = Uint128;

/// Amount multiplied by duration.
pub type Volume = Uint256;
/// A ratio, represented as tuple (nom, denom)
pub type Ratio = (Uint128, Uint128);
/// When and why was the pool closed
pub type CloseSeal = (Moment, String);
/// Address and, optionally, viewing key
pub type AccountSnapshot = (HumanAddr, Option<String>, Amount);
/// Project current value of an accumulating parameter based on stored value,
/// time since it was last updated, and rate of change, i.e.
/// `current = stored + (elapsed * rate)`
// * The need to store detailed history (and iterate over it, unboundedly)
//   is avoided by using continuously accumulating values.
// * The state can't be updated outside of a transaction,
//   the current values of the accumulators need to be computed as
//   `last value + (elapsed * rate)`, where:
//   * `last value` is fetched from storage
//   * `elapsed` is `now - last update`
//     * v2 measures time in blocks
//     * v3 measures time in seconds
//     * For transactions, `now` is `env.block.time`.
//     * For queries, `now` has to be passed by the client.
//   * `rate` depends on what is being computed:
//     * `total.volume` grows by `total.staked` every moment.
//     * `user.volume` grows by `user.staked` every moment.
//     * `user.bonding` decreases by 1 every moment, until it reaches 0.
pub fn accumulate(
    total_before_last_update: Volume,
    time_since_last_update: Duration,
    value_after_last_update: Amount,
) -> StdResult<Volume> {
    let increment =
        Volume::from(value_after_last_update).multiply_ratio(time_since_last_update, 1u128)?;
    total_before_last_update + increment
}
