use fadroma::scrt::cosmwasm_std::{
    Uint128, CanonicalAddr, StdResult, StdError,
    Extern, Storage, Api, Querier,
    to_vec
};

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) };
}

/// How much liquidity has this pool contained up to this point
/// Incremented in intervals of (moments since last update * current volume)
const POOL_TOTAL:    &[u8] = b"pool_total";

/// How much liquidity is there in the whole pool right now
const POOL_VOLUME:   &[u8] = b"pool_volume";

/// When was liquidity last updated
const POOL_SINCE:    &[u8] = b"pool_since";

/// When was liquidity last updated
//const POOL_CLAIMED:  &[u8] = b"pool_claimed";

/// When did each user first add liquidity
const USER_BORN:     &[u8] = b"user_born/";

/// How much liquidity has each user provided since they first appeared
/// Incremented in intervals of (blocks since last update * current volume)
const USER_TOTAL:    &[u8] = b"user_lifetime/";

/// How much liquidity does each user currently provide
const USER_VOLUME:   &[u8] = b"user_current/";

/// When did each user's liquidity amount last change
const USER_SINCE:    &[u8] = b"user_since/";

/// How much rewards has each user claimed so far
const USER_CLAIMED:  &[u8] = b"user_claimed/";

#[macro_export] macro_rules! load { ($self:ident, $key:expr) => {
    fadroma::scrt::storage::load(&$self.deps.storage, $key) }; }

#[macro_export] macro_rules! save { ($self:ident, $key:expr, $val:expr) => {
    $self.deps.storage.set(&$key, &to_vec(&$val)?); }; }

#[macro_export] macro_rules! ns_load { ($self:ident, $ns:expr, $key:expr) => {
    fadroma::scrt::storage::ns_load(&$self.deps.storage, $ns, $key.as_slice()) }; }

#[macro_export] macro_rules! ns_save { ($self:ident, $ns:expr, $key:expr, $val:expr) => {
    fadroma::scrt::storage::ns_save(&mut $self.deps.storage, $ns, $key.as_slice(), &$val) }; }

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Monotonic = u64;

/// A reward pool distributes rewards from its balance among liquidity providers
/// depending on how much liquidity they have provided and for what duration.
pub struct RewardPoolController <'a, S: Storage, A: Api, Q: Querier> {
    deps: &'a mut Extern<S, A, Q>
}

fn so_far (total: Uint128, elapsed: Monotonic, volume: Uint128) -> Uint128 {
    total + volume.multiply_ratio(Uint128::from(elapsed), 1u128)
}

/// (volume, total, since)
pub type Status = (Uint128, Uint128, u64);

impl <'a, S: Storage, A: Api, Q: Querier> RewardPoolController <'a, S, A, Q> {
    /// Initialize the reward pool controller by giving it control to external dependencies.
    // Ideally that should be just `deps.storage` but I don't know how to pass it
    pub fn new (deps: &'a mut Extern<S, A, Q>) -> Self { Self { deps } }

    /// Return a status report
    pub fn status (deps: &Extern<S, A, Q>, now: Monotonic) -> StdResult<Status> {
        use fadroma::scrt::storage::load;
        match load(&deps.storage, POOL_SINCE)? {
            None => {
                error!("missing POOL_SINCE")
            },
            Some(since) => {
                if now < since {
                    error!("can't query before last update")
                } else {
                    if let (Some(volume), Some(total)) = (
                        load(&deps.storage, POOL_VOLUME)?,
                        load(&deps.storage, POOL_TOTAL)?,
                    ) {
                        Ok((volume, so_far(total, now - since, volume), since))
                    } else {
                        error!("missing POOL_VOLUME or POOL_TOTAL")
                    }
                }
            }
        }

    }

    /// Called before each operation that changes the total amount of liquidity
    /// to tally it up so far (multiplying it by the moments of time it has been current,
    /// and adding that to the lifetime total of the pool)
    fn update (&mut self, now: Monotonic) -> StdResult<Uint128> {
        // update balance so far
        let since: Option<Monotonic> = load!(self, POOL_SINCE)?;
        match (
            load!(self, POOL_VOLUME)?,
            load!(self, POOL_TOTAL)?,
            since
        ) {
            // if all three are present: we can update
            // the total of the liquidity ever provided
            (Some(volume), Some(total), Some(since)) => {
                let total = so_far(total, now - since, volume);
                save!(self, POOL_TOTAL, total);
                Ok(volume)
            },
            // if any of the three vars is missing:
            // (re-)initialize the contract
            _ => {
                save!(self, POOL_VOLUME, Uint128::zero());
                save!(self, POOL_TOTAL,  Uint128::zero());
                save!(self, POOL_SINCE,  now);
                Ok(Uint128::zero())
            }
        }
    }

    /// Add liquidity
    pub fn lock (
        &mut self, now: Monotonic, address: CanonicalAddr, increment: Uint128
    ) -> StdResult<Uint128> {
        let volume: Option<Uint128>  = ns_load!(self, USER_VOLUME, address)?;
        let since: Option<Monotonic> = ns_load!(self, USER_SINCE,  address)?;
        let total: Option<Uint128>   = ns_load!(self, USER_TOTAL,  address)?;
        match (volume, since, total) {
            (Some(volume), Some(since), Some(total)) => {
                // if the user is known, update it
                ns_save!(self, USER_SINCE,  address, now)?;
                ns_save!(self, USER_TOTAL,  address, so_far(total, now - since, volume))?;
                ns_save!(self, USER_VOLUME, address, volume + increment)?;
            },
            _ => {
                // if the user is unknown, record it
                ns_save!(self, USER_BORN,    address, now)?;
                ns_save!(self, USER_CLAIMED, address, Uint128::zero())?;
                ns_save!(self, USER_SINCE,   address, now)?;
                ns_save!(self, USER_TOTAL,   address, Uint128::zero())?;
                ns_save!(self, USER_VOLUME,  address, increment)?;
            }
        }
        // if recording it in the user's balance went fine
        // tally the pool and update its current state
        let incremented = self.update(now)? + increment;
        save!(self, POOL_VOLUME, incremented);
        save!(self, POOL_SINCE,  now);
        Ok(increment)
    }

    /// Remove liquidity
    pub fn retrieve (
        &mut self, now: Monotonic, address: CanonicalAddr, decrement: Uint128
    ) -> StdResult<Uint128> {
        let volume: Option<Uint128> = ns_load!(self, USER_VOLUME, address)?;
        match volume {
            None => error!("never provided liquidity"),
            Some(volume) => {
                if volume < decrement {
                    error!(format!("not enough balance ({} < {})", volume, decrement))
                } else {
                    let decremented = (self.update(now)? - decrement)?;
                    save!(self, POOL_VOLUME, decremented);
                    save!(self, POOL_SINCE,  now);
                    Ok(decrement)
                }
            }
        }
    }

    /// Calculate how much a provider can claim,
    /// subtract it from the total balance, and return it.
    pub fn claim (
        &mut self, address: &CanonicalAddr, balance: Uint128
    ) -> StdResult<Uint128> {
        let (amount, reward) = Self::calc_claim(&self.deps, address, balance)?;

        if reward > Uint128::zero() {
            ns_save!(self, USER_CLAIMED, address, reward)?;
        }

        Ok(amount)
    }

    pub fn get_claim_amount(
        deps: &'a Extern<S, A, Q>,
        address: &CanonicalAddr,
        balance: Uint128
    ) -> StdResult<Uint128> {
        let (amount, _) = Self::calc_claim(deps, address, balance)?;

        Ok(amount)
    }

    pub fn get_balance(
        deps: &'a Extern<S, A, Q>,
        address: &CanonicalAddr
    ) -> StdResult<Uint128> {
        use fadroma::scrt::storage::ns_load;

        let balance = ns_load(&deps.storage, USER_VOLUME, address.as_slice())?
            .unwrap_or(Uint128::zero());

        Ok(balance)
    }

    pub fn get_volume(
        deps: &'a Extern<S, A, Q>
    ) -> StdResult<Uint128> {
        use fadroma::scrt::storage::load;

        Ok(load(&deps.storage, POOL_VOLUME)?.unwrap_or(Uint128::zero()))
    }

    fn calc_claim(
        deps: &Extern<S, A, Q>,
        address: &CanonicalAddr,
        balance: Uint128
    ) -> StdResult<(Uint128, Uint128)> {
        use fadroma::scrt::storage::{load, ns_load};

        let pool_total: Option<Uint128> = load(&deps.storage, POOL_TOTAL)?;
        let user_total: Option<Uint128> = ns_load(&deps.storage, USER_TOTAL, address.as_slice())?;
        let claimed:    Option<Uint128> = ns_load(&deps.storage, USER_CLAIMED, address.as_slice())?;

        match (pool_total, user_total, claimed) {
            (Some(pool_total), Some(user_total), Some(claimed)) => {
                let reward: Uint128 = balance.multiply_ratio(user_total, pool_total);
                if reward > claimed {
                    Ok(((reward - claimed)?, reward))
                } else {
                    Ok((Uint128::zero(), Uint128::zero()))
                }
            },
            _ => error!("missing data"),
        }
    }
}
