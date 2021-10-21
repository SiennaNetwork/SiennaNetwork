use std::{rc::Rc, cell::RefCell};

use crate::{
    rewards_math::*,
    rewards_field::*,
    rewards_pool::*
};

use fadroma::scrt::{
    cosmwasm_std::*,
    storage::*
};

/// User account
pub struct User <'p, S: Storage, A: Api, Q: Querier> {
    pub pool:    &'p mut Pool<S, A, Q>,
    pub deps:    Rc<RefCell<Extern<S, A, Q>>>,
    pub address: CanonicalAddr,

    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    pub locked:    Field<S, A, Q, Amount>,

    /// When did this user's liquidity amount last change?
    /// Set to current time on lock/unlock.
    pub timestamp: Field<S, A, Q, Time>,

    /// How much rewards has this user claimed so far?
    /// Incremented on claim.
    pub claimed:   Field<S, A, Q, Amount>,

    /// How much liquidity has this user provided since they first appeared.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// in intervals of (moments since last update * current balance)
    last_lifetime: Field<S, A, Q, Volume>,

    #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
    /// For how many units of time has this user provided liquidity
    /// On lock/unlock, if locked was > 0 before the operation,
    /// this is incremented by time elapsed since last update.
    last_present:  Field<S, A, Q, Time>,

    #[cfg(feature="user_liquidity_ratio")]
    /// For how many units of time has this user been known to the contract.
    /// Incremented on lock/unlock by time elapsed since last update.
    last_existed:  Field<S, A, Q, Time>,

    #[cfg(feature="claim_cooldown")]
    /// For how many units of time has this user provided liquidity
    /// Decremented on lock/unlock, reset to configured cooldown on claim.
    last_cooldown: Field<S, A, Q, Time>
}

impl <'p, S: Storage, A: Api, Q: Querier> User <'p, S, A, Q> {

    pub fn new (
        pool:    &'p mut Pool<S, A, Q>,
        address: CanonicalAddr
    ) -> Self {
        let deps = pool.deps;
        User {
            deps: deps.clone(),
            pool: pool,
            address,

            last_lifetime: deps.field(&concat(b"/user/lifetime/", address.as_slice()))
                                  .or(Volume::zero()),

            locked:        deps.field(&concat(b"/user/current/",  address.as_slice()))
                                  .or(Amount::zero()),

            timestamp:     deps.field(&concat(b"/user/updated/",  address.as_slice()))
                                  .or(pool.now().unwrap()),

            claimed:       deps.field(&concat(b"/user/claimed/",  address.as_slice()))
                                  .or(Amount::zero()),

            #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
            last_present:  deps.field(&concat(b"/user/present/",  address.as_slice()))
                                  .or(0u64),

            #[cfg(feature="user_liquidity_ratio")]
            last_existed:  deps.field(&concat(b"/user/existed/",  address.as_slice()))
                                  .or(0u64),

            #[cfg(feature="claim_cooldown")]
            last_cooldown: deps.field(&concat(b"/user/cooldown/", address.as_slice()))
                                  .or(pool.cooldown.get().unwrap()),
        }
    }

    // time-related getters --------------------------------------------------------------------


    #[cfg(any(feature="claim_cooldown", feature="user_liquidity_ratio"))]
    /// Time that progresses always. Used to increment existence.
    pub fn elapsed (&self) -> StdResult<Time> {
        let now = self.pool.now()?;
        if let Ok(timestamp) = self.timestamp.get() {
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
    pub fn elapsed_while_present (&self) -> StdResult<Time> {
        if self.locked.get()? > Amount::zero() {
            self.elapsed()
        } else {
            Ok(0 as Time)
        }
    }

    // user existence = time since this user first locked tokens -------------------------------

    #[cfg(feature="user_liquidity_ratio")]
    /// Up-to-date time for which the user has existed
    pub fn existed (&self) -> StdResult<Time> {
        Ok(self.last_existed.get()? + self.elapsed()?)
    }

    #[cfg(feature="user_liquidity_ratio")]
    pub fn liquidity_ratio (&self) -> StdResult<Amount> {
        Ok(
            Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(self.present()?, self.existed()?)?
            .low_u128().into()
        )
    }

    // user presence = time the user has had >0 LP tokens locked in the pool -------------------

    #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
    /// Up-to-date time for which the user has provided liquidity
    pub fn present (&self) -> StdResult<Time> {
        Ok(self.last_present.get()? + self.elapsed_while_present()?)
    }

    // cooldown - reset on claim, decremented towards 0 as time advances -----------------------

    #[cfg(feature="claim_cooldown")]
    pub fn cooldown (&self) -> StdResult<Time> {
        #[cfg(feature="pool_closes")]
        if self.pool.closed.get()?.is_some() {
            return Ok(0u64)
        }
        Ok(Time::saturating_sub(
            self.last_cooldown.get()?,
            self.elapsed()?
        ))
    }

    // lp-related getters ----------------------------------------------------------------------

    pub fn lifetime (&self) -> StdResult<Volume> {
        tally(
            self.last_lifetime.get()?,
            self.elapsed_while_present()?,
            self.locked.get()?
        )
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
            let ratio = self.pool.global_ratio.get()?;
            budget = budget.diminish_or_zero(ratio.0, ratio.1)?
        }

        Ok(self.share(budget.u128())?.low_u128().into())
    }

    pub fn claimable (&self) -> StdResult<Amount> {
        #[cfg(feature="age_threshold")]
        // can only claim after age threshold
        if self.present()? < self.pool.threshold.get()? {
            return Ok(Amount::zero())
        }

        // can only claim if earned something
        let earned = self.earned()?;
        if earned == Amount::zero() {
            return Ok(Amount::zero())
        }

        // can only claim if earned > claimed
        let claimed = self.claimed.get()?;
        if earned <= claimed {
            return Ok(Amount::zero())
        }

        // can only claim if the pool has balance
        let balance = self.pool.balance();
        let claimable = (earned - claimed)?;
        // not possible to claim more than the remaining pool balance
        if claimable > balance {
            Ok(balance)
        } else {
            Ok(claimable)
        }
    }

    pub fn lock_tokens (&mut self, increment: Amount) -> StdResult<Amount> {
        self.update(
            self.locked.get()? + increment,
            self.pool.locked.get()? + increment.into()
        )?;
        // Return the amount to be transferred from the user to the contract
        Ok(increment)
    }

    pub fn retrieve_tokens (&mut self, decrement: Amount) -> StdResult<Amount> {
        // Must have enough locked to retrieve
        let locked = self.locked.get()?;
        if locked < decrement {
            return Err(StdError::generic_err(format!("not enough locked ({} < {})", locked, decrement)))
        }
        self.update(
            (locked - decrement)?,
            (self.pool.locked.get()? - decrement.into())?
        )?;
        // Return the amount to be transferred back to the user
        Ok(decrement)
    }

    pub fn claim_reward (&mut self) -> StdResult<Amount> {

        // If user must wait before first claim, enforce that here.
        #[cfg(feature="age_threshold")]
        enforce_cooldown(self.present()?, self.pool.threshold.get()?)?;

        // If user must wait between claims, enforce that here.
        #[cfg(feature="claim_cooldown")]
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
        let locked = self.locked.get()?;

        // Update user timestamp, and the things synced to it.
        self.update(locked, self.pool.locked.get()?)?;

        // Update how much has been claimed
        self.increment_claimed(claimable)?;

        // ...and, optionally, reset the cooldown so that
        // the user has to wait before claiming again)
        #[cfg(feature="claim_cooldown")]
        self.last_cooldown.set(&self.pool.cooldown.get()?)?;

        // 2. Optionally, reset the user's `lifetime` and `share` if they have currently
        //    0 tokens locked. The intent is for this to be the user's last reward claim
        //    after they've left the pool completely. If they provide exactly 0 liquidity
        //    at some point, when they come back they have to start over, which is OK
        //    because they can then start claiming rewards immediately, without waiting
        //    for threshold, only cooldown.
        #[cfg(feature="selective_memory")]
        if locked == Amount::zero() {
            self.last_lifetime.set(&Volume::zero())?;
            self.claimed.set(&Amount::zero())?;
        }

        // Return the amount that the contract will send to the user
        Ok(claimable)

    }

    fn increment_claimed (&mut self, reward: Amount) -> StdResult<()> {
        self.pool.increment_claimed(reward)?;
        self.claimed.set(&(self.claimed.get()? + reward))?;
        Ok(())
    }

    #[cfg(all(test, feature="user_liquidity_ratio"))]
    pub fn reset_liquidity_ratio (&self) -> StdResult<()> {
        let existed = self.existed()?;
        self.update(self.locked()?, self.pool.locked.get()?)?;
        self.present.set(existed)?;
        Ok(())
    }

    /// Commit rolling values to deps
    fn update (&mut self, user_locked: Amount, pool_locked: Amount) -> StdResult<&mut Self> {
        // Prevent replay
        let now = self.pool.now()?;
        if let Ok(timestamp) = self.timestamp.get() {
            if timestamp > now {
                return Err(StdError::generic_err("no time travel"))
            }
        }

        // Increment existence
        #[cfg(feature="user_liquidity_ratio")]
        self.last_existed.set(&self.existed()?)?;

        // Increment presence if user has currently locked tokens
        #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
        self.last_present.set(&self.present()?)?;

        // Cooldown is calculated since the timestamp.
        // Since we'll be updating the timestamp, commit the current cooldown
        #[cfg(feature="claim_cooldown")]
        self.last_cooldown.set(&self.cooldown()?)?;

        // Always increment lifetime
        self.last_lifetime.set(&self.lifetime()?)?;

        // Set user's time of last update to now
        self.timestamp.set(&now)?;

        // Update amount locked
        self.locked.set(&user_locked)?;

        // Update total amount locked in pool
        self.pool.update_locked(pool_locked)?;

        Ok(self)
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
