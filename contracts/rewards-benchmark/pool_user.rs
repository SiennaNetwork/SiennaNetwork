use fadroma::scrt::{
    cosmwasm_std::{Uint128, CanonicalAddr, StdResult, StdError, Storage, ReadonlyStorage},
    storage::{Readonly, Writable},
    utils::Uint256
};

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

/// For how many units of time has this user provided liquidity
const USER_EXISTED:   &[u8] = b"user_existed/";
/// How much liquidity has each user provided since they first appeared;
/// incremented in intervals of (blocks since last update * current balance)
const USER_TALLIED:   &[u8] = b"user_lifetime/";
/// How much liquidity does each user currently provide
const USER_BALANCE:   &[u8] = b"user_current/";
/// When did each user's liquidity amount last change
const USER_UPDATED:   &[u8] = b"user_updated/";
/// How much rewards has each user claimed so far
const USER_CLAIMED:   &[u8] = b"user_claimed/";

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Monotonic = u64;
/// Amount of funds
pub type Amount    = Uint128;
/// Amount (u128) * time (u64)
pub type Volume    = Uint256;
/// A ratio represented as tuple (nom, denom)
pub type Ratio     = (Uint128, Uint128);
/// (balance, lifetime, last update)
pub type Status    = (Amount, Volume, Monotonic);

/// Calculate the current total based on the stored total and the time since last update.
pub fn tally (
    total_before_last_update: Volume,
    time_updated_last_update: Monotonic,
    value_after_last_update:  Amount
) -> StdResult<Volume> {
    total_before_last_update + Volume::from(value_after_last_update)
        .multiply_ratio(time_updated_last_update, 1u128)?
}

/// Reward pool
pub struct Pool <S> {
    storage: S,
    now:     Option<Monotonic>
}

impl<S> Pool<S> {
    /// Create a new pool with a storage handle
    pub fn new (storage: S) -> Self {
        Self { storage, now: None }
    }
    /// Add the current time to the pool for operations that need it
    pub fn at (self, now: Monotonic) -> Self {
        Self { storage: self.storage, now: Some(now) }
    }
    /// Get an individual user from the pool
    pub fn user (&self, address: CanonicalAddr) -> User<S> {
        // variant: existed check here; new_user for first lock?
        User { pool: *self, address }
    }
}

impl<S: ReadonlyStorage> Readonly<S> for Pool<&S> {
    fn storage (&self) -> &S { self.storage }
}

impl<S: ReadonlyStorage> PoolReadonly<S> for Pool<&S> {
    fn now (&self) -> StdResult<Monotonic> {
        self.now.ok_or(StdError::generic_err("current time not set"))
    }
}

pub trait PoolReadonly<S: ReadonlyStorage>: Readonly<S> {

    fn now (&self) -> StdResult<Monotonic>;

    fn status (&self) -> StdResult<(Amount, Volume, Monotonic)> {
        if let Some(last_update) = self.load(POOL_UPDATED)? {
            if self.now()? >= last_update {
                let balance  = self.load(POOL_BALANCE)? as Option<Amount>;
                let lifetime = self.load(POOL_TALLIED)? as Option<Volume>;
                if let (Some(balance), Some(lifetime)) = (balance, lifetime) {
                    let elapsed = self.now()? - last_update;
                    Ok((balance, tally(lifetime, elapsed, balance.into())?, last_update))
                } else { error!("missing POOL_BALANCE or POOL_TALLIED") }
            } else { error!("can't query before last update") }
        } else { error!("missing POOL_UPDATED") }
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
    fn threshold (&self) -> StdResult<Ratio> {
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
        let previous:     Option<Volume>    = self.load(POOL_TALLIED)?;
        let balance:      Option<Amount>    = self.load(POOL_BALANCE)?;
        let last_updated: Option<Monotonic> = self.load(POOL_UPDATED)?;
        if let (
            Some(previous), Some(balance), Some(last_updated)
        ) = (previous, balance, last_updated) {
            tally(previous, self.now()? - last_updated, balance.into())
        } else {
            error!("missing pool liquidity data")
        }
    }

}

impl<S: Storage> Writable<S> for Pool<&S> {
    fn storage_mut (&mut self) -> &mut S { &mut *self.storage }
}

impl<S: Storage> PoolWritable<S> for Pool<&S> {}

pub trait PoolWritable<S: Storage>: Writable<S> + PoolReadonly<S> {
    fn set_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
        self.save(POOL_RATIO, ratio)
    }
    fn set_threshold (&mut self, threshold: &Monotonic) -> StdResult<&mut Self> {
        self.save(POOL_THRESHOLD, threshold)
    }
    fn update (&mut self) -> StdResult<Amount> {
        Ok(Uint128::zero())
    }
}

pub struct User <S> {
    pool:    Pool<S>,
    address: CanonicalAddr
}

impl<S: ReadonlyStorage> Readonly<S> for User<&S> {
    fn storage (&self) -> &S { &self.pool.storage }
}

impl<S: Storage> Writable<S> for User<&S> {
    fn storage_mut (&mut self) -> &mut S { &mut self.pool.storage }
}

impl<S: ReadonlyStorage> UserReadonly<S> for User<&S> {
    fn pool (&self) -> &Pool<&S> {
        &self.pool
    }
    fn address (&self) -> CanonicalAddr {
        self.address
    }
    // trait fields WHEN???
}

impl<S: Storage> UserWritable<S> for User<&S> {}

pub trait UserReadonly<S: ReadonlyStorage>: Readonly<S> {

    fn pool    (&self) -> &Pool<&S>;
    fn address (&self) -> CanonicalAddr;

    fn balance (&self) -> StdResult<Amount> {
        Ok(self.load_ns(USER_BALANCE, self.address().as_slice())?
            .unwrap_or(Amount::zero()))
    }

    fn age (&self) -> StdResult<Monotonic> {
        let address = self.address().as_slice();
        let existed = self.load_ns(USER_EXISTED, address)?
            .unwrap_or(0 as Monotonic);
        let balance = self.load_ns(USER_BALANCE, address)?
            .unwrap_or(Amount::zero());
        let since   = self.load_ns(USER_UPDATED, address)?
            .unwrap_or(0 as Monotonic);
        if balance > Amount::zero() {
            // if user is currently providing liquidity,
            // the time since last update gets added to the age
            let elapsed = self.pool().now()? - since;
            Ok(existed + elapsed)
        } else {
            Ok(existed)
        }
    }

    fn lifetime (&self) -> StdResult<Volume> {
        let address = self.address().as_slice();
        tally(
            self.load_ns(USER_TALLIED, address)?
                .unwrap_or(Volume::zero()),
            self.pool().now()? - self.load_ns(USER_UPDATED, address)?
                .ok_or(error!("USER_UPDATED missing"))?,
            self.load_ns(USER_BALANCE, address)?
                .unwrap_or(Amount::zero()))
    }

    fn claimed (&self) -> StdResult<Amount> {
        Ok(self.load_ns(USER_CLAIMED, self.address().as_slice())?
            .unwrap_or(Amount::zero()))
    }

    fn reward (&self, balance: Amount) -> StdResult<(Amount, Amount, Amount)> {
        let budget = self.pool().budget(balance)?;
        let user   = self.tally()?;
        let pool   = self.pool().tally()?;
        if pool > Volume::zero() {
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
            error!("pool is empty")
        }
    }

}

pub trait UserWritable<S: Storage>: Writable<S> + UserReadonly<S> {

    fn lock (
        &mut self, now: Monotonic, address: CanonicalAddr, increment: Amount
    ) {
        match self.load_ns(USER_UPDATED, address.as_slice())? as Option<Amount> {
            None => {
                // First time lock - set liquidity
                self.save_ns(USER_BALANCE, address.as_slice(), increment)?;
                self.save_ns(USER_CLAIMED, address.as_slice(), Amount::zero())?;
                self.save_ns(USER_EXISTED, address.as_slice(), Amount::zero())?;
            },
            Some(since) => {
                // Increment liquidity of user
                let balance = self.load_ns(USER_BALANCE, address.as_slice())?;
                self.save_ns(USER_BALANCE, address.as_slice(), balance + increment)?;
            }
        }

        self.save_ns(USER_UPDATED, address.as_slice(), now)?;

        // Increment liquidity in pool
        let new_pool_balance = self.pool().tally()? + increment;
        self.save(POOL_BALANCE, new_pool_balance)?
            .save(POOL_UPDATED, now)?;

        // Return the amount to lock
        Ok(increment)
    }

    fn retrieve (
        &mut self, now: Monotonic, address: CanonicalAddr, decrement: Amount
    ) {
        match self.load_ns(USER_BALANCE, address.as_slice())? as Option<Amount> {
            None => error!("never provided liquidity"),
            Some(balance) => {
                if balance < decrement {
                    error!(format!("not enough balance ({} < {})", balance, decrement))
                } else {
                    // Remove liquidity from user
                    let new_user_balance = (balance - decrement)?;
                    self.save_ns(USER_BALANCE, address.as_slice(), new_user_balance)?;

                    // Remove liquidity from pool
                    let new_pool_balance = (self.pool_tally(now)? - decrement)?;
                    self.save(POOL_BALANCE, new_pool_balance)?
                        .save(POOL_UPDATED,  now)?;

                    // Return the amount to return
                    Ok(decrement)
                }
            }
        }
    }

    fn claim (&mut self, balance: Amount) -> StdResult<Amount> {
        let age       = self.age()?;
        let threshold = self.pool.threshold()?;
        if age >= threshold {
            let (unlocked, _claimed, claimable) =
                self.reward(balance)?;
            if claimable > Amount::zero() {
                // something to claim
                self.save_ns(USER_CLAIMED, self.address.as_slice(), &unlocked)?;
                Ok(claimable)
            } else if unlocked > Amount::zero() {
                // everything claimed
                error!("already claimed")
            } else {
                // nothing was claimable
                error!("nothing to claim")
            }
        } else {
            error!(format!("{} blocks until eligible", threshold - age))
        }
    }

}
