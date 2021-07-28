use crate::rewards_math::*;
use fadroma::scrt::{
    cosmwasm_std::{StdError, CanonicalAddr},
    storage::*
};

macro_rules! error { ($info:expr) => { Err(StdError::generic_err($info)) }; }

/// Store the moment the user is created to compute total pool existence.
/// Set on init.
const POOL_CREATED:   &[u8] = b"/pool/created";

#[cfg(feature="pool_liquidity_ratio")]
/// Used to compute what portion of the time the pool was not empty.
/// On lock/unlock, if the pool was not empty, this is incremented
/// by the time elapsed since the last update.
const POOL_LIQUID:    &[u8] = b"/pool/not_empty";

/// How much liquidity has this pool contained up to this point.
/// On lock/unlock, if locked > 0 before the operation, this is incremented
/// in intervals of (moments since last update * current balance)
const POOL_LIFETIME:  &[u8] = b"/pool/lifetime";

/// How much liquidity is there in the whole pool right now.
/// Incremented/decremented on lock/unlock.
const POOL_LOCKED:    &[u8] = b"/pool/balance";

/// When was liquidity last updated.
/// Set to current time on lock/unlock.
const POOL_TIMESTAMP: &[u8] = b"/pool/updated";

/// Rewards claimed by everyone so far.
/// Incremented on claim.
const POOL_CLAIMED:   &[u8] = b"/pool/claimed";

/// Ratio of liquidity provided to rewards received.
/// Configured on init.
const POOL_RATIO:     &[u8] = b"/pool/ratio";

/// How much the user needs to wait before they can claim for the first time.
/// Configured on init.
const POOL_THRESHOLD: &[u8] = b"/pool/threshold";

/// How much the user must wait between claims.
/// Configured on init.
const POOL_COOLDOWN:  &[u8] = b"/pool/cooldown";

/// How much liquidity has this user provided since they first appeared.
/// On lock/unlock, if the pool was not empty, this is incremented
/// in intervals of (moments since last update * current balance)
const USER_LIFETIME:  &[u8] = b"/user/lifetime/";

/// How much liquidity does this user currently provide.
/// Incremented/decremented on lock/unlock.
const USER_LOCKED:    &[u8] = b"/user/current/";

/// When did this user's liquidity amount last change
/// Set to current time on lock/unlock.
const USER_TIMESTAMP: &[u8] = b"/user/updated/";

/// How much rewards has each user claimed so far.
/// Incremented on claim.
const USER_CLAIMED:   &[u8] = b"/user/claimed/";

/// For how many units of time has this user provided liquidity
/// Incremented on lock/unlock by time elapsed since last update.
const USER_EXISTED:   &[u8] = b"/user/existed/";

/// For how many units of time has this user provided liquidity
/// On lock/unlock, if locked was > 0 before the operation,
/// this is incremented by time elasped since last update.
const USER_PRESENT:   &[u8] = b"/user/present/";

/// For how many units of time has this user provided liquidity
/// Decremented on lock/unlock, reset to configured cooldown on claim.
const USER_COOLDOWN:  &[u8] = b"/user/cooldown/";

/// Reward pool
pub struct Pool <S> {
    pub storage: S,
    now:         Option<Time>,
    balance:     Option<Amount>
}

/// User account
pub struct User <S> {
    pool:    Pool<S>,
    address: CanonicalAddr
}

impl <S> Pool<S> {
    /// Create a new pool with a storage handle
    pub fn new (storage: S) -> Self {
        Self { storage, now: None, balance: None }
    }
    /// Set the current time
    pub fn at (self, now: Time) -> Self {
        Self { storage: self.storage, balance: self.balance, now: Some(now) }
    }
    /// Set the current balance
    pub fn with_balance (self, balance: Amount) -> Self {
        Self { storage: self.storage, now: self.now, balance: Some(balance) }
    }
    /// Get an individual user from the pool
    pub fn user (self, address: CanonicalAddr) -> User<S> {
        User { pool: self, address }
    }
}

stateful!(Pool (storage):

    Readonly {

        /// Time since the pool was created
        pub fn existed (&self) -> StdResult<Time> {
            Ok(self.now()? - self.created()?) }

        fn created (&self) -> StdResult<Time> {
            match self.load(POOL_CREATED)? {
                Some(created) => Ok(created),
                None => Err(StdError::generic_err("missing creation date")) } }

        #[cfg(feature="pool_liquidity_ratio")]
        /// Time for which the pool was not empty.
        pub fn liquid (&self) -> StdResult<Time> {
            Ok(self.last_liquid()? + if self.locked()? > Amount::zero() {
                self.elapsed()?
            } else {
                0 as Time
            }) }

        #[cfg(feature="pool_liquidity_ratio")]
        fn last_liquid (&self) -> StdResult<Time> {
            match self.load(POOL_LIQUID)? {
                Some(created) => Ok(created),
                None => Ok(0 as Time) } }

        #[cfg(feature="pool_liquidity_ratio")]
        pub fn liquidity_ratio (&self) -> StdResult<Amount> {
            Ok(Volume::from(HUNDRED_PERCENT)
                .multiply_ratio(self.liquid()?, self.existed()?)?
                .low_u128().into())
        }

        /// The total liquidity ever contained in this pool.
        pub fn lifetime (&self) -> StdResult<Volume> {
            tally(self.last_lifetime()?, self.elapsed()?, self.locked()?) }

        /// Snapshot of total liquidity at moment of last update.
        fn last_lifetime (&self) -> StdResult<Volume> {
            Ok(self.load(POOL_LIFETIME)?.unwrap_or(Volume::zero())) }

        /// Get the time since last update (0 if no last update)
        pub fn elapsed (&self) -> StdResult<Time> {
            Ok(self.now()? - self.timestamp()?) }

        /// Get the current time or fail
        pub fn now (&self) -> StdResult<Time> {
            self.now.ok_or(StdError::generic_err("current time not set")) }

        /// Load the last update timestamp or default to current time
        /// (this has the useful property of keeping `elapsed` zero for strangers)
        pub fn timestamp (&self) -> StdResult<Time> {
            match self.load(POOL_TIMESTAMP)? {
                Some(time) => Ok(time),
                None       => Ok(self.now()?) } }

        /// Amount of currently locked LP tokens in this pool
        pub fn locked (&self) -> StdResult<Amount> {
            Ok(self.load(POOL_LOCKED)?.unwrap_or(Amount::zero())) }

        /// Amount of rewards already claimed
        pub fn claimed (&self) -> StdResult<Amount> {
            Ok(self.load(POOL_CLAIMED)?.unwrap_or(Amount::zero())) }

        /// The full reward budget = rewards claimed + current balance of this contract in reward token
        pub fn budget (&self) -> StdResult<Amount> {
            Ok(self.claimed()? + self.balance()) }

        /// Current balance in reward token, or zero.
        pub fn balance (&self) -> Amount {
            self.balance.unwrap_or(Amount::zero()) }

        /// Ratio between share of liquidity provided and amount of reward
        /// Should be <= 1 to make sure rewards budget is sufficient.
        pub fn ratio (&self) -> StdResult<Ratio> {
            match self.load(POOL_RATIO)? {
                Some(ratio) => Ok(ratio),
                None        => error!("missing reward ratio") } }

        /// For how many blocks does the user need to have provided liquidity
        /// in order to be eligible for rewards
        pub fn threshold (&self) -> StdResult<Time> {
            match self.load(POOL_THRESHOLD)? {
                Some(threshold) => Ok(threshold),
                None            => error!("missing lock threshold") } }

        /// For how many blocks does the user need to wait
        /// after claiming rewards before being able to claim them again
        pub fn cooldown (&self) -> StdResult<Time> {
            match self.load(POOL_COOLDOWN)? {
                Some(cooldown) => Ok(cooldown),
                None           => error!("missing claim cooldown") } } }

    Writable {

        pub fn configure_created (&mut self, time: &Time) -> StdResult<&mut Self> {
            self.save(POOL_CREATED, time) }

        pub fn configure_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
            self.save(POOL_RATIO, ratio) }

        pub fn configure_threshold (&mut self, threshold: &Time) -> StdResult<&mut Self> {
            self.save(POOL_THRESHOLD, threshold) }

        pub fn configure_cooldown (&mut self, cooldown: &Time) -> StdResult<&mut Self> {
            self.save(POOL_COOLDOWN, cooldown) }

        pub fn increment_claimed (&mut self, reward: Amount) -> StdResult<&mut Self> {
            self.save(POOL_CLAIMED, self.claimed()? + reward) }

        /// Every time the amount of tokens locked in the pool is updated,
        /// the pool's lifetime liquidity is tallied and and the timestamp is updated.
        /// This is the only user-triggered input to the pool.
        pub fn update_locked (&mut self, balance: Amount) -> StdResult<&mut Self> {
            let lifetime = self.lifetime()?;
            let now      = self.now()?;

            #[cfg(feature="pool_liquidity_ratio")] {
                let liquid = self.liquid()?;
                self.save(POOL_LIQUID, liquid)?;
            }

            self.save(POOL_LIFETIME,  lifetime)?
                .save(POOL_TIMESTAMP, now)?
                .save(POOL_LOCKED,    balance) } } );

stateful!(User (pool.storage):

    Readonly {

        // time-related getters --------------------------------------------------------------------

        /// Time of last lock or unlock
        pub fn timestamp (&self) -> StdResult<Option<Time>> {
            Ok(self.load_ns(USER_TIMESTAMP, self.address.as_slice())?) }

        /// Time that progresses always. Used to increment existence.
        pub fn elapsed (&self) -> StdResult<Time> {
            let now = self.pool.now()?;
            if let Ok(Some(timestamp)) = self.timestamp() {
                if now < timestamp { // prevent replay
                    return error!("no data")
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
            let now = self.pool.now()?;
            if let Ok(Some(timestamp)) = self.timestamp() {
                if now < timestamp { // prevent replay
                    return error!("no data")
                } else if self.locked()? > Amount::zero() {
                    Ok(now - timestamp)
                } else {
                    Ok(0 as Time)
                }
            } else {
                Ok(0 as Time)
            }
        }

        /// Up-to-date time for which the user has existed
        pub fn existed (&self) -> StdResult<Time> {
            Ok(self.last_existed()? + self.elapsed()?) }

        /// Load last value of user existence
        fn last_existed (&self) -> StdResult<Time> {
            Ok(self.load_ns(USER_EXISTED, self.address.as_slice())?
                .unwrap_or(0 as Time)) }

        /// Up-to-date time for which the user has provided liquidity
        pub fn present (&self) -> StdResult<Time> {
            Ok(self.last_present()? + self.elapsed_present()?) }

        /// Load last value of user present
        fn last_present (&self) -> StdResult<Time> {
            Ok(self.load_ns(USER_PRESENT, self.address.as_slice())?
                .unwrap_or(0 as Time)) }

        pub fn cooldown (&self) -> StdResult<Time> {
            Ok(Time::saturating_sub(self.last_cooldown()?, self.elapsed()?)) }

        fn last_cooldown (&self) -> StdResult<Time> {
            Ok(self.load_ns(USER_COOLDOWN, self.address.as_slice())?
                .unwrap_or(self.pool.cooldown()?)) }

        // lp-related getters ----------------------------------------------------------------------

        pub fn lifetime (&self) -> StdResult<Volume> {
            let existed = self.existed()?;
            Ok(if existed > 0u64 {
                let locked = self.locked()?;

                #[cfg(feature="user_liquidity_ratio")]
                let locked = locked.multiply_ratio(self.present()?, existed);

                tally(self.last_lifetime()?, self.elapsed_present()?, locked)?
            } else {
                Volume::zero()
            })
        }

        fn last_lifetime (&self) -> StdResult<Volume> {
            Ok(self.load_ns(USER_LIFETIME, self.address.as_slice())?.unwrap_or(Volume::zero())) }

        pub fn locked (&self) -> StdResult<Amount> {
            Ok(self.load_ns(USER_LOCKED, self.address.as_slice())?.unwrap_or(Amount::zero())) }

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
            let pool    = self.pool.lifetime()?;
            let existed = self.pool.existed()?;
            Ok(if pool > Volume::zero() && existed > 0u64 {
                let share = Volume::from(basis);

                let share = share.multiply_ratio(self.lifetime()?, pool)?;

                #[cfg(feature="pool_liquidity_ratio")]
                let share = share.multiply_ratio(self.pool.liquid()?, existed)?;

                share
            } else {
                0.into()
            })
        }

        pub fn earned (&self) -> StdResult<Amount> {
            let budget = self.pool.budget()?;
            let ratio = self.pool.ratio()?;
            Ok(self.share(budget.u128())?
                .multiply_ratio(ratio.0, ratio.1)?
                .low_u128().into()) }

        pub fn claimed (&self) -> StdResult<Amount> {
            Ok(self.load_ns(USER_CLAIMED, self.address.as_slice())?.unwrap_or(Amount::zero())) }

        pub fn claimable (&self) -> StdResult<Amount> {
            // you must lock for this long to claim
            if self.present()? < self.pool.threshold()? {
                return Ok(Amount::zero()) }

            let earned = self.earned()?;
            if earned == Amount::zero() {
                return Ok(Amount::zero()) }

            let claimed = self.claimed()?;
            if earned <= claimed {
                return Ok(Amount::zero()) }

            if let Some(balance) = self.pool.balance {
                let claimable = (earned - claimed)?;
                if claimable > balance {
                    return Ok(balance)
                } else {
                    return Ok(claimable)
                }
            } else {
                return Ok(Amount::zero())
            }
        }

    }

    Writable {

        // time-related mutations ------------------------------------------------------------------

        fn reset_cooldown (&mut self) -> StdResult<&mut Self> {
            let address = self.address.clone();
            self.save_ns(USER_COOLDOWN, address.as_slice(), self.pool.cooldown()?) }

        // lp-related mutations -------------------------------------------------------------------

        fn update (&mut self, user_balance: Amount, pool_balance: Amount) -> StdResult<()> {
            // Prevent replay
            let now = self.pool.now()?;
            if let Some(timestamp) = self.timestamp()? {
                if timestamp > now {
                    return error!("no data") } }
            // These rolling values will be comitted to storage
            let lifetime = self.lifetime()?;
            let existed  = self.existed()?;
            let present  = self.present()?;
            let cooldown = self.cooldown()?;
            // Update the user's record
            let address = self.address.clone();
            self// Always increment existence
                .save_ns(USER_EXISTED,   address.as_slice(), existed)?
                // If already providing liquidity, increment presence...
                .save_ns(USER_PRESENT,   address.as_slice(), present)?
                // Store the user's lifetime liquidity so far - diminished by absence
                .save_ns(USER_LIFETIME,  address.as_slice(), lifetime)?
                // ...and decrement cooldown
                .save_ns(USER_COOLDOWN,  address.as_slice(), cooldown)?
                // Set user's time of last update to now
                .save_ns(USER_TIMESTAMP, address.as_slice(), now)?
                // Update amount locked
                .save_ns(USER_LOCKED,    address.as_slice(), user_balance)?
                // Update total amount locked in pool
                .pool.update_locked(pool_balance)?;
            Ok(()) }

        pub fn lock_tokens (&mut self, increment: Amount) -> StdResult<Amount> {
            let locked = self.locked()?;
            self.update(
                locked + increment,
                self.pool.locked()? + increment.into())?;
            // Return the amount to be transferred from the user to the contract
            Ok(increment) }

        pub fn retrieve_tokens (&mut self, decrement: Amount) -> StdResult<Amount> {
            // Must have enough locked to retrieve
            let locked = self.locked()?;
            if locked < decrement {
                return error!(format!("not enough locked ({} < {})", locked, decrement)) }
            self.update(
                (self.locked()? - decrement)?,
                (self.pool.locked()? - decrement.into())?)?;
            // Return the amount to be transferred back to the user
            Ok(decrement) }

        // reward-related mutations ----------------------------------------------------------------

        fn increment_claimed (&mut self, reward: Amount) -> StdResult<&mut Self> {
            let address = self.address.clone();
            self.save_ns(USER_CLAIMED, address.as_slice(), self.claimed()? + reward) }

        pub fn claim_reward (&mut self) -> StdResult<Amount> {
            // Age must be above threshold to claim
            enforce_cooldown(self.present()?, self.pool.threshold()?)?;

            // User must wait between claims
            enforce_cooldown(0, self.cooldown()?)?;

            // See if there is some unclaimed reward amount:
            let claimable = self.claimable()?;
            if claimable == Amount::zero() {
                return error!(
                    "You've already received as much as your share of the reward pool allows. \
                    Keep your liquidity tokens locked and wait for more rewards to be vested, \
                    and/or lock more liquidity provision tokens to grow your share.") }

            // Update the user timestamp, and the other things that may be synced to it
            // Sacrifices gas cost for avoidance of hidden dependencies
            self.update(self.locked()?, self.pool.locked()?)?;

            // Reset the user cooldown, though
            self.reset_cooldown()?;

            // And keep track of how much they've claimed
            self.increment_claimed(claimable)?;
            self.pool.increment_claimed(claimable)?;

            // Return the amount to be sent to the user
            Ok(claimable)
        }
    }
);

fn enforce_cooldown (elapsed: Time, cooldown: Time) -> StdResult<()> {
    if elapsed >= cooldown {
        Ok(())
    } else {
        error!(format!("lock tokens for {} more blocks to be eligible", cooldown - elapsed))
    }
}
