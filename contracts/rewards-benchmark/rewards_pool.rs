use crate::rewards_math::*;
use crate::rewards_user::*;
use fadroma::scrt::{cosmwasm_std::{StdError, CanonicalAddr}, storage::traits2::*,};

macro_rules! error { ($info:expr) => { Err(StdError::generic_err($info)) }; }

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current balance)
const TALLIED:   &[u8] = b"pool_lifetime";
/// How much liquidity is there in the whole pool right now
const BALANCE:   &[u8] = b"pool_balance";
/// When was liquidity last updated
const UPDATED:   &[u8] = b"pool_updated";
/// Rewards claimed by everyone so far
const CLAIMED:   &[u8] = b"pool_claimed";
/// Ratio of liquidity provided to rewards received
const RATIO:     &[u8] = b"pool_ratio";
/// Ratio of liquidity provided to rewards received
const THRESHOLD: &[u8] = b"pool_threshold";

/// Reward pool
pub struct Pool <S> {
    pub storage: S,
    now: Option<Monotonic>
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
    /// Get the current time or fail
    pub fn now (&self) -> StdResult<Monotonic> {
        self.now.ok_or(StdError::generic_err("current time not set"))
    }
    /// Get an individual user from the pool
    pub fn user (&self, address: CanonicalAddr) -> User<S> {
        // variant: existed check here; new_user for first lock?
        User { pool: *self, address }
    }
}

impl<S: ReadonlyStorage> Readonly<S> for Pool<S> {
    fn storage (&self) -> &S { &self.storage }
}

impl<S: ReadonlyStorage> Pool<S> {

    pub fn status (&self) -> StdResult<(Amount, Volume, Monotonic)> {
        if let Some(last_update) = self.load(UPDATED)? {
            if self.now()? >= last_update {
                let balance  = self.load(BALANCE)? as Option<Amount>;
                let lifetime = self.load(TALLIED)? as Option<Volume>;
                if let (Some(balance), Some(lifetime)) = (balance, lifetime) {
                    let elapsed = self.now()? - last_update;
                    Ok((balance, tally(lifetime, elapsed, balance.into())?, last_update))
                } else { error!("missing BALANCE or TALLIED") }
            } else { error!("can't query before last update") }
        } else { error!("missing UPDATED") }
    }

    /// Amount of currently locked LP tokens in this pool
    pub fn balance (&self) -> StdResult<Amount> {
        Ok(self.load(BALANCE)?.unwrap_or(Amount::zero()))
    }

    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1 to make sure rewards budget is sufficient. 
    pub fn ratio (&self) -> StdResult<Ratio> {
        match self.load(RATIO)? {
            Some(ratio) => Ok(ratio),
            None        => error!("missing reward ratio")
        }
    }

    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    pub fn threshold (&self) -> StdResult<Monotonic> {
        match self.load(THRESHOLD)? {
            Some(ratio) => Ok(ratio),
            None        => error!("missing reward threshold")
        }
    }

    /// The full reward budget = rewards claimed + current balance of this contract in reward token
    pub fn budget (&self, balance: Amount) -> StdResult<Amount> {
        Ok(self.load(CLAIMED)?.unwrap_or(Amount::zero()) + balance)
    }

    /// The total liquidity ever contained in this pool.
    pub fn lifetime (&self) -> StdResult<Volume> {
        let previous:     Option<Volume>    = self.load(TALLIED)?;
        let balance:      Option<Amount>    = self.load(BALANCE)?;
        let last_updated: Option<Monotonic> = self.load(UPDATED)?;
        if let (
            Some(previous), Some(balance), Some(last_updated)
        ) = (previous, balance, last_updated) {
            tally(previous, self.now()? - last_updated, balance.into())
        } else {
            error!("missing pool liquidity data")
        }
    }

}

impl<S: Storage> Writable<S> for Pool<S> {
    fn storage_mut (&mut self) -> &mut S { &mut self.storage }
}

impl<S: Storage> Pool<S> {

    pub fn set_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
        self.save(RATIO, ratio)
    }

    pub fn set_threshold (&mut self, threshold: &Monotonic) -> StdResult<&mut Self> {
        self.save(THRESHOLD, threshold)
    }

    pub fn update (&mut self, new_balance: Amount) -> StdResult<&mut Self> {
        self.save(UPDATED, self.now()?)?
            .save(BALANCE, new_balance)
    }

}
