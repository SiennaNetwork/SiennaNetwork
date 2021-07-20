use crate::rewards_math::*;
use fadroma::scrt::{cosmwasm_std::{StdError, CanonicalAddr}, storage::traits2::*,};

macro_rules! error { ($info:expr) => { Err(StdError::generic_err($info)) }; }

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current balance)
const POOL_LIFETIME:  &[u8] = b"pool_lifetime";
/// How much liquidity is there in the whole pool right now
const POOL_LOCKED:    &[u8] = b"pool_balance";
/// When was liquidity last updated
const POOL_UPDATED:   &[u8] = b"pool_updated";
/// Rewards claimed by everyone so far
const POOL_CLAIMED:   &[u8] = b"pool_claimed";
/// Ratio of liquidity provided to rewards received
const POOL_RATIO:     &[u8] = b"pool_ratio";
/// Ratio of liquidity provided to rewards received
const POOL_THRESHOLD: &[u8] = b"pool_threshold";

/// How much liquidity has each user provided since they first appeared;
/// incremented in intervals of (blocks since last update * current balance)
const USER_LIFETIME:  &[u8] = b"user_lifetime/";
/// How much liquidity does each user currently provide
const USER_LOCKED:    &[u8] = b"user_current/";
/// When did each user's liquidity amount last change
const USER_UPDATED:   &[u8] = b"user_updated/";
/// How much rewards has each user claimed so far
const USER_CLAIMED:   &[u8] = b"user_claimed/";
/// For how many units of time has this user provided liquidity
const USER_EXISTED:   &[u8] = b"user_existed/";

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

macro_rules! stateful {
    (
        $Obj:ident ($($storage:tt)+): /*{ $($accessors:tt)* } no traits no accessors */
        $Readonly:ident { $($readonlies:tt)* }
        $Writable:ident { $($writables:tt)* }
    ) => {
        impl<S: ReadonlyStorage> $Readonly<S> for $Obj<&S> {
            fn storage (&self) -> &S { &self.$($storage)+ }
        }
        impl<S: ReadonlyStorage> $Readonly<S> for $Obj<&mut S> {
            fn storage (&self) -> &S { &self.$($storage)+ }
        }
        impl<S: Storage> $Writable<S> for $Obj<&mut S> {
            fn storage_mut (&mut self) -> &mut S { &mut self.$($storage)+ }
        }
        impl<S: ReadonlyStorage> $Obj<&S> {
            $($readonlies)*
        }
        impl<S: Storage> $Obj<&mut S> {
            $($readonlies)*
            $($writables)*
        }
    };
}

stateful!(Pool (storage):

    Readonly {

        /// The total liquidity ever contained in this pool.
        pub fn lifetime (&self) -> StdResult<Volume> {
            tally(self.last_lifetime()?, self.elapsed()?, self.locked()?) }

        /// Snapshot of total liquidity at moment of last update.
        pub fn last_lifetime (&self) -> StdResult<Volume> {
            Ok(self.load(POOL_LIFETIME)?.unwrap_or(Volume::zero())) }

        /// Get the time since last update (0 if no last update)
        pub fn elapsed (&self) -> StdResult<Time> {
            Ok(self.now()? - self.last_update()?) }

        /// Get the current time or fail
        pub fn now (&self) -> StdResult<Time> {
            self.now.ok_or(StdError::generic_err("current time not set")) }

        /// Load the last update timestamp or default to current time
        /// (this has the useful property of keeping `elapsed` zero for strangers)
        pub fn last_update (&self) -> StdResult<Time> {
            match self.load(POOL_UPDATED)? {
                Some(time) => Ok(time),
                None       => Ok(self.now()?) } }

        /// Amount of currently locked LP tokens in this pool
        pub fn locked (&self) -> StdResult<Amount> {
            Ok(self.load(POOL_LOCKED)?.unwrap_or(Amount::zero())) }

        /// The full reward budget = rewards claimed + current balance of this contract in reward token
        pub fn budget (&self) -> StdResult<Amount> {
            Ok(self.load(POOL_CLAIMED)?.unwrap_or(Amount::zero()) + self.balance()) }

        /// Current balance in reward token, or zero.
        fn balance (&self) -> Amount {
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
                Some(ratio) => Ok(ratio),
                None        => error!("missing reward threshold") } } }

    Writable {

        pub fn configure_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
            self.save(POOL_RATIO, ratio) }

        pub fn configure_threshold (&mut self, threshold: &Time) -> StdResult<&mut Self> {
            self.save(POOL_THRESHOLD, threshold) }

        fn update_lifetime (&mut self) -> StdResult<&mut Self> {
            self.save(POOL_LIFETIME, self.lifetime()?) }

        fn update_timestamp (&mut self) -> StdResult<&mut Self> {
            self.save(POOL_UPDATED, self.now()?) }

        /// Every time the amount of tokens locked in the pool is updated,
        /// the pool's lifetime liquidity is tallied and and the timestamp is updated.
        /// This is the only user-triggered input to the pool.
        pub fn update_locked (&mut self, balance: Amount) -> StdResult<&mut Self> {
            self.update_lifetime()?
                .update_timestamp()?
                .save(POOL_LOCKED, balance) } } );

stateful!(User (pool.storage):

    Readonly {

        pub fn age (&self) -> StdResult<Time> {
            let existed = self.last_age()?;
            let balance = self.locked()?;
            // if user is currently providing liquidity,
            if balance > Amount::zero() {
                // the time since last update is added to the stored age to get the up-to-date age
                Ok(existed + self.elapsed()?) }
            else { Ok(existed) } }

        pub fn last_age (&self) -> StdResult<Time> {
            Ok(self.load_ns(USER_EXISTED, self.address.as_slice())?
                .unwrap_or(0 as Time)) }

        pub fn lifetime (&self) -> StdResult<Volume> {
            tally(self.last_lifetime()?, self.elapsed()?, self.locked()?) }

        pub fn last_lifetime (&self) -> StdResult<Volume> {
            Ok(self.load_ns(USER_LIFETIME, self.address.as_slice())?.unwrap_or(Volume::zero())) }

        pub fn locked (&self) -> StdResult<Amount> {
            Ok(self.load_ns(USER_LOCKED, self.address.as_slice())?.unwrap_or(Amount::zero())) }

        pub fn elapsed (&self) -> StdResult<Time> {
            let now = self.pool.now()?;
            match self.last_update() {
                Ok(updated) => {
                    if now < updated {
                        return error!(format!("tried to query at {}, last update is {}", &now, &updated)) }
                    Ok(now - updated) },
                Err(_) => Ok(0 as Time) } }

        pub fn last_update (&self) -> StdResult<Time> {
            match self.load_ns(USER_UPDATED, self.address.as_slice())? {
                Some(x) => Ok(x),
                None    => Ok(self.pool.now()?) } }

        /// After first locking LP tokens, users must reach a configurable age threshold,
        /// i.e. keep LP tokens locked for at least X blocks. During that time, their portion of
        /// the total liquidity ever provided increases.
        ///
        /// The total reward for an user with an age under the threshold is zero.
        ///
        /// The total reward for a user with an age above the threshold is
        /// (claimed_rewards + budget) * user_lifetime_liquidity / pool_lifetime_liquidity
        ///
        /// Since a user's total reward can diminish, it may happen that the amount claimed
        /// by a user so far is larger than the current total reward for that user.
        /// In that case the user's claimable amount remains zero until they unlock more
        /// rewards than they've already claimed.
        /// 
        /// Since a user's total reward can diminish, it may happen that the amount remaining
        /// in the pool after a user has claimed is insufficient to pay out the next user's reward.
        /// In that case, https://google.github.io/filament/webgl/suzanne.html
        pub fn reward (&self) -> StdResult<(Amount, Amount, Amount)> {
            // new empty pool - rewards for everyone are zero
            let pool = self.pool.lifetime()?;
            if pool == Volume::zero() {
                return Ok((Amount::zero(), Amount::zero(), Amount::zero())) }
            // you must lock for this long to claim
            let age       = self.age()?;
            let threshold = self.pool.threshold()?;
            if age < threshold {
                return Ok((Amount::zero(), Amount::zero(), Amount::zero())) }
            // compute the unlocked reward
            // as a portion of the total reward `balance`
            // optionally diminished by the global `ratio`
            let ratio = self.pool.ratio()?;
            let unlocked = Volume::from(self.pool.budget()?)
                .multiply_ratio(self.lifetime()?, pool)?
                .multiply_ratio(ratio.0, ratio.1)?
                .low_u128().into();
            let claimed = self.claimed()?;
            // if already claimed this much or more...
            // stake more tokens next time?
            if unlocked <= claimed {
                return Ok((unlocked, claimed, Amount::zero())) }
            // if there is something more to claim, let em know
            Ok((unlocked, claimed, (unlocked - claimed)?)) }

        pub fn claimed (&self) -> StdResult<Amount> {
            Ok(self.load_ns(USER_CLAIMED, self.address.as_slice())?.unwrap_or(Amount::zero())) } }

    Writable {

        fn update_timestamp (&mut self) -> StdResult<&mut Self> {
            let address = self.address.clone();
            self.save_ns(USER_UPDATED,  address.as_slice(), self.pool.now()?) }

        fn update_lifetime (&mut self) -> StdResult<&mut Self> {
            let address = self.address.clone();
            self.save_ns(USER_LIFETIME, address.as_slice(), self.lifetime()?) }

        fn update_age (&mut self) -> StdResult<&mut Self> {
            let address = self.address.clone();
            self.save_ns(USER_EXISTED,  address.as_slice(), self.age()?) }

        fn update_locked (&mut self, locked: Amount) -> StdResult<&mut Self> {
            let address = self.address.clone();
            self.save_ns(USER_LOCKED,   address.as_slice(), locked) }

        pub fn lock_tokens (&mut self, increment: Amount) -> StdResult<Amount> {
            let new_user_balance = self.locked()? + increment;
            let new_pool_balance = self.pool.locked()? + increment.into();
            self.update_timestamp()? // Set user's time of last update to now
                .update_lifetime()?  // Store the user's lifetime liquidity until now
                .update_age()?       // If already providing liquidity, increases age
                .update_locked(new_user_balance)? // Increment locked of user
                .pool.update_locked(new_pool_balance)?; // Update pool
            // Return the amount to be transferred from the user to the contract
            Ok(increment) }

        pub fn retrieve_tokens (&mut self, decrement: Amount) -> StdResult<Amount> {
            // Must have enough locked to retrieve
            let locked = self.locked()?;
            if locked < decrement {
                return error!(format!("not enough locked ({} < {})", locked, decrement)) }
            let new_pool_balance = (self.pool.locked()? - decrement.into())?;
            self.update_lifetime()?  // Save the user's lifetime liquidity so far
                .update_age()?       // If currently providing liquidity, increases age
                .update_timestamp()? // Set the user's time of last update to now
                .update_locked((locked - decrement)?)? // Decrement locked of user
                .pool.update_locked(new_pool_balance)?; // Update pool
            // Return the amount to be transferred back to the user
            Ok(decrement) }

        pub fn claim_reward (&mut self) -> StdResult<Amount> {
            // Age must be above the threshold to claim
            let age       = self.age()?;
            let threshold = self.pool.threshold()?;
            if age < threshold {
                return error!(format!("lock tokens for {} more blocks to be eligible", threshold - age)) }
            let (unlocked, _claimed, claimable) = self.reward()?;
            if claimable > Amount::zero() {
                // If there is some new reward amount to claim:
                let address = self.address.clone();
                self.save_ns(USER_CLAIMED, address.as_slice(), &unlocked)?;
                Ok(claimable) }
            else if unlocked > Amount::zero() {
                // If this user has claimed all its rewards so far:
                error!("already claimed") }
            else {
                // If this user never had any rewards to claim:
                error!("nothing to claim") } } } );
