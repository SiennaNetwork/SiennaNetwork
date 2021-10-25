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

        Auth::assert_admin(self, &env)?;

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
        self.update(&env, amount, Amount::zero(), Amount::zero())?;

        // Transfer liquidity provision tokens from the user to the contract
        let lp_token = ISnip20::attach(&self.humanize(self.get(pool::LP_TOKEN)?)?);
        let transfer = lp_token.transfer_from(&address, &env.contract.address, amount)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    fn handle_retrieve (&self, env: Env, amount: Uint128) -> StdResult<HandleResponse> {

        let address = env.message.sender;

        // Decrement user and pool liquidity
        self.update(&env, Amount::zero(), amount, Amount::zero())?;

        // Transfer liquidity provision tokens from the contract to the user
        let lp_token = ISnip20::attach(&self.humanize(self.get(pool::LP_TOKEN)?)?);
        let transfer = lp_token.transfer(&address, amount)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    fn handle_claim (&self, env: Env) -> StdResult<HandleResponse> {
        let pool = self.get_pool_status(env.block.time)?;
        let id   = self.canonize(env.message.sender)?;
        let user = self.get_user_status(&pool, id)?;

        // If user must wait before first claim, enforce that here.
        enforce_cooldown(user.present, pool.threshold)?;

        // If user must wait between claims, enforce that here.
        enforce_cooldown(0, user.cooldown)?;

        // See if there is some unclaimed reward amount:
        if user.claimable == Amount::zero() {
            return Err(StdError::generic_err(
                "You've already received as much as your share of the reward pool allows. \
                Keep your liquidity tokens locked and wait for more rewards to be vested, \
                and/or lock more liquidity tokens to grow your share of the reward pool."
            ))
        }

        // Update user timestamp, and the things synced to it.
        self.update(&env, Amount::zero(), Amount::zero(), user.claimable)?;

        // Reset the cooldown, so that the user has to wait before claiming again)
        self.set_ns(user::COOLDOWN, id.as_slice(), pool.cooldown)?;

        if user.locked == Amount::zero() {
            // Optionally, reset the user's `lifetime` and `share` if they have currently
            // 0 tokens locked. The intent is for this to be the user's last reward claim
            // after they've left the pool completely. If they provide exactly 0 liquidity
            // at some point, when they come back they have to start over, which is OK
            // because they can then start claiming rewards immediately, without waiting
            // for threshold, only cooldown.
            self.set_ns(user::LIFETIME, id.as_slice(), Volume::zero())?;
            self.set_ns(user::CLAIMED,  id.as_slice(), Amount::zero())?;
        }

        // Transfer reward tokens from the contract to the user
        let reward_token = ISnip20::attach(&self.humanize(self.get(pool::REWARD_TOKEN)?)?);
        let transfer = reward_token.transfer(&env.message.sender, user.claimable)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    fn handle_close_pool (&self, env: Env, message: String) -> StdResult<HandleResponse> {
        Auth::assert_admin(self, &env);
        self.set(pool::CLOSED, Some((env.block.time, message)));
        Ok(HandleResponse::default())
    }

    /// Commit rolling values to storage
    /// Every time the amount of tokens locked in the pool is updated,
    /// the pool's lifetime liquidity is tallied and and the timestamp is updated.
    fn update (
        &self,
        env:       &Env,
        deposited: Amount,
        withdrawn: Amount,
        reward:    Amount
    ) -> StdResult<()> {

        // Get pool state
        let now = env.block.time;
        let mut pool = self.get_pool_status(now)?;
        if pool.updated > now {
            return Err(StdError::generic_err("no time travel"))
        }

        // Get user state
        let id  = self.canonize(env.message.sender)?;
        let mut user = self.get_user_status(&pool, id)?;
        if user.updated > now {
            return Err(StdError::generic_err("no time travel"))
        }

        // Add deposits
        if deposited > Amount::zero() {
            user.locked += deposited;
            pool.locked += deposited;
        }

        // Subtract withdrawals
        if withdrawn > Amount::zero() {
            // User must have enough locked to retrieve
            if user.locked < withdrawn {
                return Err(StdError::generic_err(format!(
                    "not enough locked ({} < {})", user.locked, withdrawn
                )))
            }
            user.locked = (user.locked - withdrawn)?;

            // If pool does not have enough lp tokens then something has gone badly wrong
            if pool.locked < withdrawn {
                return Err(StdError::generic_err(format!(
                    "FATAL: not enough tokens in pool ({} < {})", pool.locked, withdrawn
                )))
            }
            pool.locked = (pool.locked - withdrawn)?;
        }

        // Accumulate claims
        if reward > Amount::zero() {
            user.claimed += reward;
            pool.claimed += reward;
            self.set_ns(user::CLAIMED, id.as_slice(), user.claimed)?;
            self.set(pool::CLAIMED, pool.claimed)?;
        }

        // Save updates to balances
        if deposited > Amount::zero() || withdrawn > Amount::zero() {
            self.set_ns(user::LOCKED, id.as_slice(), user.locked)?;
            self.set(pool::LOCKED, pool.locked)?;
        }

        self.set_ns(user::EXISTED,   id.as_slice(), user.existed)?;
        self.set_ns(user::PRESENT,   id.as_slice(), user.present)?;
        self.set_ns(user::COOLDOWN,  id.as_slice(), user.cooldown)?;
        self.set_ns(user::LIFETIME,  id.as_slice(), user.lifetime)?;
        self.set_ns(user::TIMESTAMP, id.as_slice(), now)?;

        match pool.seeded {
            None => {
                // If this is the first time someone is locking tokens in this pool,
                // store the timestamp. This is used to start the pool liquidity ratio
                // calculation from the time of first lock instead of from the time
                // of contract init.
                // * Using is_none here fails type inference.
                self.set(pool::SEEDED, now)?;
            },
            Some(0) => {
                // * Zero timestamp is special-cased - apparently cosmwasm 0.10
                //   can't tell the difference between None and the 1970s.
                return Err(StdError::generic_err("you jivin' yet?"));
            },
            _ => {}
        };

        self.set(pool::LIQUID,    pool.liquid)?;
        self.set(pool::LIFETIME,  pool.lifetime)?;
        self.set(pool::TIMESTAMP, now)?;

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
        now:     Time,
        address: Option<HumanAddr>,
        key:     Option<String>
    ) -> StdResult<RewardsResponse> {
        if address.is_some() && key.is_none() {
            return Err(StdError::generic_err("no viewing key"))
        }
        let pool = self.get_pool_status(now)?;
        if now < pool.updated {
            return Err(StdError::generic_err("no history"))
        }
        let user = if let (Some(address), Some(key)) = (address, key) {
            let id = self.canonize(address)?;
            Auth::check_viewing_key(self, &ViewingKey(key), id.as_slice());
            Some(self.get_user_status(&pool, id)?)
        } else {
            None
        };
        Ok(RewardsResponse::Status { time: now, pool, user })
    }

    fn get_pool_status (&self, now: Time) -> StdResult<PoolStatus> {
        let updated = self.get(pool::TIMESTAMP)?;
        if now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed  = now - updated;
        let locked   = self.get(pool::LOCKED)?;
        let lifetime = tally(self.get(pool::LIFETIME)?, elapsed, locked)?;

        let self_link    = self.humanize(self.get(pool::SELF)?)?;
        let reward_link  = self.humanize(self.get(pool::REWARD_TOKEN)?)?;
        let reward_vk    = self.get::<ViewingKey>(pool::REWARD_VK)?.0;
        let reward_token = ISnip20::attach(&reward_link).query(&(self.querier()));
        let mut balance  = reward_token.balance(&self_link, &reward_vk)?;

        // separate balances for single-sided staking
        let lp_link      = self.humanize(self.get(pool::LP_TOKEN)?)?;
        if reward_link == lp_link {
            balance = (balance - locked)?;
        }

        let claimed = self.get(pool::CLAIMED)?;
        let vested  = claimed + balance;

        let liquid          = Volume::zero();
        let existed         = Volume::zero();
        let liquidity_ratio = Volume::from(HUNDRED_PERCENT)
            .diminish_or_max(liquid, existed)?
            .low_u128().into();

        let global_ratio = self.get(pool::RATIO)?;
        let closed    = self.get(pool::CLOSED)?;
        let cooldown  = self.get(pool::COOLDOWN)?;
        let threshold = self.get(pool::THRESHOLD)?;

        Ok(PoolStatus {
            now,
            cooldown, threshold,
            updated, locked, lifetime,
            balance, claimed, vested,
            liquid: liquidity_ratio, global_ratio,
            closed,
        })
    }

    fn get_user_status (
        &self, pool: &PoolStatus, id: CanonicalAddr
    ) -> StdResult<UserStatus> {
        let updated = self.get_ns(user::TIMESTAMP, id.as_slice())?;
        if pool.now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed  = pool.now - updated;
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

        let age      = self.get_ns(user::AGE, id.as_slice())?;
        let cooldown = self.get_ns(user::COOLDOWN, id.as_slice())?;

        Ok(UserStatus {
            updated,
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
    pub lp_token:     Option<ContractLink<HumanAddr>>,
    pub reward_token: Option<ContractLink<HumanAddr>>,
    pub reward_vk:    Option<String>,
    pub ratio:        Option<(Uint128, Uint128)>,
    pub threshold:    Option<Time>,
    pub cooldown:     Option<Time>
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
        time: Time,
        pool: PoolStatus,
        user: Option<UserStatus>
    }
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct PoolStatus {
    /// Current time
    now:          Time,

    /// Load the last update timestamp or default to current time
    /// (this has the useful property of keeping `elapsed` zero for strangers)
    /// When was liquidity last updated.
    /// Set to current time on lock/unlock.
    updated:      Time,

    /// How much liquidity has this pool contained up to this point.
    /// On lock/unlock, if locked > 0 before the operation, this is incremented
    /// in intervals of (moments since last update * current balance)
    lifetime:     Volume,

    /// How much liquidity is there in the whole pool right now.
    /// Incremented/decremented on lock/unlock.
    locked:       Amount,
    
    /// Whether this pool is closed
    closed:       Option<String>,

    balance:      Amount,

    /// Rewards claimed by everyone so far.
    /// Amount of rewards already claimed
    /// Incremented on claim.
    claimed:      Amount,

    vested:       Amount,

    /// How much the user needs to wait before they can claim for the first time.
    /// Configured on init.
    /// For how many blocks does the user need to have provided liquidity
    /// in order to be eligible for rewards
    threshold:    Time,

    /// How much the user must wait between claims.
    /// Configured on init.
    /// For how many blocks does the user need to wait
    /// after claiming rewards before being able to claim them again
    cooldown:     Time,

    /// Used to compute what portion of the time the pool was not empty.
    /// On lock/unlock, if the pool was not empty, this is incremented
    /// by the time elapsed since the last update.
    liquid:       Amount,

    global_ratio: Amount
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct UserStatus {

    /// When did this user first provide liquidity?
    /// Set to current time on first update.
    registered:      Time,

    /// For how much time this user has provided non-zero liquidity?
    /// Incremented on update by user.elapsed if user.locked > 0.
    present:         Time,

    /// When did this user's liquidity amount last change?
    /// Set to current time on update.
    updated:         Time,

    /// How much liquidity has this user provided since they first appeared?
    /// Incremented on update by user.locked * elapsed if user.locked > 0
    lifetime:        Volume,

    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    locked:          Amount,

    /// What portion of the pool is this user currently contributing?
    /// Computed as user.locked / pool.locked
    momentary_share: Amount,

    /// What portion of all the liquidity has this user ever contributed?
    /// Computed as user.lifetime / pool.lifetime
    average_share:   Amount,

    /// How much rewards has this user earned?
    /// Computed as user.average_share * pool.vested
    earned:          Amount,

    /// How much rewards has this user claimed so far?
    /// Incremented on claim by the amount claimed.
    claimed:         Amount,

    /// How much rewards can this user claim?
    /// Computed as user.earned - user.claimed, clamped at 0.
    claimable:       Amount,

    /// User-friendly reason why claimable is 0
    reason:          Option<String>,

    /// For how many units of time has this user been known to the contract?
    /// Incremented on lock/unlock by time elapsed since last update.
    age:             Time,

    /// How many units of time remain until the user can claim again?
    /// Decremented on lock/unlock, reset to pool.cooldown on claim.
    cooldown:        Time,
}

fn enforce_cooldown (elapsed: Time, cooldown: Time) -> StdResult<()> {
    if elapsed >= cooldown {
        Ok(())
    } else {
        Err(StdError::generic_err(format!(
            "lock tokens for {} more blocks to be eligible", cooldown - elapsed
        )))
    }
}
