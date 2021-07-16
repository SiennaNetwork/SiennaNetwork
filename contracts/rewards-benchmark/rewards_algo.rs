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
}

/// User account
pub struct User <S> {
    pub storage: S,
    now:         Option<Monotonic>,
    address:     CanonicalAddr
}

impl <S> Pool<S> {
    /// Create a new pool with a storage handle
    pub fn new (storage: S) -> Self {
        Self { storage, now: None }
    }
    /// Add the current time to the pool for operations that need it
    pub fn at (self, now: Monotonic) -> Self {
        Self { storage: self.storage, now: Some(now) }
    }
    /// Get an individual user from the pool
    pub fn user (self, address: CanonicalAddr) -> User<S> {
        User { storage: self.storage, now: self.now, address }
    }
}

macro_rules! stateful {
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

stateful!(Pool {

    /// Get the current time or fail
    fn now (&self) -> StdResult<Monotonic> {
        self.now.ok_or(StdError::generic_err("current time not set"))
    }

} PoolReadonly { // pool stateful operations

    fn now (&self) -> StdResult<Monotonic>; 

    fn status (&self) -> StdResult<(Amount, Volume, Monotonic)> {
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
    fn balance (&self) -> StdResult<Amount> {
        Ok(self.load(POOL_BALANCE)?.unwrap_or(Amount::zero()))
    }

    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1 to make sure rewards budget is sufficient. 
    fn ratio (&self) -> StdResult<Ratio> {
        match self.load(POOL_RATIO)? {
            Some(ratio) => Ok(ratio),
            None        => error!("missing reward ratio")
        }
    }

    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    fn threshold (&self) -> StdResult<Monotonic> {
        match self.load(POOL_THRESHOLD)? {
            Some(ratio) => Ok(ratio),
            None        => error!("missing reward threshold")
        }
    }

    /// The full reward budget = rewards claimed + current balance of this contract in reward token
    fn budget (&self, balance: Amount) -> StdResult<Amount> {
        Ok(self.load(POOL_CLAIMED)?.unwrap_or(Amount::zero()) + balance)
    }

    /// The total liquidity ever contained in this pool.
    fn lifetime (&self) -> StdResult<Volume> {
        let balance = self.balance()?;
        let previous:     Option<Volume>    = self.load(POOL_TALLIED)?;
        let last_updated: Option<Monotonic> = self.load(POOL_UPDATED)?;
        if let (Some(previous), Some(last_updated)) = (previous, last_updated) {
            tally(previous, self.now()? - last_updated, balance.into())
        } else {
            Ok(Volume::zero())
        }
    }

} PoolWritable {

    pub fn set_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
        self.save(POOL_RATIO, ratio)
    }

    pub fn set_threshold (&mut self, threshold: &Monotonic) -> StdResult<&mut Self> {
        self.save(POOL_THRESHOLD, threshold)
    }

    pub fn update (&mut self, new_balance: Amount) -> StdResult<&mut Self> {
        let tallied = self.lifetime()?;
        let now     = self.now()?;
        self.save(POOL_TALLIED, tallied)?
            .save(POOL_UPDATED, now)?
            .save(POOL_BALANCE, new_balance)
    }

});

stateful!(User {

    fn address (&self) -> &CanonicalAddr {
        &self.address
    }
    fn now (&self) -> StdResult<Monotonic> {
        match self.now { Some(now) => Ok(now), None => error!("missing now") }
    }
    fn pool (&self) -> Pool<&S> {
        Pool { storage: self.storage, now: self.now }
    }

} UserReadonly {

    fn pool (&self) -> Pool<&S>;
    fn now (&self) -> StdResult<Monotonic>;
    fn address (&self) -> &CanonicalAddr;

    fn updated (&self) -> StdResult<Monotonic> {
        match self.load_ns(USER_UPDATED, self.address().as_slice())? {
            Some(x) => Ok(x),
            None    => error!("missing USER_UPDATED")
        }
    }

    fn existed (&self) -> StdResult<Monotonic> {
        Ok(self.load_ns(USER_EXISTED, self.address().as_slice())?
            .unwrap_or(0 as Monotonic))
    }

    fn elapsed (&self) -> StdResult<Monotonic> {
        let now = self.now()?;
        match self.updated() {
            Ok(updated) => if now >= updated {
                Ok(now - updated)
            } else {
                error!(format!("tried to query at {}, last update is {}", &now, &updated))
            },
            // default to 0
            Err(_) => Ok(0 as Monotonic)
        }
    }

    fn tallied (&self) -> StdResult<Volume> {
        Ok(self.load_ns(USER_TALLIED, self.address().as_slice())?
            .unwrap_or(Volume::zero()))
    }

    fn balance (&self) -> StdResult<Amount> {
        Ok(self.load_ns(USER_BALANCE, self.address().as_slice())?
            .unwrap_or(Amount::zero()))
    }

    fn claimed (&self) -> StdResult<Amount> {
        Ok(self.load_ns(USER_CLAIMED, self.address().as_slice())?
            .unwrap_or(Amount::zero()))
    }

    fn age (&self) -> StdResult<Monotonic> {
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

    fn lifetime (&self) -> StdResult<Volume> {
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
    fn reward (&self, balance: Amount) -> StdResult<(Amount, Amount, Amount)> {
        let age       = self.age()?;
        let threshold = self.pool().threshold()?;
        let pool      = self.pool().lifetime()?;
        if age >= threshold && pool > Volume::zero() {
            let user     = self.lifetime()?;
            let budget   = self.pool().budget(balance)?;
            let ratio    = self.pool().ratio()?;
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

} UserWritable {

    pub fn pool_mut (&mut self) -> Pool<&mut S> {
        Pool { storage: self.storage, now: self.now }
    }

    pub fn lock (&mut self, increment: Amount) -> StdResult<Amount> {
        let address = self.address.clone();

        // Remember when the user was last updated, i.e. now
        self.save_ns(USER_UPDATED, address.as_slice(), self.now()?)?;

        // Save the user's lifetime liquidity so far
        self.save_ns(USER_TALLIED, address.as_slice(), self.lifetime()?)?;

        // If current balance is > 0, increment the user's age
        // with the time since the last update
        self.save_ns(USER_EXISTED, address.as_slice(), self.age()?)?;

        // Increment liquidity from user
        self.save_ns(USER_BALANCE, address.as_slice(), self.balance()? + increment)?;

        // Increment liquidity in pool
        let next_balance = self.pool().balance()? + increment.into();
        self.pool_mut().update(next_balance)?;

        // Return the amount to lock
        Ok(increment)
    }

    pub fn retrieve (&mut self, decrement: Amount) -> StdResult<Amount> {
        let address = self.address.clone();
        let balance = self.balance()?;

        // Must have enough balance to retrieve
        if balance < decrement {
            error!(format!("not enough balance ({} < {})", balance, decrement))
        } else {
            // Save the user's lifetime liquidity so far
            self.save_ns(USER_TALLIED, address.as_slice(), self.lifetime()?)?;

            // If current balance is > 0, increment the user's age
            // with the time since the last update
            self.save_ns(USER_EXISTED, address.as_slice(), self.age()?)?;

            // Remember when the user was last updated, i.e. now
            self.save_ns(USER_UPDATED, address.as_slice(),
            self.now()?)?;

            // Remove liquidity from user
            self.save_ns(USER_BALANCE, address.as_slice(), (balance - decrement)?)?;

            // Remove liquidity from pool
            let next_balance = (self.pool().balance()? - decrement.into())?;
            self.pool_mut().update(next_balance)?;

            // Return the amount to return
            Ok(decrement)
        }
    }

    pub fn claim (&mut self, balance: Amount) -> StdResult<Amount> {
        let age       = self.age()?;
        let threshold = self.pool().threshold()?;

        // Age must be above the threshold to claim
        if age >= threshold {
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
        } else {
            error!(format!("lock tokens for {} more blocks to be eligible", threshold - age))
        }
    }

});
