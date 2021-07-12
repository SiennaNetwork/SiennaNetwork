use core::borrow::BorrowMut;
use fadroma::scrt::{
    cosmwasm_std::{
        Uint128, CanonicalAddr, StdResult, StdError,
        Storage, ReadonlyStorage,
        to_vec, from_slice
    },
    storage::{Readonly, Writable}
};
use serde::{Serialize,de::DeserializeOwned};

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) };
}

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current volume)
pub const POOL_TOTAL:    &[u8] = b"pool_total";

/// How much liquidity is there in the whole pool right now
pub const POOL_VOLUME:   &[u8] = b"pool_volume";

/// When was liquidity last updated
pub const POOL_SINCE:    &[u8] = b"pool_since";

/// When was liquidity last updated
//const POOL_CLAIMED:  &[u8] = b"pool_claimed";

/// When did each user first add liquidity
const USER_BORN:     &[u8] = b"user_born/";

/// How much liquidity has each user provided since they first appeared
/// Incremented in intervals of (blocks since last update * current volume)
pub const USER_TOTAL:    &[u8] = b"user_lifetime/";

/// How much liquidity does each user currently provide
pub const USER_VOLUME:   &[u8] = b"user_current/";

/// When did each user's liquidity amount last change
pub const USER_SINCE:    &[u8] = b"user_since/";

/// How much rewards has each user claimed so far
const USER_CLAIMED:  &[u8] = b"user_claimed/";

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Monotonic = u64;

/// Calculate the current total based on the stored total and the time since last update.
pub fn so_far (total: Uint128, elapsed: Monotonic, volume: Uint128) -> Uint128 {
    total + volume.multiply_ratio(Uint128::from(elapsed), 1u128)
}

/// (volume, total, since)
pub type Status = (Uint128, Uint128, u64);

/// A reward pool distributes rewards from its balance among liquidity providers
/// depending on how much liquidity they have provided and for what duration.
pub struct RewardPoolController <S> (S);

impl <S> RewardPoolController <S> {
    pub fn new (storage: S) -> Self { Self(storage) }
}
impl <S: ReadonlyStorage> Readonly <S> for RewardPoolController <&S> {
    fn storage (&self) -> &S { &self.0 }
}
impl <S: Storage> Readonly <S> for RewardPoolController <&mut S> {
    fn storage (&self) -> &S { self.0 }
}
impl <S: Storage> Writable <S> for RewardPoolController <&mut S> {
    fn storage_mut (&mut self) -> &mut S { &mut *self.0 }
}

/// It's ugly that this needs to be a trait.
pub trait RewardPoolCalculations <S: ReadonlyStorage>: Readonly<S> {
    /// Return a status report
    fn status (&self, now: Monotonic) -> StdResult<Status> {
        match self.load(POOL_SINCE)? {
            None => {
                error!("missing POOL_SINCE")
            },
            Some(since) => {
                if now < since {
                    error!("can't query before last update")
                } else {
                    if let (Some(volume), Some(total)) = (
                        self.load(POOL_VOLUME)?,
                        self.load(POOL_TOTAL)?,
                    ) {
                        Ok((volume, so_far(total, now - since, volume), since))
                    } else {
                        error!("missing POOL_VOLUME or POOL_TOTAL")
                    }
                }
            }
        }
    }

    fn get_claimable (&self, budget: Uint128, now: Monotonic, address: &CanonicalAddr) -> StdResult<Uint128> {
        let (claimable, _) = self.calculate_reward(budget, address, now)?;
        Ok(claimable)
    }

    fn calculate_reward (
        &self,
        budget:  Uint128,
        address: &CanonicalAddr,
        now:     Monotonic
    ) -> StdResult<(Uint128, Uint128)> {
        let user    = self.user_so_far(now, address)?;
        let pool    = self.pool_so_far(now)?;
        let claimed = self.get_user_claimed(address)?;
        let reward  = budget.multiply_ratio(user, pool);
        if reward > claimed {
            Ok(((reward - claimed)?, reward))
        } else {
            Ok((Uint128::zero(), Uint128::zero()))
        }
    }

    fn user_so_far (
        &self, now: Monotonic, address: &CanonicalAddr
    ) -> StdResult<Uint128> {
        let user_total: Option<Uint128> = self.load_ns(USER_TOTAL, address.as_slice())?;
        let user_volume: Option<Uint128> = self.load_ns(USER_VOLUME, address.as_slice())?;
        let user_since: Option<Monotonic> = self.load_ns(USER_SINCE, address.as_slice())?;
        if let (
            Some(user_total), Some(user_volume), Some(user_since)
        ) = (user_total, user_volume, user_since) {
            Ok(so_far(user_total, now - user_since, user_volume))
        } else {
            error!("missing user liquidity data")
        }
    }

    fn pool_so_far (&self, now: Monotonic) -> StdResult<Uint128> {
        let pool_total: Option<Uint128> = self.load(POOL_TOTAL)?;
        let pool_volume: Option<Uint128> = self.load(POOL_VOLUME)?;
        let pool_since: Option<Monotonic> = self.load(POOL_SINCE)?;
        if let (
            Some(pool_total), Some(pool_volume), Some(pool_since)
        ) = (pool_total, pool_volume, pool_since) {
            Ok(so_far(pool_total, now - pool_since, pool_volume))
        } else {
            error!("missing pool liquidity data")
        }
    }

    fn get_user_claimed (&self, address: &CanonicalAddr) -> StdResult<Uint128> {
        Ok(self.load_ns(USER_CLAIMED, address.as_slice())?.unwrap_or(Uint128::zero()))
    }

    fn get_user_balance (&self, address: &CanonicalAddr) -> StdResult<Uint128> {
        Ok(self.load_ns(USER_VOLUME, address.as_slice())?.unwrap_or(Uint128::zero()))
    }

    fn get_pool_volume (&self) -> StdResult<Uint128> {
        Ok(self.load(POOL_VOLUME)?.unwrap_or(Uint128::zero()))
    }
}

impl <S: Storage> RewardPoolCalculations <S> for RewardPoolController <&S> {}

impl <S: Storage + ReadonlyStorage> RewardPoolCalculations <S> for RewardPoolController <&mut S> {}

impl <S: Storage + ReadonlyStorage> RewardPoolController <&mut S> {

    /// Called before each operation that changes the total amount of liquidity.
    /// Updates the previous total, current volume, and last update in storage.
    /// (Current total is calculated from them using the `so_far` function).
    fn update (&mut self, now: Monotonic) -> StdResult<Uint128> {
        // update balance so far
        let since: Option<Monotonic> = self.load(POOL_SINCE)?;
        match (
            self.load(POOL_VOLUME)?,
            self.load(POOL_TOTAL)?,
            since
        ) {
            // if all three are present: we can update
            // the total of the liquidity ever provided
            (Some(volume), Some(total), Some(since)) => {
                let total = so_far(total, now - since, volume);
                self.save(POOL_TOTAL, total)?;
                Ok(volume)
            },
            // if any of the three vars is missing:
            // (re-)initialize the contract
            _ => {
                self.save(POOL_VOLUME, Uint128::zero())?;
                self.save(POOL_TOTAL,  Uint128::zero())?;
                self.save(POOL_SINCE,  now)?;
                Ok(Uint128::zero())
            }
        }
    }

    /// Add liquidity
    pub fn lock (
        &mut self, now: Monotonic, address: CanonicalAddr, increment: Uint128
    ) -> StdResult<Uint128> {
        let volume: Option<Uint128>  = self.load_ns(USER_VOLUME, address.as_slice())?;
        let since: Option<Monotonic> = self.load_ns(USER_SINCE,  address.as_slice())?;
        let total: Option<Uint128>   = self.load_ns(USER_TOTAL,  address.as_slice())?;
        match (volume, since, total) {
            (Some(volume), Some(since), Some(total)) => {
                // if the user is known, update it
                self.save_ns(USER_SINCE,  address.as_slice(), now)?;
                self.save_ns(USER_TOTAL,  address.as_slice(), so_far(total, now - since, volume))?;
                self.save_ns(USER_VOLUME, address.as_slice(), volume + increment)?;
            },
            _ => {
                // if the user is unknown, record it
                self.save_ns(USER_BORN,    address.as_slice(), now)?;
                self.save_ns(USER_CLAIMED, address.as_slice(), Uint128::zero())?;
                self.save_ns(USER_SINCE,   address.as_slice(), now)?;
                self.save_ns(USER_TOTAL,   address.as_slice(), Uint128::zero())?;
                self.save_ns(USER_VOLUME,  address.as_slice(), increment)?;
            }
        }
        // if recording it in the user's balance went fine
        // tally the pool and update its current state
        let incremented = self.update(now)? + increment;
        self.save(POOL_VOLUME, incremented)?;
        self.save(POOL_SINCE,  now)?;
        Ok(increment)
    }

    /// Remove liquidity
    pub fn retrieve (
        &mut self, now: Monotonic, address: CanonicalAddr, decrement: Uint128
    ) -> StdResult<Uint128> {
        let volume: Option<Uint128> = self.load_ns(USER_VOLUME, address.as_slice())?;
        match volume {
            None => error!("never provided liquidity"),
            Some(volume) => {
                if volume < decrement {
                    error!(format!("not enough balance ({} < {})", volume, decrement))
                } else {
                    let decremented = (self.update(now)? - decrement)?;
                    self.save(POOL_VOLUME, decremented)?;
                    self.save(POOL_SINCE,  now)?;
                    Ok(decrement)
                }
            }
        }
    }

    /// Calculate how much a provider can claim,
    /// subtract it from the total balance, and return it.
    pub fn claim (
        &mut self, budget: Uint128, address: &CanonicalAddr, now: Monotonic
    ) -> StdResult<Uint128> {
        let (amount, reward) = self.calculate_reward(budget, address, now)?;
        if reward > Uint128::zero() {
            self.save_ns(USER_CLAIMED, address.as_slice(), &reward)?
        }
        Ok(amount)
    }
}
