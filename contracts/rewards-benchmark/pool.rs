use fadroma::scrt::{
    cosmwasm_std::{Uint128, CanonicalAddr, StdResult, StdError, Storage, ReadonlyStorage},
    storage::{Readonly, Writable},
    utils::Uint256
};

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) };
}

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current balance)
const POOL_LIFETIME:  &[u8] = b"pool_lifetime";
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
const USER_LIFETIME:  &[u8] = b"user_lifetime/";
/// How much liquidity does each user currently provide
const USER_BALANCE:   &[u8] = b"user_current/";
/// When did each user's liquidity amount last change
const USER_UPDATED:   &[u8] = b"user_updated/";
/// How much rewards has each user claimed so far
const USER_CLAIMED:   &[u8] = b"user_claimed/";

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Monotonic = u64;
/// Amount of funds
pub type Volume    = Uint128;
/// Volume (u128) * time (u64)
pub type Liquidity = Uint256;
/// A ratio represented as tuple (nom, denom)
pub type Ratio     = (Uint128, Uint128);
/// (balance, lifetime, last update)
pub type Status    = (Volume, Liquidity, Monotonic);

/// Calculate the current total based on the stored total and the time since last update.
pub fn tally (
    total_before_last_update: Liquidity,
    time_updated_last_update: Monotonic,
    value_after_last_update:  Volume
) -> StdResult<Liquidity> {
    total_before_last_update + Liquidity::from(value_after_last_update)
        .multiply_ratio(time_updated_last_update, 1u128)?
}

/// A reward pool distributes rewards from its balance among liquidity providers
/// depending on how much liquidity they have provided and for what duration.
pub struct RewardPoolController <S> {
    storage: S
}

impl <S> RewardPoolController <S> {
    pub fn new (storage: S) -> Self { Self { storage } }
}
//why not just:
//impl <S: ReadonlyStorage> Readonly <S> for RewardPoolController <S> {
    //fn storage (&self) -> S { self.storage }
//}
//instead of these three:
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
        if let Some(since) = self.load(POOL_UPDATED)? {
            if now >= since {
                let balance: Option<Volume>    = self.load(POOL_BALANCE)?;
                let total:   Option<Liquidity> = self.load(POOL_LIFETIME)?;
                if let (Some(balance), Some(total)) = (balance, total) {
                    let elapsed = now - since;
                    Ok((balance, tally(total, elapsed, balance.into())?, since))
                } else {
                    error!("missing POOL_BALANCE or POOL_LIFETIME")
                }
            } else {
                error!("can't query before last update")
            }
        } else {
            error!("missing POOL_UPDATED")
        }
    }

    /// Sum of currently locked LP tokens in this pool
    fn pool_balance (&self) -> StdResult<Volume> {
        Ok(self.load(POOL_BALANCE)?.unwrap_or(Volume::zero()))
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

    fn pool_tally (&self, now: Monotonic) -> StdResult<Liquidity> {
        let pool_lifetime: Option<Liquidity> = self.load(POOL_LIFETIME)?;
        let pool_balance:  Option<Volume>    = self.load(POOL_BALANCE)?;
        let pool_updated:  Option<Monotonic> = self.load(POOL_UPDATED)?;
        if let (
            Some(pool_lifetime), Some(pool_balance), Some(pool_updated)
        ) = (pool_lifetime, pool_balance, pool_updated) {
            tally(pool_lifetime, now - pool_updated, pool_balance.into())
        } else {
            error!("missing pool liquidity data")
        }
    }

    /// Volume amount of LP tokens locked by this user
    fn user_balance (&self, address: &CanonicalAddr) -> StdResult<Volume> {
        Ok(self.load_ns(USER_BALANCE, address.as_slice())?.unwrap_or(Volume::zero()))
    }

    fn user_existed (
        &self, now: Monotonic, address: &CanonicalAddr
    ) -> StdResult<Monotonic> {
        let age     = self.load_ns(USER_EXISTED, address.as_slice())?.unwrap_or(0 as Monotonic);
        let balance = self.load_ns(USER_BALANCE, address.as_slice())?.unwrap_or(Volume::zero());
        let since   = self.load_ns(USER_UPDATED, address.as_slice())?.unwrap_or(0 as Monotonic);
        if balance > Volume::zero() {
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
        let lifetime = self.load_ns(USER_LIFETIME, address.as_slice())?.unwrap_or(Liquidity::zero());
        let balance  = self.load_ns(USER_BALANCE,  address.as_slice())?.unwrap_or(Volume::zero());
        let updated  = self.load_ns(USER_UPDATED,  address.as_slice())?.unwrap_or(0 as Monotonic);
        tally(lifetime, now - updated, balance)
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
        let user   = self.user_tally(now, address)?;
        let pool   = self.pool_tally(now)?;
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
    /// Updates the previous total, current balance, and last update in storage.
    /// (Volume total is calculated from them using the `tally` function).
    fn pool_tally (&mut self, now: Monotonic) -> StdResult<Volume> {
        if let (
            Some(balance), Some(total), Some(since)
        ) = (
            self.load(POOL_BALANCE)?,
            self.load(POOL_LIFETIME)?,
            self.load(POOL_UPDATED)? as Option<Monotonic>
        ) {
            // if all three are present: we can update
            // the total of the liquidity ever provided
            let total = tally(total, now - since, balance)?;
            self.save(POOL_LIFETIME, total)?;
            Ok(balance)
        } else {
            // if any of the three vars is missing:
            // (re-)initialize the pool
            self.save(POOL_BALANCE,  Volume::zero())?
                .save(POOL_LIFETIME, Liquidity::zero())?
                .save(POOL_UPDATED,  now)?;
            Ok(Volume::zero())
        }
    }

    /// Called before each operation that changes the total amount of liquidity.
    /// Updates the previous total, current volume, and last update in storage.
    /// (Volume total is calculated from them using the `tally` function).
    /// Returns the current volume
    fn user_tally (&mut self, now: Monotonic, address: CanonicalAddr) -> StdResult<Volume> {
        let address = address.as_slice();
        if let (
            Some(balance), Some(total), Some(since), Some(age)
        ) = (
            self.load_ns(USER_BALANCE,  address)?,
            self.load_ns(USER_LIFETIME, address)?,
            self.load_ns(USER_UPDATED,  address)? as Option<Monotonic>,
            self.load_ns(USER_EXISTED,  address)? as Option<Monotonic>
        ) {
            // if all three are present: we can update
            // the total of the liquidity ever provided
            let elapsed = now - since;
            let total = tally(total, elapsed, balance)?;
            self.save_ns(USER_LIFETIME, address, total)?;
            Ok(balance)
        } else {
            // if any of the three vars is missing:
            // (re-)initialize the user
            self.save_ns(USER_BALANCE,  address, Volume::zero())?
                .save_ns(USER_LIFETIME, address, Liquidity::zero())?
                .save_ns(USER_UPDATED,  address, now)?;
            Ok(Volume::zero())
        }
    }

    /// Add liquidity
    pub fn user_lock (
        &mut self, now: Monotonic, address: CanonicalAddr, increment: Volume
    ) -> StdResult<Volume> {
        //if let (
            //Some(volume), Some(total), Some(since), Some(age)
        //) = (
            //self.load_ns(USER_BALANCE, address.as_slice())?,
            //self.load_ns(USER_LIFETIME,  address.as_slice())?,
            //self.load_ns(USER_UPDATED,  address.as_slice())? as Option<Monotonic>,
            //self.load_ns(USER_EXISTED,    address.as_slice())? as Option<Monotonic>
        //) {
            //// if the user is known, update the corresponding fields
            //let elapsed = now - since;
            //let user_lifetime = tally(total, elapsed, volume)?;
            //self.save_ns(USER_UPDATED,  address.as_slice(), now)?
                //.save_ns(USER_LIFETIME,  address.as_slice(), user_lifetime)?
                //.save_ns(USER_BALANCE, address.as_slice(), volume + increment)?;
            //// if the user was already providing liquidity, update its age
            //if volume > Volume::zero() {
                //self.save_ns(USER_EXISTED, address.as_slice(), age + elapsed)?;
            //}
        //} else {
            //// if the user is unknown, populate all fields for that user with initial values
            //self.save_ns(USER_CLAIMED, address.as_slice(), Liquidity::zero())?
                //.save_ns(USER_EXISTED,     address.as_slice(), 0u64)?
                //.save_ns(USER_UPDATED,   address.as_slice(), now)?
                //.save_ns(USER_LIFETIME,   address.as_slice(), Liquidity::zero())?
                //.save_ns(USER_BALANCE,  address.as_slice(), increment)?;
        //}

        match self.load_ns(USER_UPDATED, address.as_slice())? as Option<Volume> {
            None => {
                // First time lock - set liquidity
                self.save_ns(USER_BALANCE, address.as_slice(), increment)?;
                self.save_ns(USER_CLAIMED, address.as_slice(), Volume::zero())?;
                self.save_ns(USER_EXISTED, address.as_slice(), Volume::zero())?;
            },
            Some(since) => {
                // Increment liquidity of user
                let balance = self.load_ns(USER_BALANCE, address.as_slice())?;
                self.save_ns(USER_BALANCE, address.as_slice(), balance + increment)?;
            }
        }

        self.save_ns(USER_UPDATED, address.as_slice(), now)?;

        // Increment liquidity in pool
        let new_pool_balance = self.pool_tally(now)? + increment;
        self.save(POOL_BALANCE, new_pool_balance)?
            .save(POOL_UPDATED, now)?;

        // Return the amount to lock
        Ok(increment)
    }

    /// Remove liquidity
    pub fn user_retrieve (
        &mut self, now: Monotonic, address: CanonicalAddr, decrement: Volume
    ) -> StdResult<Volume> {
        match self.load_ns(USER_BALANCE, address.as_slice())? as Option<Volume> {
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

    /// Calculate how much a provider can claim,
    /// subtract it from the total balance, and return it.
    pub fn user_claim (
        &mut self, now: Monotonic, balance: Volume, address: &CanonicalAddr
    ) -> StdResult<Volume> {
        let age       = self.user_existed(now, address)?;
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
