use fadroma::scrt::{
    cosmwasm_std::{
        Uint128, CanonicalAddr, StdResult, StdError,
        Storage, ReadonlyStorage,
    },
    storage::{Readonly, Writable}
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

/// When did each user first add liquidity
const USER_BORN:      &[u8] = b"user_born/";

/// How much liquidity has each user provided since they first appeared
/// Incremented in intervals of (blocks since last update * current volume)
const USER_TOTAL:     &[u8] = b"user_lifetime/";

/// How much liquidity does each user currently provide
const USER_VOLUME:    &[u8] = b"user_current/";

/// When did each user's liquidity amount last change
const USER_SINCE:     &[u8] = b"user_since/";

/// How much rewards has each user claimed so far
const USER_CLAIMED:   &[u8] = b"user_claimed/";

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Monotonic = u64;

/// A ratio represented as tuple (nom, denom)
pub type Ratio = (Uint128, Uint128);

/// Calculate the current total based on the stored total and the time since last update.
pub fn lifetime_liquidity (previously: Uint128, elapsed: Monotonic, volume: Uint128) -> Uint128 {
    previously + volume.multiply_ratio(Uint128::from(elapsed), 1u128)
}

/// (volume, total, since)
pub type Status = (Uint128, Uint128, u64);

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
            None => {
                error!("missing POOL_SINCE")
            },
            Some(since) => {
                if now < since {
                    error!("can't query before last update")
                } else {
                    if let (
                        Some(volume), Some(total)
                    ) = (
                        self.load(POOL_VOLUME)?, self.load(POOL_TOTAL)?,
                    ) {
                        Ok((volume, lifetime_liquidity(total, now - since, volume), since))
                    } else {
                        error!("missing POOL_VOLUME or POOL_TOTAL")
                    }
                }
            }
        }
    }

    /// Sum of currently locked LP tokens in this pool
    fn pool_volume (&self) -> StdResult<Uint128> {
        Ok(self.load(POOL_VOLUME)?.unwrap_or(Uint128::zero()))
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
    fn pool_lifetime_reward_budget (&self, balance: Uint128) -> StdResult<Uint128> {
        Ok(self.load(POOL_CLAIMED)?.unwrap_or(Uint128::zero()) + balance)
    }

    fn pool_lifetime_liquidity (&self, now: Monotonic) -> StdResult<Uint128> {
        let pool_total:  Option<Uint128>   = self.load(POOL_TOTAL)?;
        let pool_volume: Option<Uint128>   = self.load(POOL_VOLUME)?;
        let pool_since:  Option<Monotonic> = self.load(POOL_SINCE)?;
        if let (
            Some(pool_total), Some(pool_volume), Some(pool_since)
        ) = (pool_total, pool_volume, pool_since) {
            Ok(lifetime_liquidity(pool_total, now - pool_since, pool_volume))
        } else {
            error!("missing pool liquidity data")
        }
    }

    /// Current amount of LP tokens locked by this user
    fn user_volume (&self, address: &CanonicalAddr) -> StdResult<Uint128> {
        Ok(self.load_ns(USER_VOLUME, address.as_slice())?.unwrap_or(Uint128::zero()))
    }

    fn user_lifetime_liquidity (
        &self, now: Monotonic, address: &CanonicalAddr
    ) -> StdResult<Uint128> {
        let user_total:  Option<Uint128>   = self.load_ns(USER_TOTAL, address.as_slice())?;
        let user_volume: Option<Uint128>   = self.load_ns(USER_VOLUME, address.as_slice())?;
        let user_since:  Option<Monotonic> = self.load_ns(USER_SINCE, address.as_slice())?;
        if let (
            Some(user_total), Some(user_volume), Some(user_since)
        ) = (user_total, user_volume, user_since) {
            Ok(lifetime_liquidity(user_total, now - user_since, user_volume))
        } else {
            error!("missing user liquidity data")
        }
    }

    fn user_lifetime_rewards_claimed (&self, address: &CanonicalAddr) -> StdResult<Uint128> {
        Ok(self.load_ns(USER_CLAIMED, address.as_slice())?.unwrap_or(Uint128::zero()))
    }

    fn user_reward (
        &self,
        now:     Monotonic,
        balance: Uint128,
        address: &CanonicalAddr,
    ) -> StdResult<(Uint128, Uint128, Uint128)> {
        let budget = self.pool_lifetime_reward_budget(balance)?;
        let user = self.user_lifetime_liquidity(now, address)?;
        let pool = self.pool_lifetime_liquidity(now)?;
        if pool > Uint128::zero() {
            let ratio    = self.pool_ratio()?;
            let unlocked = budget.multiply_ratio(user, pool).multiply_ratio(ratio.0, ratio.1);
            let claimed  = self.user_lifetime_rewards_claimed(address)?;
            if unlocked > claimed {
                Ok((unlocked, claimed, (unlocked - claimed)?))
            } else {
                Ok((unlocked, claimed, Uint128::zero()))
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
    /// (Current total is calculated from them using the `lifetime_liquidity` function).
    fn pool_update (&mut self, now: Monotonic) -> StdResult<Uint128> {
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
                let total = lifetime_liquidity(total, now - since, volume);
                self.save(POOL_TOTAL, total)?;
                Ok(volume)
            },
            // if any of the three vars is missing:
            // (re-)initialize the contract
            _ => {
                self.save(POOL_VOLUME, Uint128::zero())?
                    .save(POOL_TOTAL,  Uint128::zero())?
                    .save(POOL_SINCE,  now)?;
                Ok(Uint128::zero())
            }
        }
    }

    /// Add liquidity
    pub fn user_lock (
        &mut self, now: Monotonic, address: CanonicalAddr, increment: Uint128
    ) -> StdResult<Uint128> {
        let volume: Option<Uint128>  = self.load_ns(USER_VOLUME, address.as_slice())?;
        let since: Option<Monotonic> = self.load_ns(USER_SINCE,  address.as_slice())?;
        let total: Option<Uint128>   = self.load_ns(USER_TOTAL,  address.as_slice())?;
        match (volume, since, total) {
            (Some(volume), Some(since), Some(total)) => {
                // if the user is known, update it
                self.save_ns(USER_SINCE,  address.as_slice(), now)?
                    .save_ns(USER_TOTAL,  address.as_slice(), lifetime_liquidity(total, now - since, volume))?
                    .save_ns(USER_VOLUME, address.as_slice(), volume + increment)?;
            },
            _ => {
                // if the user is unknown, record it
                self.save_ns(USER_BORN,    address.as_slice(), now)?
                    .save_ns(USER_CLAIMED, address.as_slice(), Uint128::zero())?
                    .save_ns(USER_SINCE,   address.as_slice(), now)?
                    .save_ns(USER_TOTAL,   address.as_slice(), Uint128::zero())?
                    .save_ns(USER_VOLUME,  address.as_slice(), increment)?;
            }
        }
        // if recording it in the user's balance went fine
        // tally the pool and update its current state
        let incremented = self.pool_update(now)? + increment;
        self.save(POOL_VOLUME, incremented)?
            .save(POOL_SINCE,  now)?;
        Ok(increment)
    }

    /// Remove liquidity
    pub fn user_retrieve (
        &mut self, now: Monotonic, address: CanonicalAddr, decrement: Uint128
    ) -> StdResult<Uint128> {
        let volume: Option<Uint128> = self.load_ns(USER_VOLUME, address.as_slice())?;
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
        &mut self, now: Monotonic, balance: Uint128, address: &CanonicalAddr
    ) -> StdResult<Uint128> {
        let budget = self.pool_lifetime_reward_budget(balance)?;
        println!("\n[Now: {} | Budget: {}/{} | Address: {}]", &now, &balance, &budget, &address);
        let (unlocked, claimed, claimable) = self.user_reward(now, balance, address)?;
        println!("[Unlocked: {} | Claimed: {} | Claimable: {}]", &unlocked, &claimed, &claimable);
        if claimable > Uint128::zero() {
            // something to claim
            self.save_ns(USER_CLAIMED, address.as_slice(), &unlocked)?;
            Ok(claimable)
        } else if unlocked > Uint128::zero() {
            // everything claimed
            error!("already claimed")
        } else {
            error!("nothing to claim")
        }
    }
}
