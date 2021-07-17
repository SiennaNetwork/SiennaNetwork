use crate::rewards_math::*;
use fadroma::scrt::{cosmwasm_std::{StdError, CanonicalAddr}, storage::traits2::*,};

macro_rules! error { ($info:expr) => { Err(StdError::generic_err($info)) }; }

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current balance)
const POOL_TALLIED:   &[u8] = b"pool_lifetime";
/// How much liquidity is there in the whole pool right now
const POOL_BALANCE:   &[u8] = b"pool_balance";
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
const USER_TALLIED: &[u8] = b"user_lifetime/";
/// How much liquidity does each user currently provide
const USER_BALANCE: &[u8] = b"user_current/";
/// When did each user's liquidity amount last change
const USER_UPDATED: &[u8] = b"user_updated/";
/// How much rewards has each user claimed so far
const USER_CLAIMED: &[u8] = b"user_claimed/";
/// For how many units of time has this user provided liquidity
const USER_EXISTED: &[u8] = b"user_existed/";

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
        /// Get the current time or fail
        pub fn now (&self) -> StdResult<Time> {
            self.now.ok_or(StdError::generic_err("current time not set"))
        }
        /// Get the current balance, up-to-date lifetime liquidity, and time of last update
        pub fn status (&self) -> StdResult<(Amount, Volume, Time)> {
            if let Some(last_update) = self.load(POOL_UPDATED)? {
                if self.now()? >= last_update {
                    let balance  = self.load(POOL_BALANCE)? as Option<Amount>;
                    let lifetime = self.load(POOL_TALLIED)? as Option<Volume>;
                    if let (Some(balance), Some(lifetime)) = (balance, lifetime) {
                        let elapsed = self.now()? - last_update;
                        Ok((balance, tally(lifetime, elapsed, balance.into())?, last_update))
                    } else { error!("missing BALANCE or TALLIED") }
                } else { error!("can't query before last update") }
            } else { Ok((Amount::zero(), Volume::zero(), 0 as Time)) }
        }
        /// Amount of currently locked LP tokens in this pool
        pub fn balance (&self) -> StdResult<Amount> {
            Ok(self.load(POOL_BALANCE)?.unwrap_or(Amount::zero()))
        }
        /// Ratio between share of liquidity provided and amount of reward
        /// Should be <= 1 to make sure rewards budget is sufficient. 
        pub fn ratio (&self) -> StdResult<Ratio> {
            match self.load(POOL_RATIO)? {
                Some(ratio) => Ok(ratio),
                None        => error!("missing reward ratio")
            }
        }
        /// For how many blocks does the user need to have provided liquidity
        /// in order to be eligible for rewards
        pub fn threshold (&self) -> StdResult<Time> {
            match self.load(POOL_THRESHOLD)? {
                Some(ratio) => Ok(ratio),
                None        => error!("missing reward threshold")
            }
        }
        /// The full reward budget = rewards claimed + current balance of this contract in reward token
        pub fn budget (&self, balance: Amount) -> StdResult<Amount> {
            Ok(self.load(POOL_CLAIMED)?.unwrap_or(Amount::zero()) + balance)
        }
        /// The total liquidity ever contained in this pool.
        pub fn lifetime (&self) -> StdResult<Volume> {
            let balance = self.balance()?;
            let previous:     Option<Volume>    = self.load(POOL_TALLIED)?;
            let last_updated: Option<Time> = self.load(POOL_UPDATED)?;
            if let (Some(previous), Some(last_updated)) = (previous, last_updated) {
                tally(previous, self.now()? - last_updated, balance.into())
            } else {
                Ok(Volume::zero())
            }
        }
    }
    Writable {
        pub fn save_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
            self.save(POOL_RATIO, ratio)
        }
        pub fn save_threshold (&mut self, threshold: &Time) -> StdResult<&mut Self> {
            self.save(POOL_THRESHOLD, threshold)
        }
        pub fn update_balance (&mut self, new_balance: Amount) -> StdResult<&mut Self> {
            let tallied = self.lifetime()?;
            let now     = self.now()?;
            self.save(POOL_TALLIED, tallied)?
                .save(POOL_UPDATED, now)?
                .save(POOL_BALANCE, new_balance)
        }
    }
);

stateful!(User (pool.storage):
    Readonly {
        pub fn address (&self) -> &CanonicalAddr {
            &self.address
        }
        pub fn now (&self) -> StdResult<Time> {
            self.pool.now()
        }
        pub fn updated (&self) -> StdResult<Time> {
            match self.load_ns(USER_UPDATED, self.address().as_slice())? {
                Some(x) => Ok(x),
                None    => error!("missing USER_UPDATED")
            }
        }
        pub fn existed (&self) -> StdResult<Time> {
            Ok(self.load_ns(USER_EXISTED, self.address().as_slice())?
                .unwrap_or(0 as Time))
        }
        pub fn elapsed (&self) -> StdResult<Time> {
            let now = self.now()?;
            match self.updated() {
                Ok(updated) => if now >= updated {
                    Ok(now - updated)
                } else {
                    error!(format!("tried to query at {}, last update is {}", &now, &updated))
                },
                // default to 0
                Err(_) => Ok(0 as Time)
            }
        }
        pub fn tallied (&self) -> StdResult<Volume> {
            Ok(self.load_ns(USER_TALLIED, self.address().as_slice())?.unwrap_or(Volume::zero()))
        }
        pub fn balance (&self) -> StdResult<Amount> {
            Ok(self.load_ns(USER_BALANCE, self.address().as_slice())?.unwrap_or(Amount::zero()))
        }
        pub fn claimed (&self) -> StdResult<Amount> {
            Ok(self.load_ns(USER_CLAIMED, self.address().as_slice())?.unwrap_or(Amount::zero()))
        }
        pub fn age (&self) -> StdResult<Time> {
            let existed = self.existed()?;
            let balance = self.balance()?;
            if balance > Amount::zero() {
                // if user is currently providing liquidity,
                // the time since last update gets added to the age
                Ok(existed + self.elapsed()?)
            } else {
                Ok(existed)
            }
        }
        pub fn lifetime (&self) -> StdResult<Volume> {
            tally(self.tallied()?, self.elapsed()?, self.balance()?)
        }
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
        pub fn reward (&self, balance: Amount) -> StdResult<(Amount, Amount, Amount)> {
            let age       = self.age()?;
            let threshold = self.pool.threshold()?;
            let pool      = self.pool.lifetime()?;
            if age >= threshold && pool > Volume::zero() {
                let user     = self.lifetime()?;
                let budget   = self.pool.budget(balance)?;
                let ratio    = self.pool.ratio()?;
                let unlocked = Volume::from(budget)
                    .multiply_ratio(user, pool)?
                    .multiply_ratio(ratio.0, ratio.1)?
                    .low_u128().into();
                let claimed  = self.claimed()?;
                if unlocked > claimed {
                    Ok((unlocked, claimed, (unlocked - claimed)?))
                } else {
                    Ok((unlocked, claimed, Amount::zero()))
                }
            } else {
                Ok((Amount::zero(), Amount::zero(), Amount::zero()))
            }
        }
    }
    Writable {
        fn update (&mut self) -> StdResult<&mut Self> {
            self.save_ns(USER_UPDATED, self.address.as_slice(), self.now()?)
        }

        fn update_lifetime (&mut self) -> StdResult<&mut Self> {
            self.save_ns(USER_TALLIED, self.address.as_slice(), self.lifetime()?)
        }

        fn update_age (&mut self) -> StdResult<&mut Self> {
            self.save_ns(USER_EXISTED, self.address.as_slice(), self.age()?)
        }

        fn update_balance (&mut self, balance: Amount) -> StdResult<&mut Self> {
            self.save_ns(USER_EXISTED, self.address.as_slice(), self.age()?)
        }

        pub fn lock_tokens (&mut self, increment: Amount) -> StdResult<Amount> {

            self.update()?          // Set user's time of last update to now
                .update_lifetime()? // Store the user's lifetime liquidity until now
                .update_age()?      // If already providing liquidity, increases age
                .update_balance(self.balance()? + increment)? // Increment balance of user
                .pool.update_balance(self.pool.balance()? + increment.into())?; // Update pool

            // Return the amount to be transferred from the user to the contract
            Ok(increment)

        }

        pub fn retrieve_tokens (&mut self, decrement: Amount) -> StdResult<Amount> {

            // Must have enough balance to retrieve
            let balance = self.balance()?;
            if balance < decrement {
                return error!(format!("not enough balance ({} < {})", balance, decrement))
            }

            self.update_lifetime()? // Save the user's lifetime liquidity so far
                .update_age()?      // If currently providing liquidity, increases age
                .update()?          // Set the user's time of last update to now
                .update_balance((balance - decrement)?)? // Decrement balance of user
                .pool.update_balance((self.pool.balance()? - decrement.into())?)?; // Update pool

            // Return the amount to be transferred back to the user
            Ok(decrement)

        }

        pub fn claim_reward (&mut self, balance: Amount) -> StdResult<Amount> {

            // Age must be above the threshold to claim
            let age       = self.age()?;
            let threshold = self.pool.threshold()?;
            if age < threshold {
                return error!(format!("lock tokens for {} more blocks to be eligible", threshold - age))
            }

            let (unlocked, _claimed, claimable) = self.reward(balance)?;
            if claimable > Amount::zero() {
                // If there is some new reward amount to claim:
                let address = self.address.clone();
                self.save_ns(USER_CLAIMED, address.as_slice(), &unlocked)?;
                Ok(claimable)
            } else if unlocked > Amount::zero() {
                // If this user has claimed all its rewards so far:
                error!("already claimed")
            } else {
                // If this user never had any rewards to claim:
                error!("nothing to claim")
            }

        }
    }
);
