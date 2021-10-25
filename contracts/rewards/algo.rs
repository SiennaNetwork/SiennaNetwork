use fadroma::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use crate::{core::*, math::*, auth::Auth};

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct RewardsInit {
    reward_token: ContractLink<HumanAddr>,
    viewing_key:  ViewingKey,
    config:       RewardsConfig
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsHandle {
    Configure(RewardsConfig),
    ClosePool { message: String },
    Lock      { amount: Amount },
    Retrieve  { amount: Amount },
    Claim     {}
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct RewardsConfig {
    lp_token:  Option<ContractLink<HumanAddr>>,
    ratio:     Option<(Uint128, Uint128)>,
    threshold: Option<Time>,
    cooldown:  Option<Time>
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsQuery {
    Status {
        at:      Time,
        address: Option<HumanAddr>,
        key:     Option<String>
    }
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsResponse {
    Status {
        time:         Time,
        reward_token: ContractLink<HumanAddr>,
        config:       RewardsConfig,
        pool:         PoolStatus,
        user:         Option<UserStatus>
    }
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct PoolStatus {
    /// Load the last update timestamp or default to current time
    /// (this has the useful property of keeping `elapsed` zero for strangers)
    /// When was liquidity last updated.
    /// Set to current time on lock/unlock.
    last_update: Time,

    /// How much liquidity has this pool contained up to this point.
    /// On lock/unlock, if locked > 0 before the operation, this is incremented
    /// in intervals of (moments since last update * current balance)
    lifetime:    Volume,

    /// How much liquidity is there in the whole pool right now.
    /// Incremented/decremented on lock/unlock.
    locked:      Amount,
    
    /// Whether this pool is closed
    closed:      Option<String>,

    balance:     Amount,

    /// Rewards claimed by everyone so far.
    /// Amount of rewards already claimed
    /// Incremented on claim.
    claimed:     Amount,

    vested:      Amount,

    /// How much the user needs to wait before they can claim for the first time.
    /// Configured on init.
    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    threshold:   Time,

    /// How much the user must wait between claims.
    /// Configured on init.
    /// For how many blocks does the user need to wait
    /// after claiming rewards before being able to claim them again
    cooldown:    Time,

    /// Used to compute what portion of the time the pool was not empty.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// by the time elapsed since the last update.
    liquid:      Amount
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct UserStatus {
    /// When did this user's liquidity amount last change?
    /// Set to current time on lock/unlock.
    last_update: Time,

    /// How much liquidity has this user provided since they first appeared.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// in intervals of (moments since last update * current balance)
    lifetime:    Volume,

    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    locked:      Amount,

    share:       Amount,

    earned:      Amount,

    /// How much rewards has this user claimed so far?
    /// Incremented on claim.
    claimed:     Amount,

    claimable:   Amount,

    /// For how many units of time has this user been known to the contract.
    /// Incremented on lock/unlock by time elapsed since last update.
    age:         Time,

    /// How many units of time remain until the user can claim again.
    /// Decremented on lock/unlock, reset to configured cooldown on claim.
    cooldown:    Time

    reason: Option<String>
}

pub mod keys {
    pub mod pool {
        pub const CLAIMED:      &[u8] = b"/pool/claimed";
        pub const CLOSED:       &[u8] = b"/pool/closed";
        pub const COOLDOWN:     &[u8] = b"/pool/cooldown";
        pub const CREATED:      &[u8] = b"/pool/created";
        pub const LIFETIME:     &[u8] = b"/pool/lifetime";
        pub const LIQUID:       &[u8] = b"/pool/not_empty";
        pub const LOCKED:       &[u8] = b"/pool/balance";
        pub const LP_TOKEN:     &[u8] = b"/pool/lp_token";
        pub const RATIO:        &[u8] = b"/pool/ratio";
        pub const REWARD_TOKEN: &[u8] = b"/pool/reward_token";
        pub const REWARD_VK:    &[u8] = b"/pool/reward_vk";
        pub const SEEDED:       &[u8] = b"/pool/created";
        pub const SELF:         &[u8] = b"/pool/self";
        pub const THRESHOLD:    &[u8] = b"/pool/threshold";
        pub const TIMESTAMP:    &[u8] = b"/pool/updated";
    }
    pub mod user {
        pub const CLAIMED:   &[u8] = b"/user/claimed/";
        pub const COOLDOWN:  &[u8] = b"/user/cooldown/";
        pub const EXISTED:   &[u8] = b"/user/existed/";
        pub const LIFETIME:  &[u8] = b"/user/lifetime/";
        pub const LOCKED:    &[u8] = b"/user/current/";
        pub const PRESENT:   &[u8] = b"/user/present/";
        pub const TIMESTAMP: &[u8] = b"/user/updated/";
    }
}

pub trait Rewards<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    fn init (&self, env: &Env, msg: &RewardsInit) -> StdResult<()> {
        self.set(keys::pool::REWARD_TOKEN, &msg.reward_token.canonize(&self.api()));
        self.set(keys::pool::REWARD_VK,    &msg.viewing_key);
        self.set(keys::pool::SELF, &ContractLink {
            address:   env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        }.canonize(&self.api())?);
        self.handle_configure(&RewardsConfig {
            lp_token:  msg.config.lp_token,
            ratio:     Some(msg.config.ratio.unwrap_or((1u128.into(), 1u128.into()))),
            threshold: Some(msg.config.threshold.unwrap_or(DAY)),
            cooldown:  Some(msg.config.cooldown.unwrap_or(DAY))
        });
        Ok(())
    }

    fn handle (&self, env: Env, msg: RewardsHandle) -> StdResult<HandleResponse> {
        match msg {
            RewardsHandle::Configure(config) =>
                self.handle_configure(&config),
            RewardsHandle::Lock { amount } =>
                self.handle_lock(env.message.sender, amount),
            RewardsHandle::Retrieve { amount } =>
                self.handle_retrieve(env.message.sender, amount),
            RewardsHandle::Claim {} =>
                self.handle_claim(env.message.sender),
            RewardsHandle::ClosePool { message } =>
                self.handle_close_pool(env.message.sender, message),
        }
    }

    fn handle_configure (&self, config: &RewardsConfig) -> StdResult<HandleResponse> {
        let RewardsConfig { lp_token, ratio, threshold, cooldown } = config;
        if let Some(lp_token) = config.lp_token {
            self.set(keys::pool::LP_TOKEN,  &lp_token.canonize(&self.api())?);
        }
        if let Some(ratio) = config.ratio {
            self.set(keys::pool::RATIO,     &ratio);
        }
        if let Some(threshold) = config.threshold {
            self.set(keys::pool::THRESHOLD, &threshold);
        }
        if let Some(cooldown) = config.cooldown {
            self.set(keys::pool::COOLDOWN,  &cooldown);
        }
        Ok(HandleResponse::default())
    }

    fn handle_lock (&self, env: Env, amount: Uint128) -> StdResult<HandleResponse> {
        let address = env.message.sender;

        // Increment user and pool liquidity
        let user_locked = self.get_ns(keys::user::LOCKED, self.canonize(address)?.as_slice())? + amount;
        let pool_locked = self.get(keys::pool::LOCKED)? + amount;
        self.update(user_locked, pool_locked)?;

        // Transfer liquidity provision tokens from the user to the contract
        let lp_token = ISnip20::attach(&self.humanize(self.get(keys::pool::LP_TOKEN)?)?);
        let transfer = lp_token.transfer_from(&address, &env.contract.address, amount)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})
    }

    fn handle_retrieve (&self, env: Env, address: HumanAddr, amount: Uint128) -> StdResult<HandleResponse> {
        let address = env.message.sender;

        // Must have enough locked to retrieve
        let user_locked = self.get_ns(keys::user::LOCKED, address.as_slice())?;
        if user_locked < amount {
            return Err(StdError::generic_err(
                format!("not enough locked ({} < {})", user_locked, amount)
            ))
        }

        // Decrement user and pool liquidity
        let user_locked = user_locked - amount;
        let pool_locked = self.get(keys::pool::LOCKED)? - amount;
        self.update(user_locked, pool_locked)?;

        // Transfer liquidity provision tokens from the contract to the user
        let lp_token = ISnip20::attach(&self.humanize(self.get(keys::pool::LP_TOKEN)?)?);
        let transfer = lp_token.transfer(address, amount)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})
    }

    fn handle_claim (&self, address: HumanAddr) -> StdResult<HandleResponse> {
        // If user must wait before first claim, enforce that here.
        enforce_cooldown(self.present(&*storage)?, self.pool.threshold.get(&*storage)?)?;

        // If user must wait between claims, enforce that here.
        enforce_cooldown(0, self.cooldown(&*storage)?)?;

        // See if there is some unclaimed reward amount:
        let claimable = self.claimable(&*storage)?;
        if claimable == Amount::zero() {
            return Err(StdError::generic_err(
                "You've already received as much as your share of the reward pool allows. \
                Keep your liquidity tokens locked and wait for more rewards to be vested, \
                and/or lock more liquidity tokens to grow your share of the reward pool."
            ))
        }

        // Now we need the user's liquidity token balance for two things:
        let locked = self.locked.get(&*storage)?;

        // Update user timestamp, and the things synced to it.
        self.update(storage, locked, self.pool.locked.get(&*storage)?)?;

        // Update how much has been claimed
        self.increment_claimed(storage, claimable)?;

        // ...and, optionally, reset the cooldown so that
        // the user has to wait before claiming again)
        self.last_cooldown.set(storage, self.pool.cooldown.get(storage)?)?;

        // 2. Optionally, reset the user's `lifetime` and `share` if they have currently
        //    0 tokens locked. The intent is for this to be the user's last reward claim
        //    after they've left the pool completely. If they provide exactly 0 liquidity
        //    at some point, when they come back they have to start over, which is OK
        //    because they can then start claiming rewards immediately, without waiting
        //    for threshold, only cooldown.
        if locked == Amount::zero() {
            self.last_lifetime.set(storage, Volume::zero())?;
            self.claimed.set(storage, Amount::zero())?;
        }

        // Return the amount that the contract will send to the user
        Ok(claimable)
    }

    fn handle_close_pool (&self, address: HumanAddr, message: String) -> StdResult<HandleResponse> {
        self.closed.set(storage, Some((self.now(&*storage)?, message)))
        Ok(HandleResponse::default())
    }

    /// Commit rolling values to storage
    fn update (self, user_locked: Amount, pool_locked: Amount) -> StdResult<()> {
        // Prevent replay
        let now = self.pool.now(&*storage)?;
        if let Ok(timestamp) = self.timestamp.get(storage) {
            if timestamp > now {
                return Err(StdError::generic_err("no time travel"))
            }
        }

        // Increment existence
        self.last_existed.set(storage, self.existed(&*storage)?)?;

        // Increment presence if user has currently locked tokens
        self.last_present.set(storage, self.present(&*storage)?)?;

        // Cooldown is calculated since the timestamp.
        // Since we'll be updating the timestamp, commit the current cooldown
        self.last_cooldown.set(storage, self.cooldown(&*storage)?)?;

        // Always increment lifetime
        self.last_lifetime.set(storage, self.lifetime(&*storage)?)?;

        // Set user's time of last update to now
        self.timestamp.set(storage, now)?;

        // Update amount locked
        self.locked.set(storage, user_locked)?;

        // Update total amount locked in pool
        self.pool.update_locked(storage, pool_locked)?;

        Ok(())
    }

    fn query (&self, msg: RewardsQuery) -> StdResult<RewardsResponse> {
        match msg {
            RewardsQuery::Status { at, address, key } =>
                self.query_status(at, address, key)
        }
    }

    fn query_status (
        &self,
        at:      Time,
        address: Option<HumanAddr>,
        key:     Option<String>
    ) -> StdResult<RewardsResponse> {
        if address.is_some() && key.is_none() {
            return Err(StdError::generic_err("no viewing key"))
        }
        let pool = self.get_pool_status(at)?;
        if at < pool.last_update {
            return Err(StdError::generic_err("no history"))
        }
        let user = if let (Some(address), Some(key)) = (address, key) {
            Some(self.get_user_status(&pool, self.canonize(address)?, key)?)
        } else {
            None
        };
        Ok(RewardsResponse::Status {
            time: at,
            reward_token: self.humanize(self.get(keys::pool::REWARD_TOKEN)?)?,
            config:       self.get_config(&pool)?,
            pool,
            user 
        })

    }

    fn get_config (&self, pool: &Pool) -> StdResult<RewardsConfig> {
        Ok(RewardsConfig {
            lp_token:  None,
            ratio:     None,
            threshold: None,
            cooldown:  None
        })
    }

    fn get_pool_status (&self, now: Time) -> StdResult<PoolStatus> {
        let last_update = self.get(keys::pool::TIMESTAMP)?;
        if now < last_update {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed  = now - last_update;
        let locked   = self.get(keys::pool::LOCKED)?;
        let lifetime = tally(self.get(keys::pool::LIFETIME)?, elapsed, locked)?;

        let balance = self.load_reward_balance()?;
        let claimed = self.get(keys::pool::CLAIMED)?;
        let vested  = claimed + balance;

        let liquid          = Volume::zero();
        let existed         = Volume::zero();
        let liquidity_ratio = Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(liquid, existed)?
            .low_u128().into();

        let global_ratio = self.get(keys::pool::RATIO);

        let closed = self.get(keys::pool::CLOSED)?;

        Ok(PoolStatus {
            now,

            last_update,
            locked,
            lifetime,

            balance,
            claimed,
            vested,

            liquid: liquidity_ratio,
            global_ratio,

            closed,
        })
    }


    fn get_user_status (
        &self, pool: &PoolStatus, address: CanonicalAddr, key: String
    ) -> StdResult<UserStatus> {
        let last_update = self.get_ns(b"/user/timestamp", address.as_slice())?;
        if pool.now < last_update {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed  = pool.now - last_update;
        let locked   = self.get_ns(keys::user::LOCKED, address.as_slice())?;
        let lifetime = tally(self.get_ns(keys::user::LIFETIME, address.as_slice())?, elapsed, locked)?;

        let existed = self.get_ns(keys::user::EXISTED, address.as_slice())? +
            elapsed;
        let present = self.get_ns(keys::user::PRESENT, address.as_slice())? +
            if locked > Amount::zero() { elapsed } else { 0 };

        let share = Volume::from(basis)
            .diminish_or_zero(lifetime, pool.lifetime)?
            .diminish_or_zero(present, existed)?;
        let earned = Amount::from(pool.budget)
            .diminish_or_zero(pool.liquid, pool.existed)?
            .diminish_or_zero(pool.global_ratio.0, pool.global_ratio.1)?;
        let claimed = self.get_ns(keys::user::CLAIMED, address.as_slice())?;

        let mut reason: Option<&str> = None;
        let claimable = if present < pool.threshold { // 
            reason = Some("can only claim after age threshold");
            Amount::zero()
        } else if earned == Amount::zero() {
            reason = Some("can only claim positive earnings");
            Amount::zero()
        } else if earned <= claimed {
            reason = Some("can only claim if earned > claimed");
            Amount::zero()
        } else {
            let claimable = (earned - claimed)?;
            if claimable > pool.balance {
                reason = Some("can't claim more than the remaining pool balance");
                pool.balance
            } else {
                claimable
            }
        };

        let age = self.get_ns(b"/user/age", address.as_slice())?;
        let cooldown = self.get_ns(b"/user/cooldown", address.as_slice())?;

        Ok(UserStatus {
            last_update,
            locked,
            lifetime,

            share,
            earned,
            claimed,
            claimable,

            age,
            cooldown,

            reason: reason.map(|x|x.to_string())
        })
    }

    fn load_reward_balance (&self) -> StdResult<Amount> {
        let self_link   = self.humanize(self.get(keys::pool::SELF)?)?;
        let reward_link = self.humanize(self.get(keys::pool::REWARD_TOKEN)?)?;
        let reward_vk   = self.get::<ViewingKey>(keys::pool::REWARD_VK)?.0;
        let lp_link     = self.humanize(self.get(keys::pool::LP_TOKEN)?)?;
        let reward_token = ISnip20::attach(&reward_link).query(&(self.querier()));
        let mut reward_balance = reward_token.balance(&self_link, &reward_vk)?;
        if reward_link == lp_link {
            let pool = Pool::new();
            let lp_balance = pool.locked.get(&self.storage())?;
            reward_balance = (reward_balance - lp_balance)?;
        }
        Ok(reward_balance)
    }

}

/// Reward pool
pub struct Pool {
    now:     Option<Time>,
    balance: Option<Amount>,

    last_lifetime:     Field<Volume>,

    pub locked:        Field<Amount>,

    pub timestamp:     Field<Time>,

    pub claimed:       Field<Amount>,

    pub threshold:     Field<Time>,

    pub cooldown:      Field<Time>,

    /// Ratio of liquidity provided to rewards received.
    /// Configured on init.
    /// Ratio between share of liquidity provided and amount of reward
    /// Should be <= 1 to make sure rewards budget is sufficient.
    pub global_ratio:  Field<Ratio>,

    last_liquid:       Field<Time>,

    /// The first time a user locks liquidity,
    /// this is set to the current time.
    /// Used to calculate pool's liquidity ratio.
    seeded:            Field<Option<Time>>,

    /// Store the moment the user is created to compute total pool existence.
    /// Set on init.
    pub created:       Field<Time>,

    pub closed:        Field<Option<(Time, String)>>
}

impl Pool {

    pub fn new () -> Self {
        Self {
            now:     None,
            balance: None,

            last_lifetime: Field::new(b"/pool/lifetime")
                .or(Volume::zero()),

            locked:        Field::new(b"/pool/locked")
                                  .or(Amount::zero()),

            timestamp:     Field::new(b"/pool/timestamp")
                                  .required("missing timestamp"),

            claimed:       Field::new(b"/pool/claimed")
                                  .or(Amount::zero()),

            threshold:     Field::new(b"/pool/threshold")
                                  .required("missing lock threshold"),

            cooldown:      Field::new(b"/pool/cooldown")
                                  .required("missing claim cooldown"),

            global_ratio:  Field::new(b"/pool/global_ratio")
                                  .required("missing reward ratio"),

            last_liquid:   Field::new(b"/pool/last_liquid")
                                  .required("missing last liquid"),

            seeded:        Field::new(b"/pool/seeded")
                                  .required("nobody has locked any tokens yet"),

            created:       Field::new(b"/pool/created")
                                  .required("missing creation date"),

            closed:        Field::new(b"/pool/closed"),

        }
    }

    /// Return a new Pool at given time.
    /// When time is provided, some fields are redefined to have default values.
    pub fn at (self, storage: &impl Storage, now: Time) -> StdResult<Self> {
        Ok(Self {
            now: Some(now),

            threshold:   Field::new(b"/pool/threshold")
                                .or(now),

            last_liquid: Field::new(b"/pool/last_liquid")
                                .or(self.existed(storage)?),

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

    pub fn user (self, storage: &impl Storage, address: CanonicalAddr) -> User {
        User::new(storage, self, address)
    }

    /// Get the time since last update (0 if no last update)
    pub fn elapsed (&self, storage: &impl Storage) -> StdResult<Time> {
        Ok(self.now(storage)? - self.timestamp.get(storage)?)
    }

    /// Get the current time or fail
    pub fn now (&self, storage: &impl Storage) -> StdResult<Time> {
        let mut now = self.now.ok_or(StdError::generic_err("current time not set"))?;

        // stop time when closing pool
        if let Some((t_closed, _)) = self.closed.get(storage)? {
            if now < t_closed {
                return Err(StdError::generic_err("no time travel"));
            }
            now = t_closed
        }

        Ok(now)
    }

    /// The total liquidity ever contained in this pool.
    pub fn lifetime (&self, storage: &impl Storage) -> StdResult<Volume> {
        tally(
            self.last_lifetime.get(storage)?,
            self.elapsed(storage)?,
            self.locked.get(storage)?
        )
    }

    /// The full reward budget = rewards claimed + current balance of this contract in reward token
    pub fn budget (&self, storage: &impl Storage) -> StdResult<Amount> {
        Ok(self.claimed.get(storage)? + self.balance())
    }

    /// Current balance in reward token, or zero.
    pub fn balance (&self) -> Amount {
        self.balance.unwrap_or(Amount::zero())
    }

    /// Time for which the pool was not empty.
    pub fn liquid (&self, storage: &impl Storage) -> StdResult<Time> {
        let mut liquid = self.last_liquid.get(storage)?;
        if self.locked.get(storage)? > Amount::zero() {
            liquid += self.elapsed(storage)?
        }
        Ok(liquid)
    }

    pub fn liquidity_ratio (&self, storage: &impl Storage) -> StdResult<Amount> {
        Ok(Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(self.liquid(storage)?, self.existed(storage)?)?
            .low_u128().into()
        )
    }

    pub fn existed (&self, storage: &impl Storage) -> StdResult<Time> {
        if let Some(seeded) = self.seeded.get(storage)? {
            Ok(self.now(storage)? - seeded) 
        } else {
            Err(StdError::generic_err("missing time of first lock"))
        }
    }

    /// Increment the total amount of claimed rewards for all users.
    pub fn increment_claimed (mut self, storage: &mut impl Storage, reward: Amount) -> StdResult<()> {
        self.claimed.set(storage, (self.claimed.get(&*storage)? + reward))
    }

    /// Every time the amount of tokens locked in the pool is updated,
    /// the pool's lifetime liquidity is tallied and and the timestamp is updated.
    /// This is the only user-triggered input to the pool.
    pub fn update_locked (&self, storage: &mut impl Storage, balance: Amount) -> StdResult<()> {
        // If this is the first time someone is locking tokens in this pool.
        // store the timestamp. This is used to start the pool liquidity ratio
        // calculation from the time of first lock instead of from the time
        // of contract init.
        // * Using is_none here fails type inference.
        // * Zero timestamp is special-cased - apparently cosmwasm 0.10
        //   can't tell the difference between None and the 1970s.
        match self.seeded.get(&*storage)? as Option<Time> {
            None => {
                self.seeded.set(storage, self.now)?;
            },
            Some(0) => {
                return Err(StdError::generic_err("you jivin' yet?"));
            },
            _ => {}
        };

        let lifetime = self.lifetime(&*storage)?;
        let now      = self.now(&*storage)?;
        let liquid   = self.liquid(&*storage)?;
        self.last_liquid.set(storage, liquid)?;
        self.last_lifetime.set(storage, lifetime)?;
        self.locked.set(storage, balance)?;
        self.timestamp.set(storage, now)?;

        Ok(())
    }

    pub fn close (&self, storage: &mut impl Storage, message: String) -> StdResult<()> {
    }

    pub fn reset_liquidity_ratio (&self) -> StdResult<()> {
        //let existed = self.existed()?;
        //self.update_locked(self.balance())?;
        //self.existed.set(existed)
        unimplemented!("what needs to happen here")
    }

}

/// User account
pub struct User {
    pub pool:    Pool,
    pub address: CanonicalAddr,

    pub locked:    Field<Amount>,

    pub timestamp: Field<Time>,

    pub claimed:   Field<Amount>,

    last_lifetime: Field<Volume>,

    /// For how many units of time has this user provided liquidity
    /// On lock/unlock, if locked was > 0 before the operation,
    /// this is incremented by time elapsed since last update.
    last_present:  Field<Time>,

    last_existed:  Field<Time>,

    last_cooldown: Field<Time>
}

impl User {

    pub fn new (storage: &impl Storage, pool: Pool, address: CanonicalAddr) -> Self {
        User {
            pool,
            address,

            last_lifetime: Field::new(&concat(b"/user/lifetime/", address.as_slice()))
                                  .or(Volume::zero()),

            locked:        Field::new(&concat(b"/user/current/",  address.as_slice()))
                                  .or(Amount::zero()),

            timestamp:     Field::new(&concat(b"/user/updated/",  address.as_slice()))
                                  .or(pool.now(storage).unwrap()),

            claimed:       Field::new(&concat(b"/user/claimed/",  address.as_slice()))
                                  .or(Amount::zero()),

            last_present:  Field::new(&concat(b"/user/present/",  address.as_slice()))
                                  .or(0u64),

            last_existed:  Field::new(&concat(b"/user/existed/",  address.as_slice()))
                                  .or(0u64),

            last_cooldown: Field::new(&concat(b"/user/cooldown/", address.as_slice()))
                                  .or(pool.cooldown.get(storage).unwrap()),
        }
    }

    // time-related getters --------------------------------------------------------------------


    /// Time that progresses always. Used to increment existence.
    pub fn elapsed (&self, storage: &impl Storage) -> StdResult<Time> {
        let now = self.pool.now(storage)?;
        if let Ok(timestamp) = self.timestamp.get(storage) {
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
    pub fn elapsed_while_present (&self, storage: &impl Storage) -> StdResult<Time> {
        if self.locked.get(storage)? > Amount::zero() {
            self.elapsed(storage)
        } else {
            Ok(0 as Time)
        }
    }

    // user existence = time since this user first locked tokens -------------------------------

    /// Up-to-date time for which the user has existed
    pub fn existed (&self, storage: &impl Storage) -> StdResult<Time> {
        Ok(self.last_existed.get(storage)? + self.elapsed(storage)?)
    }

    pub fn liquidity_ratio (&self, storage: &impl Storage) -> StdResult<Amount> {
        Ok(
            Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(
                self.present(storage)?,
                self.existed(storage)?)?
            .low_u128().into()
        )
    }

    // user presence = time the user has had >0 LP tokens locked in the pool -------------------

    /// Up-to-date time for which the user has provided liquidity
    pub fn present (&self, storage: &impl Storage) -> StdResult<Time> {
        Ok(self.last_present.get(storage)? + self.elapsed_while_present(storage)?)
    }

    // cooldown - reset on claim, decremented towards 0 as time advances -----------------------

    pub fn cooldown (&self, storage: &impl Storage) -> StdResult<Time> {
        if self.pool.closed.get(storage)?.is_some() {
            return Ok(0u64)
        }
        Ok(Time::saturating_sub(
            self.last_cooldown.get(storage)?,
            self.elapsed(storage)?
        ))
    }

    // lp-related getters ----------------------------------------------------------------------

    pub fn lifetime (&self, storage: &impl Storage) -> StdResult<Volume> {
        tally(
            self.last_lifetime.get(storage)?,
            self.elapsed_while_present(storage)?,
            self.locked.get(storage)?
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

    pub fn share (self, storage: &impl Storage, basis: u128) -> StdResult<Volume> {
        let share = Volume::from(basis);

        // reduce lifetime by normal lifetime ratio
        let share = share.diminish_or_zero(self.lifetime(storage)?, self.pool.lifetime(storage)?)?;

        // reduce lifetime by liquidity ratio
        let share = share.diminish_or_zero(self.present(storage)?, self.existed(storage)?)?;

        Ok(share)
    }

    pub fn earned (self, storage: &impl Storage) -> StdResult<Amount> {
        let mut budget = Amount::from(self.pool.budget(storage)?);

        //WTF happened here
            //budget = budget.diminish_or_zero(self.pool.liquid()?, self.pool.existed()?)?;
        //}

            //let ratio = self.pool.global_ratio.get()?;
            //budget = budget.diminish_or_zero(ratio.0, ratio.1)?
        //}

        Ok(self.share(storage, budget.u128())?.low_u128().into())
    }

    pub fn claimable (self, storage: &impl Storage) -> StdResult<Amount> {
        // can only claim after age threshold
        if self.present(storage)? < self.pool.threshold.get(storage)? {
            return Ok(Amount::zero())
        }

        // can only claim if earned something
        let earned = self.earned(storage)?;
        if earned == Amount::zero() {
            return Ok(Amount::zero())
        }

        // can only claim if earned > claimed
        let claimed = self.claimed.get(storage)?;
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

    pub fn lock_tokens (self, storage: &mut impl Storage, increment: Amount) -> StdResult<Amount> {
    }

    pub fn retrieve_tokens (self, storage: &mut impl Storage, decrement: Amount) -> StdResult<Amount> {
    }

    pub fn claim_reward (self, storage: &mut impl Storage) -> StdResult<Amount> {
    }

    fn increment_claimed (self, storage: &mut impl Storage, reward: Amount) -> StdResult<()> {
        self.pool.increment_claimed(storage, reward)?;
        self.claimed.set(storage, (self.claimed.get(storage)? + reward))?;
        Ok(())
    }

    pub fn reset_liquidity_ratio (self, storage: &mut impl Storage) -> StdResult<()> {
        let existed = self.existed(&*storage)?;
        self.update(storage, self.locked.get(&*storage)?, self.pool.locked.get(&*storage)?)?;
        self.last_present.set(storage, existed)?;
        Ok(())
    }

    /// Commit rolling values to storage
    fn update (self, storage: &mut impl Storage, user_locked: Amount, pool_locked: Amount) -> StdResult<()> {
        // Prevent replay
        let now = self.pool.now(&*storage)?;
        if let Ok(timestamp) = self.timestamp.get(storage) {
            if timestamp > now {
                return Err(StdError::generic_err("no time travel"))
            }
        }

        // Increment existence
        self.last_existed.set(storage, self.existed(&*storage)?)?;

        // Increment presence if user has currently locked tokens
        self.last_present.set(storage, self.present(&*storage)?)?;

        // Cooldown is calculated since the timestamp.
        // Since we'll be updating the timestamp, commit the current cooldown
        self.last_cooldown.set(storage, self.cooldown(&*storage)?)?;

        // Always increment lifetime
        self.last_lifetime.set(storage, self.lifetime(&*storage)?)?;

        // Set user's time of last update to now
        self.timestamp.set(storage, now)?;

        // Update amount locked
        self.locked.set(storage, user_locked)?;

        // Update total amount locked in pool
        self.pool.update_locked(storage, pool_locked)?;

        Ok(())
    }

}

fn enforce_cooldown (elapsed: Time, cooldown: Time) -> StdResult<()> {
    if elapsed >= cooldown {
        Ok(())
    } else {
        Err(StdError::generic_err(format!("lock tokens for {} more seconds to be eligible", cooldown - elapsed)))
    }
}
