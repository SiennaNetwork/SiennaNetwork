use std::{rc::Rc, cell::RefCell};

use crate::{
    rewards_math::*,
    rewards_field::*,
    rewards_user::User,
};

use fadroma::scrt::{
    cosmwasm_std::StdError,
    storage::*
};

/// Reward pool
pub struct Pool <S> {
    pub storage: Rc<RefCell<S>>,

    now:     Option<Time>,
    balance: Option<Amount>,

    /// How much liquidity has this pool contained up to this point.
    /// On lock/unlock, if locked > 0 before the operation, this is incremented
    /// in intervals of (moments since last update * current balance)
    last_lifetime:     Field<S, Volume>,

    /// How much liquidity is there in the whole pool right now.
    /// Incremented/decremented on lock/unlock.
    pub locked:        Field<S, Amount>,

    /// Load the last update timestamp or default to current time
    /// (this has the useful property of keeping `elapsed` zero for strangers)
    /// When was liquidity last updated.
    /// Set to current time on lock/unlock.
    pub timestamp:     Field<S, Time>,

    /// Rewards claimed by everyone so far.
    /// Amount of rewards already claimed
    /// Incremented on claim.
    pub claimed:       Field<S, Amount>,

    #[cfg(feature="age_threshold")]
    /// How much the user needs to wait before they can claim for the first time.
    /// Configured on init.
    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    pub threshold:     Field<S, Time>,

    #[cfg(feature="claim_cooldown")]
    /// How much the user must wait between claims.
    /// Configured on init.
    /// For how many blocks does the user need to wait
    /// after claiming rewards before being able to claim them again
    pub cooldown:      Field<S, Time>,

    #[cfg(feature="global_ratio")]
    /// Ratio of liquidity provided to rewards received.
    /// Configured on init.
    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1 to make sure rewards budget is sufficient.
    pub global_ratio:  Field<S, Ratio>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// Used to compute what portion of the time the pool was not empty.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// by the time elapsed since the last update.
    last_liquid:       Field<S, Time>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// The first time a user locks liquidity,
    /// this is set to the current time.
    /// Used to calculate pool's liquidity ratio.
    seeded:            Field<S, Option<Time>>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// Store the moment the user is created to compute total pool existence.
    /// Set on init.
    pub created:       Field<S, Time>,

    #[cfg(feature="pool_closes")]
    /// Whether this pool is closed
    pub closed:        Field<S, Option<(Time, String)>>
}

impl<S: Storage> Pool<S> {

    pub fn new (storage: Rc<RefCell<S>>) -> Self {
        Self {
            storage,

            now:     None,
            balance: None,

            last_lifetime: storage.field(b"/pool/lifetime")
                                  .or(Volume::zero()),

            locked:        storage.field(b"/pool/locked")
                                  .or(Amount::zero()),

            timestamp:     storage.field(b"/pool/locked")
                                  .required("missing timestamp"),

            claimed:       storage.field(b"/pool/claimed")
                                  .or(Amount::zero()),

            #[cfg(feature="age_threshold")]
            threshold:     storage.field(b"/pool/threshold")
                                  .required("missing lock threshold"),

            #[cfg(feature="claim_cooldown")]
            cooldown:      storage.field(b"/pool/cooldown")
                                  .required("missing claim cooldown"),

            #[cfg(feature="global_ratio")]
            global_ratio:  storage.field(b"/pool/global_ratio")
                                  .required("missing reward ratio"),

            #[cfg(feature="pool_liquidity_ratio")]
            last_liquid:   storage.field(b"/pool/last_liquid")
                                  .required("missing last liquid"),

            #[cfg(feature="pool_liquidity_ratio")]
            seeded:        storage.field(b"/pool/seeded")
                                  .required("nobody has locked any tokens yet"),

            #[cfg(feature="pool_liquidity_ratio")]
            created:       storage.field(b"/pool/created")
                                  .required("missing creation date"),

            #[cfg(feature="pool_closes")]
            closed:        storage.field(b"/pool/closed"),
        }
    }

    /// Return a new Pool at given time.
    /// When time is provided, some fields are redefined to have default values.
    pub fn at (self, now: Time) -> StdResult<Self> {
        Ok(Self {
            now: Some(now),

            #[cfg(feature="age_threshold")]
            threshold:   self.storage.field(b"/pool/threshold")
                                     .or(now),

            #[cfg(feature="pool_liquidity_ratio")]
            last_liquid: self.storage.field(b"/pool/last_liquid")
                                     .or(self.existed()?),

            ..self
        })
    }

    #[cfg(test)]
    /// Mutate the current time
    pub fn set_time<'a> (&'a mut self, now: Time) -> &'a mut Self {
        self.now = Some(now);
        self
    }

    /// Return a new Pool with given balance
    pub fn with_balance (self, balance: Amount) -> Self {
        Self { balance: Some(balance), ..self }
    }

    #[cfg(test)]
    /// Mutate the current balance
    pub fn set_balance<'a> (&'a mut self, balance: Amount) -> &'a mut Self {
        self.balance = Some(balance);
        self
    }

    pub fn user (self, address: HumanAddr) -> User<S> {
        User::new(self, address)
    }

    /// Get the time since last update (0 if no last update)
    pub fn elapsed (&self) -> StdResult<Time> {
        Ok(self.now()? - self.timestamp.get()?)
    }

    /// Get the current time or fail
    pub fn now (&self) -> StdResult<Time> {
        let mut now = self.now.ok_or(StdError::generic_err("current time not set"))?;

        // stop time when closing pool
        #[cfg(feature="pool_closes")]
        if let Some((t_closed, _)) = self.closed.get()? {
            if now < t_closed {
                return Err(StdError::generic_err("no time travel"));
            }
            now = t_closed
        }

        Ok(now)
    }

    /// The total liquidity ever contained in this pool.
    pub fn lifetime (&self) -> StdResult<Volume> {
        tally(
            self.last_lifetime.get()?,
            self.elapsed()?,
            self.locked.get()?
        )
    }

    /// The full reward budget = rewards claimed + current balance of this contract in reward token
    pub fn budget (&self) -> StdResult<Amount> {
        Ok(self.claimed.get()? + self.balance())
    }

    /// Current balance in reward token, or zero.
    pub fn balance (&self) -> Amount {
        self.balance.unwrap_or(Amount::zero())
    }

    #[cfg(feature="pool_liquidity_ratio")]
    /// Time for which the pool was not empty.
    pub fn liquid (&self) -> StdResult<Time> {
        let mut liquid = self.last_liquid.get()?;
        if self.locked.get()? > Amount::zero() {
            liquid += self.elapsed()?
        }
        Ok(liquid)
    }

    #[cfg(feature="pool_liquidity_ratio")]
    pub fn liquidity_ratio (&self) -> StdResult<Amount> {
        Ok(Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(self.liquid()?, self.existed()?)?
            .low_u128().into()
        )
    }

    #[cfg(feature="pool_liquidity_ratio")]
    pub fn existed (&self) -> StdResult<Time> {
        Ok(self.now()? - self.seeded.get()?) 
    }

    /// Increment the total amount of claimed rewards for all users.
    pub fn increment_claimed (mut self, reward: Amount) -> StdResult<()> {
        self.claimed.set(&(self.claimed.get()? + reward))
    }

    /// Every time the amount of tokens locked in the pool is updated,
    /// the pool's lifetime liquidity is tallied and and the timestamp is updated.
    /// This is the only user-triggered input to the pool.
    pub fn update_locked (&mut self, balance: Amount) -> StdResult<()> {
        // If this is the first time someone is locking tokens in this pool.
        // store the timestamp. This is used to start the pool liquidity ratio
        // calculation from the time of first lock instead of from the time
        // of contract init.
        // * Using is_none here fails type inference.
        // * Zero timestamp is special-cased - apparently cosmwasm 0.10
        //   can't tell the difference between None and the 1970s.
        match self.seeded.get()? as Option<Time> {
            None => {
                self.seeded.set(&self.now)?;
            },
            Some(0) => {
                return Err(StdError::generic_err("you jivin' yet?"));
            },
            _ => {}
        };

        let lifetime = self.lifetime()?;
        let now      = self.now()?;
        let liquid   = self.liquid()?;
        self.last_liquid.set(&liquid)?;
        self.last_lifetime.set(&lifetime)?;
        self.locked.set(&balance)?;
        self.timestamp.set(&now)?;

        Ok(())
    }

    #[cfg(all(test, feature="pool_liquidity_ratio"))]
    pub fn reset_liquidity_ratio (&mut self) -> StdResult<()> {
        let existed = self.existed()?;
        self.update_locked(self.balance())?;
        self.existed.set(existed)
    }

    #[cfg(feature="pool_closes")]
    pub fn close (&mut self, message: String) -> StdResult<()> {
        self.closed.set(&Some((self.now()?, message)))
    }

}
