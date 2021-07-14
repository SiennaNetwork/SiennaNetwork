use fadroma::scrt::{
    cosmwasm_std::{
        Uint128, CanonicalAddr, StdResult, StdError,
        Storage, ReadonlyStorage,
    },
    storage::{Readonly, Writable},
    utils::Uint256
};

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) };
}

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current volume)
const POOL_TOTAL:     &[u8] = b"pool_total";

/// How much liquidity is there in the whole pool right now
const POOL_VOLUME:    &[u8] = b"pool_volume";

/// When was liquidity last updated
const POOL_SINCE:     &[u8] = b"pool_since";

/// Rewards claimed by everyone so far
const POOL_CLAIMED:   &[u8] = b"pool_claimed";

/// Ratio of liquidity provided to rewards received
const POOL_RATIO:     &[u8] = b"pool_ratio";

/// Ratio of liquidity provided to rewards received
const POOL_THRESHOLD: &[u8] = b"pool_threshold";

/// For how many units of time has this user provided liquidity
const USER_AGE:       &[u8] = b"user_age/";

/// How much liquidity has each user provided since they first appeared;
/// incremented in intervals of (blocks since last update * current volume)
const USER_TOTAL:     &[u8] = b"user_lifetime/";

/// How much liquidity does each user currently provide
const USER_VOLUME:    &[u8] = b"user_current/";

/// When did each user's liquidity amount last change
const USER_SINCE:     &[u8] = b"user_since/";

/// How much rewards has each user claimed so far
const USER_CLAIMED:   &[u8] = b"user_claimed/";

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Monotonic = u64;

/// Volume liquidity
pub type Volume = Uint128;

/// Volume (u128) * time (u64). TODO change to Uint256
pub type Liquidity = Uint256;

/// A ratio represented as tuple (nom, denom)
pub type Ratio = (Uint128, Uint128);

/// Calculate the current total based on the stored total and the time since last update.
pub fn lifetime_liquidity (
    previously: Liquidity, elapsed: Monotonic, volume: Volume
) -> StdResult<Liquidity> {
    previously + Liquidity::from(volume).multiply_ratio(elapsed, 1u128)?
}

/// (volume, total, since)
pub type Status = (Volume, Liquidity, u64);

/// A reward pool distributes rewards from its balance among liquidity providers
/// depending on how much liquidity they have provided and for what duration.
pub struct RewardPoolController <S> {
    storage: S
}

impl <S> RewardPoolController <S> {
    pub fn new (storage: S) -> Self { Self { storage } }
}
impl <S: ReadonlyStorage> Readonly <S> for RewardPoolController <&S> {
    fn storage (&self) -> &S { &self.storage }
}
impl <S: Storage> Readonly <S> for RewardPoolController <&mut S> {
    fn storage (&self) -> &S { self.storage }
}
impl <S: Storage> Writable <S> for RewardPoolController <&mut S> {
    fn storage_mut (&mut self) -> &mut S { &mut *self.storage }
}

/// It's ugly that this needs to be a trait.
/// TODO: Need to find out how to abstract over mutability.
pub trait RewardPoolCalculations <S: ReadonlyStorage>: Readonly<S> {

    /// Return a status report
    fn pool_status (&self, now: Monotonic) -> StdResult<Status> {
        match self.load(POOL_SINCE)? {
            None => error!("missing POOL_SINCE"),
            Some(since) => {
                if now < since {
                    error!("can't query before last update")
                } else {
                    let volume: Option<Volume>    = self.load(POOL_VOLUME)?;
                    let total:  Option<Liquidity> = self.load(POOL_TOTAL)?;
                    if let (Some(volume), Some(total)) = (volume, total) {
                        Ok((volume, lifetime_liquidity(total, now - since, volume.into())?, since))
                    } else {
                        error!("missing POOL_VOLUME or POOL_TOTAL")
                    }
                }
            }
        }
    }

    /// Sum of currently locked LP tokens in this pool
    fn pool_volume (&self) -> StdResult<Volume> {
        Ok(self.load(POOL_VOLUME)?.unwrap_or(Volume::zero()))
    }

    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1
    fn pool_ratio (&self) -> StdResult<Ratio> {
        match self.load(POOL_RATIO)? {
            Some(ratio) => Ok(ratio),
            None => error!("missing reward ratio")
        }
    }

    /// For how many blocks the user must have provided liquidity to be eligible for rewards
    fn pool_threshold (&self) -> StdResult<Monotonic> {
        match self.load(POOL_THRESHOLD)? {
            Some(ratio) => Ok(ratio),
            None => error!("missing reward threshold")
        }
    }

    /// Sum of reward claimed + current balance of this contract in reward token
    fn pool_lifetime_reward_budget (&self, balance: Volume) -> StdResult<Volume> {
        Ok(self.load(POOL_CLAIMED)?.unwrap_or(Volume::zero()) + balance)
    }

    fn pool_lifetime_liquidity (&self, now: Monotonic) -> StdResult<Liquidity> {
        let pool_total:  Option<Liquidity>   = self.load(POOL_TOTAL)?;
        let pool_volume: Option<Volume>   = self.load(POOL_VOLUME)?;
        let pool_since:  Option<Monotonic> = self.load(POOL_SINCE)?;
        if let (
            Some(pool_total), Some(pool_volume), Some(pool_since)
        ) = (pool_total, pool_volume, pool_since) {
            lifetime_liquidity(pool_total, now - pool_since, pool_volume.into())
        } else {
            error!("missing pool liquidity data")
        }
    }

    /// Volume amount of LP tokens locked by this user
    fn user_volume (&self, address: &CanonicalAddr) -> StdResult<Volume> {
        Ok(self.load_ns(USER_VOLUME, address.as_slice())?.unwrap_or(Volume::zero()))
    }

    fn user_age (
        &self, now: Monotonic, address: &CanonicalAddr
    ) -> StdResult<Monotonic> {
        let age    = self.load_ns(USER_AGE,    address.as_slice())?.unwrap_or(0 as Monotonic);
        let volume = self.load_ns(USER_VOLUME, address.as_slice())?.unwrap_or(Volume::zero());
        let since  = self.load_ns(USER_SINCE,  address.as_slice())?.unwrap_or(0 as Monotonic);
        if volume > Volume::zero() {
            // if user is currently providing liquidity,
            // the time since last update gets added to the age
            let elapsed = now - since;
            Ok(age + elapsed)
        } else {
            Ok(age)
        }
    }

    fn user_lifetime_liquidity (
        &self, now: Monotonic, address: &CanonicalAddr
    ) -> StdResult<Liquidity> {
        let user_total  = self.load_ns(USER_TOTAL,  address.as_slice())?.unwrap_or(Liquidity::zero());
        let user_volume = self.load_ns(USER_VOLUME, address.as_slice())?.unwrap_or(Volume::zero());
        let user_since  = self.load_ns(USER_SINCE,  address.as_slice())?.unwrap_or(0 as Monotonic);
        lifetime_liquidity(user_total, now - user_since, user_volume)
    }

    fn user_lifetime_rewards_claimed (&self, address: &CanonicalAddr) -> StdResult<Volume> {
        Ok(self.load_ns(USER_CLAIMED, address.as_slice())?.unwrap_or(Volume::zero()))
    }

    fn user_reward (
        &self,
        now:     Monotonic,
        balance: Volume,
        address: &CanonicalAddr,
    ) -> StdResult<(Volume, Volume, Volume)> {
        let budget = self.pool_lifetime_reward_budget(balance)?;
        let user   = self.user_lifetime_liquidity(now, address)?;
        let pool   = self.pool_lifetime_liquidity(now)?;
        if pool > Liquidity::zero() {
            let ratio    = self.pool_ratio()?;
            let unlocked = Liquidity::from(budget)
                .multiply_ratio(user, pool)?
                .multiply_ratio(ratio.0, ratio.1)?
                .low_u128().into();
            let claimed  = self.user_lifetime_rewards_claimed(address)?;
            if unlocked > claimed {
                Ok((unlocked, claimed, (unlocked - claimed)?))
            } else {
                Ok((unlocked, claimed, Volume::zero()))
            }
        } else {
            error!("pool is empty")
        }
    }
}

impl <S: Storage> RewardPoolCalculations <S> for RewardPoolController <&S> {}

impl <S: Storage + ReadonlyStorage> RewardPoolCalculations <S> for RewardPoolController <&mut S> {}

impl <S: Storage + ReadonlyStorage> RewardPoolController <&mut S> {

    /// Configure the ratio between share of liquidity provided and amount of reward
    pub fn save_ratio (&mut self, ratio: &Ratio) -> StdResult<&mut Self> {
        self.save(POOL_RATIO, ratio)
    }

    /// For how many blocks the user must have provided liquidity to be eligible for rewards
    pub fn save_threshold (&mut self, threshold: &Monotonic) -> StdResult<&mut Self> {
        self.save(POOL_THRESHOLD, threshold)
    }

    /// Called before each operation that changes the total amount of liquidity.
    /// Updates the previous total, current volume, and last update in storage.
    /// (Volume total is calculated from them using the `lifetime_liquidity` function).
    fn pool_update (&mut self, now: Monotonic) -> StdResult<Volume> {
        if let (
            Some(volume), Some(total), Some(since)
        ) = (
            self.load(POOL_VOLUME)?,
            self.load(POOL_TOTAL)?,
            self.load(POOL_SINCE)? as Option<Monotonic>
        ) {
            // if all three are present: we can update
            // the total of the liquidity ever provided
            let total = lifetime_liquidity(total, now - since, volume)?;
            self.save(POOL_TOTAL, total)?;
            Ok(volume)
        } else {
            // if any of the three vars is missing:
            // (re-)initialize the contract
            self.save(POOL_VOLUME, Volume::zero())?
                .save(POOL_TOTAL,  Liquidity::zero())?
                .save(POOL_SINCE,  now)?;
            Ok(Volume::zero())
        }
    }

    /// Add liquidity
    pub fn user_lock (
        &mut self, now: Monotonic, address: CanonicalAddr, increment: Volume
    ) -> StdResult<Volume> {
        if let (
            Some(volume), Some(total), Some(since), Some(age)
        ) = (
            self.load_ns(USER_VOLUME, address.as_slice())?,
            self.load_ns(USER_TOTAL,  address.as_slice())?,
            self.load_ns(USER_SINCE,  address.as_slice())? as Option<Monotonic>,
            self.load_ns(USER_AGE,    address.as_slice())? as Option<Monotonic>
        ) {
            // if the user is known, update the corresponding fields
            let elapsed = now - since;
            let user_total = lifetime_liquidity(total, elapsed, volume)?;
            self.save_ns(USER_SINCE,  address.as_slice(), now)?
                .save_ns(USER_TOTAL,  address.as_slice(), user_total)?
                .save_ns(USER_VOLUME, address.as_slice(), volume + increment)?;
            // if the user was already providing liquidity, update its age
            if volume > Volume::zero() {
                self.save_ns(USER_AGE, address.as_slice(), age + elapsed)?;
            }
        } else {
            // if the user is unknown, populate all fields for that user with initial values
            self.save_ns(USER_CLAIMED, address.as_slice(), Liquidity::zero())?
                .save_ns(USER_AGE,     address.as_slice(), 0u64)?
                .save_ns(USER_SINCE,   address.as_slice(), now)?
                .save_ns(USER_TOTAL,   address.as_slice(), Liquidity::zero())?
                .save_ns(USER_VOLUME,  address.as_slice(), increment)?;
        }
        // if updating the user's balance went fine, also update the pool
        let incremented = self.pool_update(now)? + increment;
        self.save(POOL_VOLUME, incremented)?
            .save(POOL_SINCE,  now)?;
        // return the amount locked
        Ok(increment)
    }

    /// Remove liquidity
    pub fn user_retrieve (
        &mut self, now: Monotonic, address: CanonicalAddr, decrement: Volume
    ) -> StdResult<Volume> {
        let volume: Option<Volume> = self.load_ns(USER_VOLUME, address.as_slice())?;
        match volume {
            None => error!("never provided liquidity"),
            Some(volume) => {
                if volume < decrement {
                    error!(format!("not enough balance ({} < {})", volume, decrement))
                } else {
                    let decremented = (self.pool_update(now)? - decrement)?;
                    self.save(POOL_VOLUME, decremented)?
                        .save(POOL_SINCE,  now)?;
                    Ok(decrement)
                }
            }
        }
    }

    /// Calculate how much a provider can claim,
    /// subtract it from the total balance, and return it.
    pub fn user_claim (
        &mut self, now: Monotonic, balance: Volume, address: &CanonicalAddr
    ) -> StdResult<Volume> {
        let age       = self.user_age(now, address)?;
        let threshold = self.pool_threshold()?;
        if age >= threshold {
            let (unlocked, _claimed, claimable) =
                self.user_reward(now, balance, address)?;
            if claimable > Volume::zero() {
                // something to claim
                self.save_ns(USER_CLAIMED, address.as_slice(), &unlocked)?;
                Ok(claimable)
            } else if unlocked > Volume::zero() {
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
