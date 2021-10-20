use std::{rc::Rc, cell::RefCell};

use crate::{
    rewards_math::*,
    rewards_field::{Field, FieldFactory},
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
    last_lifetime: Field<S, Volume>,

    /// How much liquidity is there in the whole pool right now.
    /// Incremented/decremented on lock/unlock.
    locked:        Field<S, Amount>,

    /// When was liquidity last updated.
    /// Set to current time on lock/unlock.
    timestamp:     Field<S, Time>,

    /// Rewards claimed by everyone so far.
    /// Incremented on claim.
    claimed:       Field<S, Amount>,

    #[cfg(feature="age_threshold")]
    /// How much the user needs to wait before they can claim for the first time.
    /// Configured on init.
    pub threshold:     Field<S, Time>,

    #[cfg(feature="claim_cooldown")]
    /// How much the user must wait between claims.
    /// Configured on init.
    pub cooldown:      Field<S, Time>,

    #[cfg(feature="global_ratio")]
    /// Ratio of liquidity provided to rewards received.
    /// Configured on init.
    pub global_ratio:  Field<S, Ratio>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// Used to compute what portion of the time the pool was not empty.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// by the time elapsed since the last update.
    last_liquid:   Field<S, Time>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// The first time a user locks liquidity,
    /// this is set to the current time.
    /// Used to calculate pool's liquidity ratio.
    seeded:        Field<S, Option<Time>>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// Store the moment the user is created to compute total pool existence.
    /// Set on init.
    pub created:       Field<S, Time>,

    #[cfg(feature="pool_closes")]
    /// Whether this pool is closed
    closed:        Field<S, Option<(Time, String)>>
}

impl<S> Pool<S> {
    pub fn new (storage: Rc<RefCell<S>>) -> Self {
        Self {
            storage,

            now:     None,
            balance: None,

            last_lifetime: storage.field(b"/pool/lifetime"),
            locked:        storage.field(b"/pool/locked"),
            timestamp:     storage.field(b"/pool/locked"),
            claimed:       storage.field(b"/pool/claimed"),

            #[cfg(feature="age_threshold")]
            threshold:     storage.field(b"/pool/threshold"),

            #[cfg(feature="claim_cooldown")]
            cooldown:      storage.field(b"/pool/cooldown"),

            #[cfg(feature="global_ratio")]
            global_ratio:  storage.field(b"/pool/global_ratio"),

            #[cfg(feature="pool_liquidity_ratio")]
            last_liquid:   storage.field(b"/pool/last_liquid"),

            #[cfg(feature="pool_liquidity_ratio")]
            seeded:        storage.field(b"/pool/seeded"),

            #[cfg(feature="pool_liquidity_ratio")]
            created:       storage.field(b"/pool/created"),

            #[cfg(feature="pool_closes")]
            closed:        storage.field(b"/pool/closed"),
        }
    }

    /// Return a new Pool at given time
    pub fn at (self, now: Time) -> Self {
        Self { now: Some(now), ..self }
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
}

impl <S: ReadonlyStorage> Pool<S> {

    // time-related getters --------------------------------------------------------------------

    /// Get the time since last update (0 if no last update)
    pub fn elapsed (&self) -> StdResult<Time> {
        Ok(self.now()? - self.timestamp()?)
    }

    /// Get the current time or fail
    pub fn now (&self) -> StdResult<Time> {
        let mut now = self.now.ok_or(StdError::generic_err("current time not set"))?;

        // stop time when closing pool
        #[cfg(feature="pool_closes")]
        if let Some((t_closed, _)) = self.closed()? {
            if now < t_closed {
                return Err(StdError::generic_err("no time travel")); }
            now = t_closed
        }

        Ok(now)
    }

    /// Load the last update timestamp or default to current time
    /// (this has the useful property of keeping `elapsed` zero for strangers)
    pub fn timestamp (&self) -> StdResult<Time> {
        self.timestamp.get_or_default(self.now()?)
    }

    // lp token-related getters ----------------------------------------------------------------

    /// The total liquidity ever contained in this pool.
    pub fn lifetime (&self) -> StdResult<Volume> {
        tally(self.last_lifetime()?, self.elapsed()?, self.locked()?)
    }

    /// Snapshot of total liquidity at moment of last update.
    fn last_lifetime (&self) -> StdResult<Volume> {
        self.last_lifetime.get_or_default(Volume::zero())
    }

    /// Amount of currently locked LP tokens in this pool
    pub fn locked (&self) -> StdResult<Amount> {
        self.locked.get_or_default(Amount::zero())
    }

    // reward-related getters ------------------------------------------------------------------

    /// Amount of rewards already claimed
    pub fn claimed (&self) -> StdResult<Amount> {
        self.claimed.get_or_default(Amount::zero())
    }

    /// The full reward budget = rewards claimed + current balance of this contract in reward token
    pub fn budget (&self) -> StdResult<Amount> {
        Ok(self.claimed()? + self.balance())
    }

    /// Current balance in reward token, or zero.
    pub fn balance (&self) -> Amount {
        self.balance.unwrap_or(Amount::zero())
    }

    // balancing features ----------------------------------------------------------------------

    #[cfg(feature="age_threshold")]
    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    pub fn threshold (&self) -> StdResult<Time> {
        self.threshold.get_or_err("missing lock threshold")
    }

    #[cfg(feature="claim_cooldown")]
    /// For how many blocks does the user need to wait
    /// after claiming rewards before being able to claim them again
    pub fn cooldown (&self) -> StdResult<Time> {
        self.threshold.get_or_err("missing claim cooldown")
    }

    #[cfg(feature="global_ratio")]
    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1 to make sure rewards budget is sufficient.
    pub fn global_ratio (&self) -> StdResult<Ratio> {
        self.global_ratio.get_or_err("missing reward ratio")
    }

    #[cfg(feature="pool_liquidity_ratio")]
    /// Time for which the pool was not empty.
    pub fn liquid (&self) -> StdResult<Time> {
        let mut liquid = self.last_liquid.get_or_default(self.existed()?)?;
        if self.locked()? > Amount::zero() {
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
        Ok(self.now()? - self.seeded()?) 
    }

    #[cfg(feature="pool_liquidity_ratio")]
    fn seeded (&self) -> StdResult<Time> {
        self.seeded.get_or_err("nobody has locked any tokens yet")
    }

    #[cfg(feature="pool_liquidity_ratio")]
    fn created (&self) -> StdResult<Time> {
        self.created.get_or_err("missing creation date")
    }

    #[cfg(feature="pool_closes")]
    pub fn closed (&self) -> StdResult<Option<(Time, String)>> {
        self.closed.get()
    }
}

impl <S: ReadonlyStorage + Storage> Pool<S> {

    /// Increment the total amount of claimed rewards for all users.
    pub fn increment_claimed (mut self, reward: Amount) -> StdResult<()> {
        self.claimed.set(&(self.claimed()? + reward))
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

        #[cfg(feature="pool_liquidity_ratio")]
        self.last_liquid.set(&self.liquid()?)?;

        self.last_lifetime.set(&lifetime)?;
        self.timestamp.set(&now)?;
        self.locked.set(&balance)?;

        Ok(())
    }

    // balancing features config ---------------------------------------------------------------

    #[cfg(feature="pool_liquidity_ratio")]
    pub fn set_seeded (self, time: &Time) -> StdResult<()> {
        self.seeded.set(&Some(*time))
    }

    #[cfg(feature="pool_liquidity_ratio")]
    pub fn set_created (self, time: &Time) -> StdResult<()> {
        self.created.set(time)
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
