use fadroma::scrt::{
    cosmwasm_std::*,
    callback::ContractInstance as ContractLink,
    snip20_api::ISnip20,
    contract::{message, messages},
    storage::*
};

use serde::{Serialize, de::DeserializeOwned};

use crate::api::*;
use crate::core::*;
use crate::math::*;
use crate::auth::ViewingKey;

use std::{
    rc::Rc,
    cell::RefCell
};

message!(RewardsConfig {
    lp_token:  Option<ContractLink<HumanAddr>>,
    ratio:     Option<(Uint128, Uint128)>,
    threshold: Option<Time>,
    cooldown:  Option<Time>
});

messages!(RewardsHandle {
    Configure(RewardsConfig),
    ClosePool { message: String }
    Lock      { amount: Amount }
    Retrieve  { amount: Amount }
    Claim     {}
});

messages!(RewardsQuery {
    Status { at: Time, address: Option<HumanAddr>, key: Option<String> }
});

messages!(RewardsResponse {
    Status {
        time:         Time,
        epoch:        u64,
        lp_token:     ContractLink<HumanAddr>,
        reward_token: ContractLink<HumanAddr>,
        pool:         PoolStatus,
        user:         Option<UserStatus>
    }
});

message!(PoolStatus {
    last_update: Time,
    lifetime:    Volume,
    locked:      Amount,
    closed:      Option<String>,
    balance:     Amount,
    claimed:     Amount,
    threshold:   Time,
    cooldown:    Time,
    liquid:      Amount
});

message!(UserStatus {
    last_update: Time,
    lifetime:    Volume,
    locked:      Amount,
    share:       Amount,
    earned:      Amount,
    claimed:     Amount,
    claimable:   Amount,
    age:         Time,
    cooldown:    Time
});

pub trait Rewards<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q> {
    fn init (&mut self, env: &Env, msg: &Init) -> StdResult<()> {
        self.storage().set(b"/self", &ContractLink {
            address:   env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        }.canonize(&self.api())?);
        self.storage().set(b"/reward_token", &msg.reward_token.canonize(&self.api()));
        self.storage().set(b"/reward_token_vk", &msg.viewing_key);
        self.configure(RewardsConfig {
            lp_token:  msg.lp_token,
            ratio:     Some(msg.ratio.unwrap_or((1u128.into(), 1u128.into()))),
            threshold: Some(msg.threshold.unwrap_or(DAY)),
            cooldown:  Some(msg.cooldown.unwrap_or(DAY))
        });
        Ok(())
    }

    fn handle (&mut self, env: &Env, msg: &RewardsHandle) -> StdResult<Option<HandleResponse>> {
        match msg {
            RewardsHandle::Configure(config) => self.configure(config),
            RewardsHandle::Lock      { .. }  => self.lock(msg),
            RewardsHandle::Retrieve  { .. }  => self.retrieve(msg),
            RewardsHandle::Claim     {}      => self.claim()
            RewardsHandle::ClosePool { .. }  => self.close_pool(msg),
        }
    }

    fn configure (&mut self, config: RewardsConfig) -> StdResult<()> {
        let RewardsConfig { lp_token, ratio, threshold, cooldown } = config;
        if let Some(lp_token) = config.lp_token {
            self.storage().set(b"/lp_token", &lp_token.canonize(&self.api())?);
        }
        if let Some(ratio) = config.ratio {
            self.storage().set(b"/ratio", &ratio);
        }
        if let Some(threshold) = config.threshold {
            self.storage().set(b"/threshold", &threshold);
        }
        if let Some(cooldown) = config.cooldown {
            self.storage().set(b"/cooldown", &cooldown);
        }
    }

    fn query (&self, msg: &RewardsQuery) -> StdResult<Option<Binary>> {
        match msg {
            RewardsQuery::Status { at, address, key } => Ok(Some(self.status(at, address, key)?))
        }
    }

    fn status (&self, at: Time, address: Option<HumanAddr>, key: Option<HumanAddr>)
        -> StdResult<Option<RewardsResponse>>
    {
    }

    fn pool_info (self, at: Time) -> StdResult<Response> {
        let balance = self.load_reward_balance()?;
        let pool = Pool::new(RefCell::new(self.deps)).at(at)?.with_balance(balance);
        let pool_last_update = pool.timestamp.get()?;
        if at >= pool_last_update {
            Ok(Response::PoolInfo {
                it_is_now: at,

                lp_token: self.storage().get(b"/lp_token")
                    .humanize(&self.deps.api)?,
                reward_token: self.storage().get(b"/reward_token")?
                    .humanize(&self.deps.api)?,

                #[cfg(feature="pool_closes")]
                pool_closed:    self.close_message(&pool)?,

                pool_last_update,
                pool_lifetime:  pool.lifetime()?,
                pool_locked:    pool.locked.get()?,

                pool_claimed:   pool.claimed.get()?,
                pool_balance:   pool.balance(),

                #[cfg(feature="age_threshold")]
                pool_threshold: pool.threshold.get()?,

                #[cfg(feature="claim_cooldown")]
                pool_cooldown:  pool.cooldown.get()?,

                #[cfg(feature="pool_liquidity_ratio")]
                pool_liquid:    pool.liquidity_ratio()?,

                /* todo add balance/claimed/total in rewards token */
            })
        } else {
            Err(StdError::generic_err("this contract does not store history"))
        }
    }

    fn user_info (self, at: Time, address: HumanAddr, key: ViewingKey) -> StdResult<Response> {
    }

}

/// Reward pool
pub struct Pool <S: Storage, A: Api, Q: Querier> {
    pub deps: Rc<RefCell<Extern<S, A, Q>>>,

    now:     Option<Time>,
    balance: Option<Amount>,

    /// How much liquidity has this pool contained up to this point.
    /// On lock/unlock, if locked > 0 before the operation, this is incremented
    /// in intervals of (moments since last update * current balance)
    last_lifetime:     Field<S, A, Q, Volume>,

    /// How much liquidity is there in the whole pool right now.
    /// Incremented/decremented on lock/unlock.
    pub locked:        Field<S, A, Q, Amount>,

    /// Load the last update timestamp or default to current time
    /// (this has the useful property of keeping `elapsed` zero for strangers)
    /// When was liquidity last updated.
    /// Set to current time on lock/unlock.
    pub timestamp:     Field<S, A, Q, Time>,

    /// Rewards claimed by everyone so far.
    /// Amount of rewards already claimed
    /// Incremented on claim.
    pub claimed:       Field<S, A, Q, Amount>,

    #[cfg(feature="age_threshold")]
    /// How much the user needs to wait before they can claim for the first time.
    /// Configured on init.
    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    pub threshold:     Field<S, A, Q, Time>,

    #[cfg(feature="claim_cooldown")]
    /// How much the user must wait between claims.
    /// Configured on init.
    /// For how many blocks does the user need to wait
    /// after claiming rewards before being able to claim them again
    pub cooldown:      Field<S, A, Q, Time>,

    #[cfg(feature="global_ratio")]
    /// Ratio of liquidity provided to rewards received.
    /// Configured on init.
    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1 to make sure rewards budget is sufficient.
    pub global_ratio:  Field<S, A, Q, Ratio>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// Used to compute what portion of the time the pool was not empty.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// by the time elapsed since the last update.
    last_liquid:       Field<S, A, Q, Time>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// The first time a user locks liquidity,
    /// this is set to the current time.
    /// Used to calculate pool's liquidity ratio.
    seeded:            Field<S, A, Q, Option<Time>>,

    #[cfg(feature="pool_liquidity_ratio")]
    /// Store the moment the user is created to compute total pool existence.
    /// Set on init.
    pub created:       Field<S, A, Q, Time>,

    #[cfg(feature="pool_closes")]
    /// Whether this pool is closed
    pub closed:        Field<S, A, Q, Option<(Time, String)>>
}

impl<S: Storage, A: Api, Q: Querier> Pool<S, A, Q> {

    pub fn new (deps: Rc<RefCell<Extern<S, A, Q>>>) -> Self {
        Self {
            deps,

            now:     None,
            balance: None,

            last_lifetime: deps.field(b"/pool/lifetime")
                                  .or(Volume::zero()),

            locked:        deps.field(b"/pool/locked")
                                  .or(Amount::zero()),

            timestamp:     deps.field(b"/pool/locked")
                                  .required("missing timestamp"),

            claimed:       deps.field(b"/pool/claimed")
                                  .or(Amount::zero()),

            #[cfg(feature="age_threshold")]
            threshold:     deps.field(b"/pool/threshold")
                                  .required("missing lock threshold"),

            #[cfg(feature="claim_cooldown")]
            cooldown:      deps.field(b"/pool/cooldown")
                                  .required("missing claim cooldown"),

            #[cfg(feature="global_ratio")]
            global_ratio:  deps.field(b"/pool/global_ratio")
                                  .required("missing reward ratio"),

            #[cfg(feature="pool_liquidity_ratio")]
            last_liquid:   deps.field(b"/pool/last_liquid")
                                  .required("missing last liquid"),

            #[cfg(feature="pool_liquidity_ratio")]
            seeded:        deps.field(b"/pool/seeded")
                                  .required("nobody has locked any tokens yet"),

            #[cfg(feature="pool_liquidity_ratio")]
            created:       deps.field(b"/pool/created")
                                  .required("missing creation date"),

            #[cfg(feature="pool_closes")]
            closed:        deps.field(b"/pool/closed"),
        }
    }

    /// Return a new Pool at given time.
    /// When time is provided, some fields are redefined to have default values.
    pub fn at (self, now: Time) -> StdResult<Self> {
        Ok(Self {
            now: Some(now),

            #[cfg(feature="age_threshold")]
            threshold:   self.deps.field(b"/pool/threshold")
                                     .or(now),

            #[cfg(feature="pool_liquidity_ratio")]
            last_liquid: self.deps.field(b"/pool/last_liquid")
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

    pub fn user <'p> (mut self, address: CanonicalAddr) -> User<'p, S, A, Q> {
        User::new(&mut self, address)
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
        if let Some(seeded) = self.seeded.get()? {
            Ok(self.now()? - seeded) 
        } else {
            Err(StdError::generic_err("missing time of first lock"))
        }
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

    #[cfg(feature="pool_closes")]
    pub fn close (&mut self, message: String) -> StdResult<()> {
        self.closed.set(&Some((self.now()?, message)))
    }

    #[cfg(all(test, feature="pool_liquidity_ratio"))]
    pub fn reset_liquidity_ratio (&mut self) -> StdResult<()> {
        let existed = self.existed()?;
        self.update_locked(self.balance())?;
        self.existed.set(existed)
    }

}

/// User account
pub struct User <'p, S: Storage, A: Api, Q: Querier> {
    pub pool:    &'p mut Pool<S, A, Q>,
    pub deps:    Rc<RefCell<Extern<S, A, Q>>>,
    pub address: CanonicalAddr,

    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    pub locked:    Field<S, A, Q, Amount>,

    /// When did this user's liquidity amount last change?
    /// Set to current time on lock/unlock.
    pub timestamp: Field<S, A, Q, Time>,

    /// How much rewards has this user claimed so far?
    /// Incremented on claim.
    pub claimed:   Field<S, A, Q, Amount>,

    /// How much liquidity has this user provided since they first appeared.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// in intervals of (moments since last update * current balance)
    last_lifetime: Field<S, A, Q, Volume>,

    #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
    /// For how many units of time has this user provided liquidity
    /// On lock/unlock, if locked was > 0 before the operation,
    /// this is incremented by time elapsed since last update.
    last_present:  Field<S, A, Q, Time>,

    #[cfg(feature="user_liquidity_ratio")]
    /// For how many units of time has this user been known to the contract.
    /// Incremented on lock/unlock by time elapsed since last update.
    last_existed:  Field<S, A, Q, Time>,

    #[cfg(feature="claim_cooldown")]
    /// For how many units of time has this user provided liquidity
    /// Decremented on lock/unlock, reset to configured cooldown on claim.
    last_cooldown: Field<S, A, Q, Time>
}

impl <'p, S: Storage, A: Api, Q: Querier> User <'p, S, A, Q> {

    pub fn new (
        pool:    &'p mut Pool<S, A, Q>,
        address: CanonicalAddr
    ) -> Self {
        let deps = pool.deps;
        User {
            deps: deps.clone(),
            pool: pool,
            address,

            last_lifetime: deps.field(&concat(b"/user/lifetime/", address.as_slice()))
                                  .or(Volume::zero()),

            locked:        deps.field(&concat(b"/user/current/",  address.as_slice()))
                                  .or(Amount::zero()),

            timestamp:     deps.field(&concat(b"/user/updated/",  address.as_slice()))
                                  .or(pool.now().unwrap()),

            claimed:       deps.field(&concat(b"/user/claimed/",  address.as_slice()))
                                  .or(Amount::zero()),

            #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
            last_present:  deps.field(&concat(b"/user/present/",  address.as_slice()))
                                  .or(0u64),

            #[cfg(feature="user_liquidity_ratio")]
            last_existed:  deps.field(&concat(b"/user/existed/",  address.as_slice()))
                                  .or(0u64),

            #[cfg(feature="claim_cooldown")]
            last_cooldown: deps.field(&concat(b"/user/cooldown/", address.as_slice()))
                                  .or(pool.cooldown.get().unwrap()),
        }
    }

    // time-related getters --------------------------------------------------------------------


    #[cfg(any(feature="claim_cooldown", feature="user_liquidity_ratio"))]
    /// Time that progresses always. Used to increment existence.
    pub fn elapsed (&self) -> StdResult<Time> {
        let now = self.pool.now()?;
        if let Ok(timestamp) = self.timestamp.get() {
            if now < timestamp { // prevent replay
                return Err(StdError::generic_err("no time travel"))
            } else {
                Ok(now - timestamp)
            }
        } else {
            Ok(0 as Time)
        }
    }

    /// Time that progresses only when the user has some tokens locked.
    /// Used to increment presence and lifetime.
    pub fn elapsed_while_present (&self) -> StdResult<Time> {
        if self.locked.get()? > Amount::zero() {
            self.elapsed()
        } else {
            Ok(0 as Time)
        }
    }

    // user existence = time since this user first locked tokens -------------------------------

    #[cfg(feature="user_liquidity_ratio")]
    /// Up-to-date time for which the user has existed
    pub fn existed (&self) -> StdResult<Time> {
        Ok(self.last_existed.get()? + self.elapsed()?)
    }

    #[cfg(feature="user_liquidity_ratio")]
    pub fn liquidity_ratio (&self) -> StdResult<Amount> {
        Ok(
            Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(self.present()?, self.existed()?)?
            .low_u128().into()
        )
    }

    // user presence = time the user has had >0 LP tokens locked in the pool -------------------

    #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
    /// Up-to-date time for which the user has provided liquidity
    pub fn present (&self) -> StdResult<Time> {
        Ok(self.last_present.get()? + self.elapsed_while_present()?)
    }

    // cooldown - reset on claim, decremented towards 0 as time advances -----------------------

    #[cfg(feature="claim_cooldown")]
    pub fn cooldown (&self) -> StdResult<Time> {
        #[cfg(feature="pool_closes")]
        if self.pool.closed.get()?.is_some() {
            return Ok(0u64)
        }
        Ok(Time::saturating_sub(
            self.last_cooldown.get()?,
            self.elapsed()?
        ))
    }

    // lp-related getters ----------------------------------------------------------------------

    pub fn lifetime (&self) -> StdResult<Volume> {
        tally(
            self.last_lifetime.get()?,
            self.elapsed_while_present()?,
            self.locked.get()?
        )
    }

    // reward-related getters ------------------------------------------------------------------

    // After first locking LP tokens, users must reach a configurable age threshold,
    // i.e. keep LP tokens locked for at least X blocks. During that time, their portion of
    // the total liquidity ever provided increases.
    //
    // The total reward for an user with an age under the threshold is zero.
    //
    // The total reward for a user with an age above the threshold is
    // (claimed_rewards + budget) * user_lifetime_liquidity / pool_lifetime_liquidity
    //
    // Since a user's total reward can diminish, it may happen that the amount claimed
    // by a user so far is larger than the current total reward for that user.
    // In that case the user's claimable amount remains zero until they unlock more
    // rewards than they've already claimed.
    //
    // Since a user's total reward can diminish, it may happen that the amount remaining
    // in the pool after a user has claimed is insufficient to pay out the next user's reward.
    // In that case, https://google.github.io/filament/webgl/suzanne.html

    pub fn share (&self, basis: u128) -> StdResult<Volume> {
        let share = Volume::from(basis);

        // reduce lifetime by normal lifetime ratio
        let share = share.diminish_or_zero(self.lifetime()?, self.pool.lifetime()?)?;

        // reduce lifetime by liquidity ratio
        #[cfg(feature="user_liquidity_ratio")]
        let share = share.diminish_or_zero(self.present()?, self.existed()?)?;

        Ok(share)
    }

    pub fn earned (&self) -> StdResult<Amount> {
        let mut budget = Amount::from(self.pool.budget()?);

        #[cfg(feature="pool_liquidity_ratio")] {
            budget = budget.diminish_or_zero(self.pool.liquid()?, self.pool.existed()?)?;
        }

        #[cfg(feature="global_ratio")] {
            let ratio = self.pool.global_ratio.get()?;
            budget = budget.diminish_or_zero(ratio.0, ratio.1)?
        }

        Ok(self.share(budget.u128())?.low_u128().into())
    }

    pub fn claimable (&self) -> StdResult<Amount> {
        #[cfg(feature="age_threshold")]
        // can only claim after age threshold
        if self.present()? < self.pool.threshold.get()? {
            return Ok(Amount::zero())
        }

        // can only claim if earned something
        let earned = self.earned()?;
        if earned == Amount::zero() {
            return Ok(Amount::zero())
        }

        // can only claim if earned > claimed
        let claimed = self.claimed.get()?;
        if earned <= claimed {
            return Ok(Amount::zero())
        }

        // can only claim if the pool has balance
        let balance = self.pool.balance();
        let claimable = (earned - claimed)?;
        // not possible to claim more than the remaining pool balance
        if claimable > balance {
            Ok(balance)
        } else {
            Ok(claimable)
        }
    }

    pub fn lock_tokens (&mut self, increment: Amount) -> StdResult<Amount> {
        self.update(
            self.locked.get()? + increment,
            self.pool.locked.get()? + increment.into()
        )?;
        // Return the amount to be transferred from the user to the contract
        Ok(increment)
    }

    pub fn retrieve_tokens (&mut self, decrement: Amount) -> StdResult<Amount> {
        // Must have enough locked to retrieve
        let locked = self.locked.get()?;
        if locked < decrement {
            return Err(StdError::generic_err(format!("not enough locked ({} < {})", locked, decrement)))
        }
        self.update(
            (locked - decrement)?,
            (self.pool.locked.get()? - decrement.into())?
        )?;
        // Return the amount to be transferred back to the user
        Ok(decrement)
    }

    pub fn claim_reward (&mut self) -> StdResult<Amount> {

        // If user must wait before first claim, enforce that here.
        #[cfg(feature="age_threshold")]
        enforce_cooldown(self.present()?, self.pool.threshold.get()?)?;

        // If user must wait between claims, enforce that here.
        #[cfg(feature="claim_cooldown")]
        enforce_cooldown(0, self.cooldown()?)?;

        // See if there is some unclaimed reward amount:
        let claimable = self.claimable()?;
        if claimable == Amount::zero() {
            return Err(StdError::generic_err(
                "You've already received as much as your share of the reward pool allows. \
                Keep your liquidity tokens locked and wait for more rewards to be vested, \
                and/or lock more liquidity tokens to grow your share of the reward pool."
            ))
        }

        // Now we need the user's liquidity token balance for two things:
        let locked = self.locked.get()?;

        // Update user timestamp, and the things synced to it.
        self.update(locked, self.pool.locked.get()?)?;

        // Update how much has been claimed
        self.increment_claimed(claimable)?;

        // ...and, optionally, reset the cooldown so that
        // the user has to wait before claiming again)
        #[cfg(feature="claim_cooldown")]
        self.last_cooldown.set(&self.pool.cooldown.get()?)?;

        // 2. Optionally, reset the user's `lifetime` and `share` if they have currently
        //    0 tokens locked. The intent is for this to be the user's last reward claim
        //    after they've left the pool completely. If they provide exactly 0 liquidity
        //    at some point, when they come back they have to start over, which is OK
        //    because they can then start claiming rewards immediately, without waiting
        //    for threshold, only cooldown.
        #[cfg(feature="selective_memory")]
        if locked == Amount::zero() {
            self.last_lifetime.set(&Volume::zero())?;
            self.claimed.set(&Amount::zero())?;
        }

        // Return the amount that the contract will send to the user
        Ok(claimable)

    }

    fn increment_claimed (&mut self, reward: Amount) -> StdResult<()> {
        self.pool.increment_claimed(reward)?;
        self.claimed.set(&(self.claimed.get()? + reward))?;
        Ok(())
    }

    #[cfg(all(test, feature="user_liquidity_ratio"))]
    pub fn reset_liquidity_ratio (&self) -> StdResult<()> {
        let existed = self.existed()?;
        self.update(self.locked()?, self.pool.locked.get()?)?;
        self.present.set(existed)?;
        Ok(())
    }

    /// Commit rolling values to deps
    fn update (&mut self, user_locked: Amount, pool_locked: Amount) -> StdResult<&mut Self> {
        // Prevent replay
        let now = self.pool.now()?;
        if let Ok(timestamp) = self.timestamp.get() {
            if timestamp > now {
                return Err(StdError::generic_err("no time travel"))
            }
        }

        // Increment existence
        #[cfg(feature="user_liquidity_ratio")]
        self.last_existed.set(&self.existed()?)?;

        // Increment presence if user has currently locked tokens
        #[cfg(any(feature="age_threshold", feature="user_liquidity_ratio"))]
        self.last_present.set(&self.present()?)?;

        // Cooldown is calculated since the timestamp.
        // Since we'll be updating the timestamp, commit the current cooldown
        #[cfg(feature="claim_cooldown")]
        self.last_cooldown.set(&self.cooldown()?)?;

        // Always increment lifetime
        self.last_lifetime.set(&self.lifetime()?)?;

        // Set user's time of last update to now
        self.timestamp.set(&now)?;

        // Update amount locked
        self.locked.set(&user_locked)?;

        // Update total amount locked in pool
        self.pool.update_locked(pool_locked)?;

        Ok(self)
    }

}

#[cfg(any(feature="claim_cooldown", feature="age_threshold"))]
fn enforce_cooldown (elapsed: Time, cooldown: Time) -> StdResult<()> {
    if elapsed >= cooldown {
        Ok(())
    } else {
        Err(StdError::generic_err(format!("lock tokens for {} more seconds to be eligible", cooldown - elapsed)))
    }
}

pub trait FieldFactory <S: Storage + AsRef<S>, A: Api, Q: Querier> {
    fn field <V> (&self, key: &[u8]) -> Field<S, A, Q, V>;
}

impl<S: Storage + AsRef<S>, A: Api, Q: Querier> FieldFactory<S, A, Q>
for Rc<RefCell<Extern<S, A, Q>>> {
    fn field <V> (&self, key: &[u8]) -> Field<S, A, Q, V> {
        Field::new(self.clone(), key.to_vec())
    }
}

pub struct Field <S: Storage, A: Api, Q: Querier, V> {
    deps:     Rc<RefCell<Extern<S, A, Q>>>,
    key:      Vec<u8>,
    value:    Option<V>,
    default:  Option<V>,
    required: Option<String>
}

impl<S: Storage, A: Api, Q: Querier, V> Field<S, A, Q, V> {

    /// Define a new field
    pub fn new (deps: Extern<S, A, Q>, key: Vec<u8>) -> Self {
        Self { deps, key, value: None, default: None, required: None }
    }

    /// Define a default value
    pub fn or (mut self, default: V) -> Self {
        self.default = Some(default);
        self
    }

    /// Define an error message for missing value with no default
    pub fn required (mut self, message: &str) -> Self {
        self.required = Some(message.to_string());
        self
    }

}

impl<S: Storage, A: Api, Q: Querier, V: DeserializeOwned>
Field<S, A, Q, V> {

    pub fn get (mut self) -> StdResult<V> {
        if let Some(value) = self.value {
            Ok(value)
        } else if let Some(data) = self.deps.borrow().storage.get(&self.key) {
            let value = from_slice(&data)?;
            self.value = Some(value);
            Ok(value)
        } else if let Some(default) = self.default {
            self.value = Some(default);
            Ok(default)
        } else if let Some(message) = self.required {
            Err(StdError::generic_err(&message))
        } else {
            Err(StdError::generic_err("not in storage"))
        }
    }

}

impl<S: Storage, A: Api, Q: Querier, V: Serialize>
Field<S, A, Q, V> {

    pub fn set (mut self, value: &V) -> StdResult<()> {
        self.deps.borrow_mut().storage.set(&self.key, &to_vec(value)?);
        self.value = Some(*value);
        Ok(())
    }

}
