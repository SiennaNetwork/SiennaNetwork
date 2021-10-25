use fadroma::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use crate::{core::*, math::*, auth::Auth, keys::*};

pub trait Rewards<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{

    fn init (&self, env: &Env, msg: &RewardsInit) -> StdResult<()> {

        self.set(pool::SELF, &ContractLink {
            address:   env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        }.canonize(&self.api())?);

        self.handle_configure(*env, RewardsConfig {
            lp_token:     msg.config.lp_token,
            reward_token: msg.config.reward_token,
            reward_vk:    msg.config.reward_vk,
            ratio:        Some(msg.config.ratio.unwrap_or((1u128.into(), 1u128.into()))),
            threshold:    Some(msg.config.threshold.unwrap_or(DAY)),
            cooldown:     Some(msg.config.cooldown.unwrap_or(DAY))
        });

        Ok(())

    }

    fn handle (&self, env: Env, msg: RewardsHandle) -> StdResult<HandleResponse> {

        match msg {

            RewardsHandle::Configure(config) =>
                self.handle_configure(env, config),

            RewardsHandle::Lock { amount } =>
                self.handle_lock(env, amount),

            RewardsHandle::Retrieve { amount } =>
                self.handle_retrieve(env, amount),

            RewardsHandle::Claim {} =>
                self.handle_claim(env),

            RewardsHandle::ClosePool { message } =>
                self.handle_close_pool(env, message),

        }

    }

    fn handle_configure (&self, env: Env, config: RewardsConfig) -> StdResult<HandleResponse> {

        Auth::assert_admin(&env);

        if let Some(lp_token)     = config.lp_token {
            self.set(pool::LP_TOKEN,     &lp_token.canonize(&self.api())?);
        }

        if let Some(ratio)        = config.ratio {
            self.set(pool::RATIO,        &ratio);
        }

        if let Some(threshold)    = config.threshold {
            self.set(pool::THRESHOLD,    &threshold);
        }

        if let Some(cooldown)     = config.cooldown {
            self.set(pool::COOLDOWN,     &cooldown);
        }

        if let Some(reward_token) = config.reward_token {
            self.set(pool::REWARD_TOKEN, &reward_token);
        }

        if let Some(reward_vk)    = config.reward_vk {
            self.set(pool::REWARD_VK,    &reward_vk);
        }

        Ok(HandleResponse::default())

    }

    fn handle_lock (&self, env: Env, amount: Amount) -> StdResult<HandleResponse> {

        let address = env.message.sender;

        // Increment user and pool liquidity
        self.update(
            self.canonize(&address)?,
            amount,
            Amount::zero(),
            Amount::zero(),
        )?;

        // Transfer liquidity provision tokens from the user to the contract
        let lp_token = ISnip20::attach(&self.humanize(self.get(pool::LP_TOKEN)?)?);
        let transfer = lp_token.transfer_from(&address, &env.contract.address, amount)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    fn handle_retrieve (&self, env: Env, amount: Uint128) -> StdResult<HandleResponse> {

        let address = env.message.sender;

        // Decrement user and pool liquidity
        self.update(
            self.canonize(&address)?,
            Amount::zero(),
            amount,
            Amount::zero(),
        )?;

        // Transfer liquidity provision tokens from the contract to the user
        let lp_token = ISnip20::attach(&self.humanize(self.get(pool::LP_TOKEN)?)?);
        let transfer = lp_token.transfer(&address, amount)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    fn handle_claim (&self, env: Env) -> StdResult<HandleResponse> {

        let pool = self.get_pool_status(env.block.time)?;

        let key = self.canonize(env.message.sender)?;
        let user = self.get_user_status(pool, key)?;


        // If user must wait before first claim, enforce that here.
        let present   = self.get_ns(user::PRESENT, key.as_slice())?;
        let threshold = self.get(pool::THRESHOLD)?;
        enforce_cooldown(present, threshold)?;

        // If user must wait between claims, enforce that here.
        let cooldown  = self.get_ns(user::COOLDOWN, key.as_slice())?;
        enforce_cooldown(0, cooldown)?;

        // See if there is some unclaimed reward amount:
        let claimable = self.claimable(&*storage)?;
        if claimable == Amount::zero() {
            return Err(StdError::generic_err(
                "You've already received as much as your share of the reward pool allows. \
                Keep your liquidity tokens locked and wait for more rewards to be vested, \
                and/or lock more liquidity tokens to grow your share of the reward pool."
            ))
        }

        // Update user timestamp, and the things synced to it.
        self.update(
            self.canonize(&address)?,
            Amount::zero(),
            Amount::zero(),
            reward,
        )?;

        // Reset the cooldown, so that the user has to wait before claiming again)
        self.last_cooldown.set(storage, self.pool.cooldown.get(storage)?)?;

        // Optionally, reset the user's `lifetime` and `share` if they have currently
        // 0 tokens locked. The intent is for this to be the user's last reward claim
        // after they've left the pool completely. If they provide exactly 0 liquidity
        // at some point, when they come back they have to start over, which is OK
        // because they can then start claiming rewards immediately, without waiting
        // for threshold, only cooldown.
        if locked == Amount::zero() {
            self.last_lifetime.set(storage, Volume::zero())?;
            self.claimed.set(storage, Amount::zero())?;
        }

        // Transfer reward tokens from the contract to the user
        let lp_token = ISnip20::attach(&self.humanize(self.get(pool::LP_TOKEN)?)?);
        let transfer = lp_token.transfer(address, amount)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    fn handle_close_pool (&self, env: Env, message: String) -> StdResult<HandleResponse> {
        Auth::assert_admin(self, &env);
        self.set(pool::CLOSED, Some((self.now(&*storage)?, message)))?;
        Ok(HandleResponse::default())
    }

    /// Commit rolling values to storage
    /// Every time the amount of tokens locked in the pool is updated,
    /// the pool's lifetime liquidity is tallied and and the timestamp is updated.
    fn update (
        self,
        id: CanonicalAddr,
        deposited: Amount,
        withdrawn: Amount,
        claimed:   Amount
    ) -> StdResult<()> {
        let mut user_locked = self.get_ns::<Amount>(user::LOCKED, id.as_slice())?;
        let mut pool_locked = self.get::<Amount>(pool::LOCKED)?;

        if deposited > Amount::zero() {
            user_locked += deposited;
            pool_locked += deposited;
        }

        if withdrawn > Amount::zero() {
            // User must have enough locked to retrieve
            if user_locked < withdrawn {
                return Err(StdError::generic_err(
                    format!("not enough locked ({} < {})", user_locked, withdrawn)
                ))
            }
            // If pool does not have enough lp tokens then something has gone badly wrong
            if pool_locked < withdrawn {
                return Err(StdError::generic_err(
                    format!("FATAL: not enough tokens in pool ({} < {})", pool_locked, withdrawn)
                ))
            }
            user_locked -= withdrawn;
            pool_locked -= withdrawn;
        }

        if deposited > Amount::zero() || withdrawn > Amount::zero() {
            self.set_ns(user::LOCKED, id.as_slice(), user_locked)?;
            self.set(pool::LOCKED, pool_locked)?;
        }

        if claimed > Amount::zero() {
            // Update how much has been claimed
            let user_claimed = self.get_ns::<Amount>(user::LOCKED, id.as_slice())?;
            let pool_claimed = self.get::<Amount>(pool::LOCKED)?;
            self.set_ns(user::CLAIMED, id.as_slice()?, user_claimed + reward)?;
            self.set(pool::CLAIMED, pool_claimed + reward)?;
        }

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
        match self.pool.seeded.get(&*storage)? as Option<Time> {
            None => {
                // If this is the first time someone is locking tokens in this pool,
                // store the timestamp. This is used to start the pool liquidity ratio
                // calculation from the time of first lock instead of from the time
                // of contract init.
                // * Using is_none here fails type inference.
                self.pool.seeded.set(storage, self.now)?;
            },
            Some(0) => {
                // * Zero timestamp is special-cased - apparently cosmwasm 0.10
                //   can't tell the difference between None and the 1970s.
                return Err(StdError::generic_err("you jivin' yet?"));
            },
            _ => {}
        };

        let lifetime = self.pool.lifetime(&*storage)?;
        let now      = self.pool.now(&*storage)?;
        let liquid   = self.pool.liquid(&*storage)?;
        self.pool.last_liquid.set(storage, liquid)?;
        self.pool.last_lifetime.set(storage, lifetime)?;
        self.pool.locked.set(storage, balance)?;
        self.pool.timestamp.set(storage, now)?;
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
            reward_token: self.humanize(self.get(pool::REWARD_TOKEN)?)?,
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
        let last_update = self.get(pool::TIMESTAMP)?;
        if now < last_update {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed  = now - last_update;
        let locked   = self.get(pool::LOCKED)?;
        let lifetime = tally(self.get(pool::LIFETIME)?, elapsed, locked)?;

        let balance = self.load_reward_balance()?;
        let claimed = self.get(pool::CLAIMED)?;
        let vested  = claimed + balance;

        let liquid          = Volume::zero();
        let existed         = Volume::zero();
        let liquidity_ratio = Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(liquid, existed)?
            .low_u128().into();

        let global_ratio = self.get(pool::RATIO);

        let closed = self.get(pool::CLOSED)?;

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
        &self, pool: &PoolStatus, id: CanonicalAddr, vk: String
    ) -> StdResult<UserStatus> {
        let last_update = self.get_ns(user::TIMESTAMP, id.as_slice())?;
        if pool.now < last_update {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed  = pool.now - last_update;
        let locked   = self.get_ns(user::LOCKED, id.as_slice())?;
        let lifetime = tally(self.get_ns(user::LIFETIME, id.as_slice())?, elapsed, locked)?;

        let existed = self.get_ns(user::EXISTED, id.as_slice())? +
            elapsed;
        let present = self.get_ns(user::PRESENT, id.as_slice())? +
            if locked > Amount::zero() { elapsed } else { 0 };

        let share = Volume::from(basis)
            .diminish_or_zero(lifetime, pool.lifetime)?
            .diminish_or_zero(present, existed)?;
        let earned = Amount::from(pool.budget)
            .diminish_or_zero(pool.liquid, pool.existed)?
            .diminish_or_zero(pool.global_ratio.0, pool.global_ratio.1)?;
        let claimed = self.get_ns(user::CLAIMED, id.as_slice())?;

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

        let age = self.get_ns(user::AGE, id.as_slice())?;
        let cooldown = self.get_ns(user::COOLDOWN, id.as_slice())?;

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
        let self_link   = self.humanize(self.get(pool::SELF)?)?;
        let reward_link = self.humanize(self.get(pool::REWARD_TOKEN)?)?;
        let reward_vk   = self.get::<ViewingKey>(pool::REWARD_VK)?.0;
        let lp_link     = self.humanize(self.get(pool::LP_TOKEN)?)?;
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
    lp_token:     Option<ContractLink<HumanAddr>>,
    reward_token: Option<ContractLink<HumanAddr>>,
    reward_vk:    Option<String>,
    ratio:        Option<(Uint128, Uint128)>,
    threshold:    Option<Time>,
    cooldown:     Option<Time>
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
    cooldown:    Time,

    /// Reason claimable is 0
    reason: Option<String>
}

fn enforce_cooldown (elapsed: Time, cooldown: Time) -> StdResult<()> {
    if elapsed >= cooldown {
        Ok(())
    } else {
        Err(StdError::generic_err(format!(
            "lock tokens for {} more blocks to be eligible", cooldown - elapsed
        ))
    }
}
