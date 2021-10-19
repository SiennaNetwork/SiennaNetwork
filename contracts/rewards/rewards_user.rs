use std::{rc::Rc, cell::RefCell};

use crate::{
    rewards_math::*,
    rewards_field::{Field, FieldFactory},
    rewards_pool::Pool
};

use fadroma::scrt::{
    cosmwasm_std::{StdError, CanonicalAddr},
    storage::*
};

/// User account
pub struct User <S> {
    pub storage: Rc<RefCell<S>>,
    pub pool:    Pool<S>,
    pub address: CanonicalAddr,

    /// How much liquidity has this user provided since they first appeared.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// in intervals of (moments since last update * current balance)
    last_lifetime: Field<S, Volume>,

    /// How much liquidity does this user currently provide.
    /// Incremented/decremented on lock/unlock.
    locked:        Field<S, Amount>,

    /// When did this user's liquidity amount last change
    /// Set to current time on lock/unlock.
    timestamp:     Field<S, Time>,

    /// How much rewards has each user claimed so far.
    /// Incremented on claim.
    claimed:       Field<S, Amount>,

    #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
    /// For how many units of time has this user provided liquidity
    /// On lock/unlock, if locked was > 0 before the operation,
    /// this is incremented by time elapsed since last update.
    present:       Field<S, Time>,

    #[cfg(feature="user_liquidity_ratio")]
    /// For how many units of time has this user been known to the contract.
    /// Incremented on lock/unlock by time elapsed since last update.
    last_existed:  Field<S, Time>,

    #[cfg(feature="claim_cooldown")]
    /// For how many units of time has this user provided liquidity
    /// Decremented on lock/unlock, reset to configured cooldown on claim.
    cooldown:      Field<S, Time>
}

impl <S: Storage> User <S> {

    pub fn new (pool: Pool<S>, address: CanonicalAddr) -> Self {
        let storage = pool.storage;
        User {
            storage: storage.clone(),
            pool:    pool,
            address,

            last_lifetime: storage.field(&concat(b"/user/lifetime/", address.as_slice())),
            locked:        storage.field(&concat(b"/user/current/",  address.as_slice())),
            timestamp:     storage.field(&concat(b"/user/updated/",  address.as_slice())),
            claimed:       storage.field(&concat(b"/user/claimed/",  address.as_slice())),

            #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
            present:       storage.field(&concat(b"/user/present/",  address.as_slice())),

            #[cfg(feature="user_liquidity_ratio")]
            last_existed:  storage.field(&concat(b"/user/existed/",  address.as_slice())),

            #[cfg(feature="claim_cooldown")]
            cooldown:      storage.field(&concat(b"/user/cooldown/", address.as_slice())),
        }
    }

    // time-related getters --------------------------------------------------------------------

    /// Time of last lock or unlock
    pub fn timestamp (&self) -> StdResult<Option<Time>> {
        self.timestamp.value()
    }

    #[cfg(any(feature="claim_cooldown", feature="user_liquidity_ratio"))]
    /// Time that progresses always. Used to increment existence.
    pub fn elapsed (&self) -> StdResult<Time> {
        let now = self.pool.now()?;

        if let Ok(Some(timestamp)) = self.timestamp() {
            if now < timestamp { // prevent replay
                return Err(StdError::generic_err("no time travel"))
            } else {
                Ok(now - timestamp)
            }
        } else {
            Ok(0 as Time)
        }
    }

    /// Time that progresses only when the user has some tokens locked.
    /// Used to increment presence and lifetime.
    pub fn elapsed_present (&self) -> StdResult<Time> {
        if self.locked()? > Amount::zero() {
            self.elapsed()
        } else {
            Ok(0 as Time)
        }
    }

    // user existence = time since this user first locked tokens -------------------------------

    #[cfg(feature="user_liquidity_ratio")]
    /// Up-to-date time for which the user has existed
    pub fn existed (&self) -> StdResult<Time> {
        Ok(self.last_existed()? + self.elapsed()?)
    }

    #[cfg(feature="user_liquidity_ratio")]
    /// Load last value of user existence
    pub fn last_existed (&self) -> StdResult<Time> {
        self.last_existed.value_or_default(0 as Time)
    }

    #[cfg(feature="user_liquidity_ratio")]
    pub fn liquidity_ratio (&self) -> StdResult<Amount> {
        Ok(Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(self.present()?, self.existed()?)?
            .low_u128().into())
    }

    // user presence = time the user has had >0 LP tokens locked in the pool -------------------

    #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
    /// Up-to-date time for which the user has provided liquidity
    pub fn present (&self) -> StdResult<Time> {
        Ok(self.last_present()? + self.elapsed_present()?)
    }

    #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
    /// Load last value of user present
    pub fn last_present (&self) -> StdResult<Time> {
        self.present.value_or_default(0 as Time)
    }

    // cooldown - reset on claim, decremented towards 0 as time advances -----------------------

    #[cfg(feature="claim_cooldown")]
    pub fn cooldown (&self) -> StdResult<Time> {
        #[cfg(feature="pool_closes")]
        if self.pool.closed()?.is_some() {
            return Ok(0u64) }
        Ok(Time::saturating_sub(self.last_cooldown()?, self.elapsed()?))
    }

    #[cfg(feature="claim_cooldown")]
    fn last_cooldown (&self) -> StdResult<Time> {
        self.cooldown.value_or_default(self.pool.cooldown()?)
    }

    // lp-related getters ----------------------------------------------------------------------

    pub fn locked (&self) -> StdResult<Amount> {
        self.locked.value_or_default(Amount::zero())
    }

    pub fn lifetime (&self) -> StdResult<Volume> {
        tally(self.last_lifetime()?, self.elapsed_present()?, self.locked()?)
    }

    fn last_lifetime (&self) -> StdResult<Volume> {
        self.last_lifetime.value_or_default(Volume::zero())
    }

    // reward-related getters ------------------------------------------------------------------

    // After first locking LP tokens, users must reach a configurable age threshold,
    // i.e. keep LP tokens locked for at least X blocks. During that time, their portion of
    // the total liquidity ever provided increases.
    //
    // The total reward for an user with an age under the threshold is zero.
    //
    // The total reward for a user with an age above the threshold is
    // (claimed_rewards + budget) * user_lifetime_liquidity / pool_lifetime_liquidity
    //
    // Since a user's total reward can diminish, it may happen that the amount claimed
    // by a user so far is larger than the current total reward for that user.
    // In that case the user's claimable amount remains zero until they unlock more
    // rewards than they've already claimed.
    //
    // Since a user's total reward can diminish, it may happen that the amount remaining
    // in the pool after a user has claimed is insufficient to pay out the next user's reward.
    // In that case, https://google.github.io/filament/webgl/suzanne.html

    pub fn share (&self, basis: u128) -> StdResult<Volume> {
        let share = Volume::from(basis);

        // reduce lifetime by normal lifetime ratio
        let share = share.diminish_or_zero(self.lifetime()?, self.pool.lifetime()?)?;

        // reduce lifetime by liquidity ratio
        #[cfg(feature="user_liquidity_ratio")]
        let share = share.diminish_or_zero(self.present()?, self.existed()?)?;

        Ok(share)
    }

    pub fn earned (&self) -> StdResult<Amount> {
        let mut budget = Amount::from(self.pool.budget()?);

        #[cfg(feature="pool_liquidity_ratio")] {
            budget = budget.diminish_or_zero(self.pool.liquid()?, self.pool.existed()?)?;
        }

        #[cfg(feature="global_ratio")] {
            let ratio = self.pool.global_ratio()?;
            budget = budget.diminish_or_zero(ratio.0, ratio.1)?
        }

        Ok(self.share(budget.u128())?.low_u128().into())
    }

    pub fn claimed (&self) -> StdResult<Amount> {
        self.claimed.value_or_default(Amount::zero())
    }

    pub fn claimable (&self) -> StdResult<Amount> {
        #[cfg(feature="age_threshold")]
        // can only claim after age threshold
        if self.present()? < self.pool.threshold()? {
            return Ok(Amount::zero())
        }

        // can only claim if earned something
        let earned = self.earned()?;
        if earned == Amount::zero() {
            return Ok(Amount::zero())
        }

        // can only claim if earned > claimed
        let claimed = self.claimed()?;
        if earned <= claimed {
            return Ok(Amount::zero())
        }

        // can only claim if the pool has balance
        let claimable = (earned - claimed)?;
        // not possible to claim more than the remaining pool balance
        if claimable > self.pool.balance() {
            Ok(balance)
        } else {
            Ok(claimable)
        }
    }

    // time-related mutations ------------------------------------------------------------------

    #[cfg(feature="claim_cooldown")]
    fn reset_cooldown (&mut self) -> StdResult<()> {
        let address = self.address.clone();
        self.cooldown.store(&self.pool.cooldown()?)?;
        Ok(())
    }

    // lp-related mutations -------------------------------------------------------------------

    fn update (&mut self, user_locked: Amount, pool_locked: Amount) -> StdResult<&mut Self> {
        // Prevent replay
        let now = self.pool.now()?;
        if let Some(timestamp) = self.timestamp()? {
            if timestamp > now {
                return Err(StdError::generic_err("no time travel"))
            }
        }

        // Commit rolling values to storage:

        let address = self.address.clone();

        #[cfg(feature="user_liquidity_ratio")] {
            // Increment existence
            self.last_existed.store(&self.existed()?)?;
        }

        #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))] {
            // Increment presence if user has currently locked tokens
            self.present.store(&self.present()?)?;
        }

        #[cfg(feature="claim_cooldown")] {
            // Cooldown is calculated since the timestamp.
            // Since we'll be updating the timestamp, commit the current cooldown
            let cooldown = self.cooldown()?;
            self.cooldown.store(&cooldown)?;
        }

        let lifetime = self.lifetime()?;
        self.last_lifetime.store(&lifetime)?;// Always increment lifetime
        self.timestamp.store(&now)?;// Set user's time of last update to now
        self.locked.store(&user_locked)?;// Update amount locked
        self.pool.update_locked(pool_locked)?;// Update total amount locked in pool

        Ok(self)
    }

    pub fn lock_tokens (&mut self, increment: Amount) -> StdResult<Amount> {
        let locked = self.locked()?;
        self.update(
            locked + increment,
            self.pool.locked()? + increment.into())?;
        // Return the amount to be transferred from the user to the contract
        Ok(increment)
    }

    pub fn retrieve_tokens (&mut self, decrement: Amount) -> StdResult<Amount> {
        // Must have enough locked to retrieve
        let locked = self.locked()?;
        if locked < decrement {
            return Err(StdError::generic_err(format!("not enough locked ({} < {})", locked, decrement)))
        }
        self.update(
            (self.locked()? - decrement)?,
            (self.pool.locked()? - decrement.into())?
        )?;
        // Return the amount to be transferred back to the user
        Ok(decrement)
    }

    // reward-related mutations ----------------------------------------------------------------

    fn increment_claimed (&mut self, reward: Amount) -> StdResult<()> {
        let address = self.address.clone();
        self.pool.increment_claimed(reward)?;
        self.claimed.store(&(self.claimed()? + reward))?;
        Ok(())
    }

    pub fn claim_reward (&mut self) -> StdResult<Amount> {
        #[cfg(feature="age_threshold")]
        // If user must wait before their first claim, enforce that here.
        enforce_cooldown(self.present()?, self.pool.threshold()?)?;

        #[cfg(feature="claim_cooldown")]
        // If user must wait between claims, enforce that here.
        enforce_cooldown(0, self.cooldown()?)?;

        // See if there is some unclaimed reward amount:
        let claimable = self.claimable()?;
        if claimable == Amount::zero() {
            return Err(StdError::generic_err(
                "You've already received as much as your share of the reward pool allows. \
                Keep your liquidity tokens locked and wait for more rewards to be vested, \
                and/or lock more liquidity tokens to grow your share of the reward pool."
            ))
        }

        // Now we need the user's liquidity token balance for two things:
        let locked = self.locked()?;

        // 1. Update the user timestamp, and the other things that may be synced to it.
        //    Sacrifices efficiency (gas cost for a few more load/save operations than
        //    the absolute minimum) for an avoidance of hidden dependencies.
        self.update(locked, self.pool.locked()?)?;

        // (In the meantime, update how much has been claimed...
        self.increment_claimed(claimable)?;

        // ...and, optionally, reset the cooldown so that
        // the user has to wait before claiming again)
        #[cfg(feature="claim_cooldown")]
        self.reset_cooldown()?; // Reset the user cooldown

        // 2. Optionally, reset the user's `lifetime` and `share` if they have currently
        //    0 tokens locked. The intent is for this to be the user's last reward claim
        //    after they've left the pool completely. If they provide exactly 0 liquidity
        //    at some point, when they come back they have to start over, which is OK
        //    because they can then start claiming rewards immediately, without waiting
        //    for threshold, only cooldown.
        #[cfg(feature="selective_memory")] {
            if locked == Amount::zero() {
                let address = self.address.clone();
                self.last_lifetime.store(&Volume::zero())?;
                self.claimed.store(&Amount::zero())?;
            }
        }

        // Return the amount that the contract will send to the user
        Ok(claimable)
    }

    #[cfg(all(test, feature="user_liquidity_ratio"))]
    pub fn reset_liquidity_ratio (&self) -> StdResult<()> {
        let address = self.address.clone();
        let existed = self.existed()?;
        self.update(self.locked()?, self.pool.locked()?)?;
        self.present.store(existed)?;
        Ok(())
    }

}

#[cfg(any(feature="claim_cooldown", feature="age_threshold"))]
fn enforce_cooldown (elapsed: Time, cooldown: Time) -> StdResult<()> {
    if elapsed >= cooldown {
        Ok(())
    } else {
        Err(StdError::generic_err(format!("lock tokens for {} more seconds to be eligible", cooldown - elapsed)))
    }
}
