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
    now:         Option<Monotonic>,
    address:     Option<CanonicalAddr>
}

/// User account
pub struct User <S> {
    pub storage: S,
    now:         Option<Monotonic>,
    address:     Option<CanonicalAddr>
}

impl <S> Pool<S> {
    /// Create a new pool with a storage handle
    pub fn new (storage: S) -> Self {
        Self { storage, now: None, address: None }
    }
    /// Add the current time to the pool for operations that need it
    pub fn at (self, now: Monotonic) -> Self {
        Self { storage: self.storage, address: self.address, now: Some(now) }
    }
    /// Get an individual user from the pool
    pub fn user (self, address: CanonicalAddr) -> Self {
        Pool { storage: self.storage, address: Some(address), now: self.now }
    }
}


macro_rules! readonly {
    (
        $Obj:ident { $($accessors:tt)* }
        $ObjReadonly:ident { $($readonlies:tt)* }
        $ObjWritable:ident { $($writables:tt)* }
    ) => {
        impl<S: ReadonlyStorage> Readonly<S> for $Obj<&S> {
            fn storage (&self) -> &S { &self.storage }
        }
        impl<S: ReadonlyStorage> Readonly<S> for $Obj<&mut S> {
            fn storage (&self) -> &S { &self.storage }
        }
        pub trait $ObjReadonly<S: ReadonlyStorage>: Readonly<S> { // now its with a trait
            $($readonlies)*
        }
        impl<S: ReadonlyStorage> $ObjReadonly<S> for $Obj<&S> {
            $($accessors)*
        }
        impl<S: ReadonlyStorage> $ObjReadonly<S> for $Obj<&mut S> {
            $($accessors)*
        }
        impl<S: Storage> Writable<S> for $Obj<&mut S> {
            fn storage_mut (&mut self) -> &mut S { &mut self.storage }
        }
        impl<S: Storage> $Obj<&mut S> {
            $($writables)*
        }
    };
}

readonly!(Pool {

    /// Get the current time or fail
    fn now (&self) -> StdResult<Monotonic> {
        self.now.ok_or(StdError::generic_err("current time not set"))
    }
    /// Get the selected user or fail
    fn address (&self) -> StdResult<CanonicalAddr> {
        match &self.address {
            Some(address) => Ok(address.clone()),
            None => Err(StdError::generic_err("address not set"))
        }
    }

} PoolReadonly { // pool readonly operations

    fn now (&self) -> StdResult<Monotonic>; 
    fn address (&self) -> StdResult<CanonicalAddr>; 

    fn pool_status (&self) -> StdResult<(Amount, Volume, Monotonic)> {
        if let Some(last_update) = self.load(POOL_UPDATED)? {
            if self.now()? >= last_update {
                let balance  = self.load(POOL_BALANCE)? as Option<Amount>;
                let lifetime = self.load(POOL_TALLIED)? as Option<Volume>;
                if let (Some(balance), Some(lifetime)) = (balance, lifetime) {
                    let elapsed = self.now()? - last_update;
                    Ok((balance, tally(lifetime, elapsed, balance.into())?, last_update))
                } else { error!("missing BALANCE or TALLIED") }
            } else { error!("can't query before last update") }
        } else { Ok((Amount::zero(), Volume::zero(), 0 as Monotonic)) }
    }

    /// Amount of currently locked LP tokens in this pool
    fn pool_balance (&self) -> StdResult<Amount> {
        Ok(self.load(POOL_BALANCE)?.unwrap_or(Amount::zero()))
    }

    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1 to make sure rewards budget is sufficient. 
    fn pool_ratio (&self) -> StdResult<Ratio> {
        match self.load(POOL_RATIO)? {
            Some(ratio) => Ok(ratio),
            None        => error!("missing reward ratio")
        }
    }

    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    fn pool_threshold (&self) -> StdResult<Monotonic> {
        match self.load(POOL_THRESHOLD)? {
            Some(ratio) => Ok(ratio),
            None        => error!("missing reward threshold")
        }
    }

    /// The full reward budget = rewards claimed + current balance of this contract in reward token
    fn pool_budget (&self, balance: Amount) -> StdResult<Amount> {
        Ok(self.load(POOL_CLAIMED)?.unwrap_or(Amount::zero()) + balance)
    }

    /// The total liquidity ever contained in this pool.
    fn pool_lifetime (&self) -> StdResult<Volume> {
        let balance = self.pool_balance()?;
        let previous:     Option<Volume>    = self.load(POOL_TALLIED)?;
        let last_updated: Option<Monotonic> = self.load(POOL_UPDATED)?;
        if let (Some(previous), Some(last_updated)) = (previous, last_updated) {
            tally(previous, self.now()? - last_updated, balance.into())
        } else {
            Ok(Volume::zero())
        }
    }

    fn user_updated (&self) -> StdResult<Monotonic> {
        match self.load_ns(USER_UPDATED, self.address()?.as_slice())? {
            Some(x) => Ok(x),
            None    => error!("missing USER_UPDATED")
        }
    }

    fn user_existed (&self) -> StdResult<Monotonic> {
        Ok(self.load_ns(USER_EXISTED, self.address()?.as_slice())?
            .unwrap_or(0 as Monotonic))
    }

    fn user_elapsed (&self) -> StdResult<Monotonic> {
        let now = self.now()?;
        match self.user_updated() {
            Ok(updated) => if now >= updated {
                Ok(now - updated)
            } else {
                error!(format!("tried to query at {}, last update is {}", &now, &updated))
            },
            // default to 0
            Err(_) => Ok(0 as Monotonic)
        }
    }

    fn user_tallied (&self) -> StdResult<Volume> {
        Ok(self.load_ns(USER_TALLIED, self.address()?.as_slice())?
            .unwrap_or(Volume::zero()))
    }

    fn user_balance (&self) -> StdResult<Amount> {
        Ok(self.load_ns(USER_BALANCE, self.address()?.as_slice())?
            .unwrap_or(Amount::zero()))
    }

    fn user_claimed (&self) -> StdResult<Amount> {
        Ok(self.load_ns(USER_CLAIMED, self.address()?.as_slice())?
            .unwrap_or(Amount::zero()))
    }

    fn user_age (&self) -> StdResult<Monotonic> {
        let existed = self.user_existed()?;
        let balance = self.user_balance()?;
        if balance > Amount::zero() {
            // if user is currently providing liquidity,
            // the time since last update gets added to the age
            Ok(existed + self.user_elapsed()?)
        } else {
            Ok(existed)
        }
    }

    fn user_lifetime (&self) -> StdResult<Volume> {
        tally(self.user_tallied()?, self.user_elapsed()?, self.user_balance()?)
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
    /// In that case, 
    fn user_reward (&self, balance: Amount) -> StdResult<(Amount, Amount, Amount)> {
        let age       = self.user_age()?;
        let threshold = self.pool_threshold()?;
        let pool      = self.pool_lifetime()?;
        if age >= threshold && pool > Volume::zero() {
            let user     = self.user_lifetime()?;
            let budget   = self.pool_budget(balance)?;
            let ratio    = self.pool_ratio()?;
            let unlocked = Volume::from(budget)
                .multiply_ratio(user, pool)?
                .multiply_ratio(ratio.0, ratio.1)?
                .low_u128().into();
            let claimed  = self.user_claimed()?;
            if unlocked > claimed {
                Ok((unlocked, claimed, (unlocked - claimed)?))
            } else {
                Ok((unlocked, claimed, Amount::zero()))
            }
        } else {
            Ok((Amount::zero(), Amount::zero(), Amount::zero()))
        }
    }

} PoolWritable {

    pub fn pool_set_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
        self.save(POOL_RATIO, ratio)
    }

    pub fn pool_set_threshold (&mut self, threshold: &Monotonic) -> StdResult<&mut Self> {
        self.save(POOL_THRESHOLD, threshold)
    }

    pub fn pool_update (&mut self, new_balance: Amount) -> StdResult<&mut Self> {
        let tallied = self.pool_lifetime()?;
        let now     = self.now()?;
        self.save(POOL_TALLIED, tallied)?
            .save(POOL_UPDATED, now)?
            .save(POOL_BALANCE, new_balance)
    }

    pub fn user_lock (&mut self, increment: Amount) -> StdResult<Amount> {
        // Remember when the user was last updated, i.e. now
        self.save_ns(USER_UPDATED, self.address()?.as_slice(),
            self.now()?)?;

        // Save the user's lifetime liquidity so far
        self.save_ns(USER_TALLIED, self.address()?.as_slice(),
            self.user_lifetime()?)?;

        // If current balance is > 0, increment the user's age
        // with the time since the last update
        self.save_ns(USER_EXISTED, self.address()?.as_slice(),
            self.user_age()?)?;

        // Increment liquidity from user
        self.save_ns(USER_BALANCE, self.address()?.as_slice(),
            self.user_balance()? + increment)?;

        // Increment liquidity in pool
        self.pool_update(self.pool_balance()? + increment.into())?;

        // Return the amount to lock
        Ok(increment)
    }

    pub fn user_retrieve (&mut self, decrement: Amount) -> StdResult<Amount> {
        let balance = self.user_balance()?;

        // Must have enough balance to retrieve
        if balance < decrement {
            error!(format!("not enough balance ({} < {})", balance, decrement))
        } else {
            // Save the user's lifetime liquidity so far
            self.save_ns(USER_TALLIED, self.address()?.as_slice(),
                self.user_lifetime()?)?;

            // If current balance is > 0, increment the user's age
            // with the time since the last update
            self.save_ns(USER_EXISTED, self.address()?.as_slice(),
                self.user_age()?)?;

            // Remember when the user was last updated, i.e. now
            self.save_ns(USER_UPDATED, self.address()?.as_slice(),
            self.now()?)?;

            // Remove liquidity from user
            self.save_ns(USER_BALANCE, self.address()?.as_slice(),
                (balance - decrement)?)?;

            // Remove liquidity from pool
            self.pool_update((self.pool_balance()? - decrement.into())?)?;

            // Return the amount to return
            Ok(decrement)
        }
    }

    pub fn user_claim (&mut self, balance: Amount) -> StdResult<Amount> {
        let age       = self.user_age()?;
        let threshold = self.pool_threshold()?;

        // Age must be above the threshold to claim
        if age >= threshold {
            let (unlocked, _claimed, claimable) = self.user_reward(balance)?;
            if claimable > Amount::zero() {
                // If there is some new reward amount to claim:
                self.save_ns(USER_CLAIMED, self.address()?.as_slice(), &unlocked)?;
                Ok(claimable)
            } else if unlocked > Amount::zero() {
                // If this user has claimed all its rewards so far:
                error!("already claimed")
            } else {
                // If this user never had any rewards to claim:
                error!("nothing to claim")
            }
        } else {
            error!(format!("lock tokens for {} more blocks to be eligible", threshold - age))
        }
    }

});
