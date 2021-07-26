use crate::rewards_math::*;
use fadroma::scrt::{
    cosmwasm_std::{StdError, CanonicalAddr},
    storage::*
};

macro_rules! error { ($info:expr) => { Err(StdError::generic_err($info)) }; }

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current balance)
const POOL_LIFETIME:  &[u8] = b"/pool/lifetime";
/// How much liquidity is there in the whole pool right now
const POOL_LOCKED:    &[u8] = b"/pool/balance";
/// When was liquidity last updated
const POOL_TIMESTAMP: &[u8] = b"/pool/updated";
/// Rewards claimed by everyone so far
const POOL_CLAIMED:   &[u8] = b"/pool/claimed";
/// Ratio of liquidity provided to rewards received
const POOL_RATIO:     &[u8] = b"/pool/ratio";
/// Initial lock before first claim
const POOL_THRESHOLD: &[u8] = b"/pool/threshold";
/// Time before each claim
const POOL_COOLDOWN:  &[u8] = b"/pool/cooldown";

/// How much liquidity has each user provided since they first appeared;
/// incremented in intervals of (blocks since last update * current balance)
const USER_LIFETIME:   &[u8] = b"/user/lifetime/";
/// How much liquidity does each user currently provide
const USER_LOCKED:     &[u8] = b"/user/current/";
/// When did each user's liquidity amount last change
const USER_TIMESTAMP:  &[u8] = b"/user/updated/";
/// How much rewards has each user claimed so far
const USER_CLAIMED:    &[u8] = b"/user/claimed/";
/// For how many units of time has this user provided liquidity
const USER_AGE:        &[u8] = b"/user/age/";
/// For how many units of time has this user provided liquidity
const USER_COOLDOWN:   &[u8] = b"/user/cooldown/";

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

        pub fn configure_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
            self.save(POOL_RATIO, ratio) }

        pub fn configure_threshold (&mut self, threshold: &Time) -> StdResult<&mut Self> {
            self.save(POOL_THRESHOLD, threshold) }

        pub fn configure_cooldown (&mut self, cooldown: &Time) -> StdResult<&mut Self> {
            self.save(POOL_COOLDOWN, cooldown) }

        fn update_lifetime (&mut self) -> StdResult<&mut Self> {
            self.save(POOL_LIFETIME, self.lifetime()?) }

        fn update_timestamp (&mut self) -> StdResult<&mut Self> {
            self.save(POOL_TIMESTAMP, self.now()?) }

        fn increment_claimed (&mut self, reward: Amount) -> StdResult<&mut Self> {
            self.save(POOL_CLAIMED, self.claimed()? + reward) }

        /// Every time the amount of tokens locked in the pool is updated,
        /// the pool's lifetime liquidity is tallied and and the timestamp is updated.
        /// This is the only user-triggered input to the pool.
        pub fn update_locked (&mut self, balance: Amount) -> StdResult<&mut Self> {
            self.update_lifetime()?
                .update_timestamp()?
                .save(POOL_LOCKED, balance) } } );

stateful!(User (pool.storage):

    Readonly {

        // time-related getters --------------------------------------------------------------------

        /// Time of last lock or unlock
        pub fn timestamp (&self) -> StdResult<Option<Time>> {
            Ok(self.load_ns(USER_TIMESTAMP, self.address.as_slice())?) }

        /// Time doesn't progress for a user unless they have some tokens locked
        pub fn elapsed (&self) -> StdResult<Time> {
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

        /// Up-to-date time for which the user has provided liquidity
        pub fn age (&self) -> StdResult<Time> {
            Ok(self.last_age()? + self.elapsed()?) }

        /// Load last value of user age
        fn last_age (&self) -> StdResult<Time> {
            Ok(self.load_ns(USER_AGE, self.address.as_slice())?
                .unwrap_or(0 as Time)) }

        pub fn cooldown (&self) -> StdResult<Time> {
            Ok(Time::saturating_sub(self.last_cooldown()?, self.elapsed()?)) }

        fn last_cooldown (&self) -> StdResult<Time> {
            Ok(self.load_ns(USER_COOLDOWN, self.address.as_slice())?
                .unwrap_or(self.pool.cooldown()?)) }

        // lp-related getters ----------------------------------------------------------------------

        pub fn lifetime (&self) -> StdResult<Volume> {
            tally(self.last_lifetime()?, self.elapsed()?, self.locked()?) }

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

        pub fn earned (&self) -> StdResult<Amount> {
            let pool = self.pool.lifetime()?;
            if pool == Volume::zero() {
                return Ok(Amount::zero()) }
            let ratio = self.pool.ratio()?;
            // compute the earned reward
            Ok(Volume::from(self.pool.budget()?)
                // as a portion of the reward budget
                .multiply_ratio(self.lifetime()?, pool)?
                // diminished by the optional global `ratio`
                .multiply_ratio(ratio.0, ratio.1)?       
                .low_u128().into()) }

        pub fn claimed (&self) -> StdResult<Amount> {
            Ok(self.load_ns(USER_CLAIMED, self.address.as_slice())?.unwrap_or(Amount::zero())) }

        pub fn claimable (&self) -> StdResult<Amount> {
            // you must lock for this long to claim
            if self.age()? < self.pool.threshold()? {
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

        // lp-related mutations -------------------------------------------------------------------

        fn update (&mut self, user_balance: Amount, pool_balance: Amount) -> StdResult<()> {
            // Prevent replay
            let now = self.pool.now()?;
            if let Some(timestamp) = self.timestamp()? {
                if timestamp > now {
                    return error!("no data") } }
            // These rolling values will be comitted to storage
            let lifetime = self.lifetime()?;
            let age      = self.age()?;
            let cooldown = self.cooldown()?;
            // Update the user's record
            let address = self.address.clone();
            self// Store the user's lifetime liquidity until now
                .save_ns(USER_LIFETIME,  address.as_slice(), lifetime)?  
                // If already providing liquidity, increments age...
                .save_ns(USER_AGE,       address.as_slice(), age)?
                // ...and decrements cooldown
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

        fn reset_cooldown (&mut self) -> StdResult<&mut Self> {
            let address = self.address.clone();
            self.save_ns(USER_COOLDOWN, address.as_slice(), self.pool.cooldown()?) }

        pub fn claim_reward (&mut self) -> StdResult<Amount> {
            // Age must be above threshold to claim
            enforce_cooldown(self.age()?, self.pool.threshold()?)?;

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
            self.update(self.locked()?, self.pool.locked()?);

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
