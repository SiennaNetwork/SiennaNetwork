use fadroma::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use crate::{math::*, auth::Auth, keys::*};

pub trait Rewards<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{

    /// Initialize the rewards module
    fn init (&mut self, env: &Env, config: RewardsConfig) -> StdResult<CosmosMsg> {

        let config = RewardsConfig {
            lp_token:     config.lp_token,
            reward_token: Some(config.reward_token.ok_or(
                StdError::generic_err("need to provide link to reward token")
            )?),
            reward_vk:    Some(config.reward_vk.unwrap_or("".into())),
            ratio:        Some(config.ratio.unwrap_or((1u128.into(), 1u128.into()))),
            threshold:    Some(config.threshold.unwrap_or(DAY)),
            cooldown:     Some(config.cooldown.unwrap_or(DAY))
        };

        self.set(pool::SELF, &self.canonize(ContractLink {
            address:   env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        })?)?;

        self.handle_configure(&config)?;

        self.set_own_vk(&config.reward_vk.unwrap())

    }

    /// Handle transactions
    fn handle (&mut self, env: Env, msg: RewardsHandle) -> StdResult<HandleResponse> {

        match msg {

            RewardsHandle::Configure(config) => {
                Auth::assert_admin(self, &env)?;
                self.handle_configure(&config)
            },

            RewardsHandle::Lock { amount } =>
                self.handle_deposit(env, amount),

            RewardsHandle::Retrieve { amount } =>
                self.handle_withdraw(env, amount),

            RewardsHandle::Claim {} =>
                self.handle_claim(env),

            RewardsHandle::ClosePool { message } =>
                self.handle_close_pool(env, message),

        }

    }

    /// Store configuration values
    fn handle_configure (&mut self, config: &RewardsConfig) -> StdResult<HandleResponse> {

        let mut messages = vec![];

        if let Some(reward_token) = &config.reward_token {
            self.set(pool::REWARD_TOKEN, &reward_token);
        }
        if let Some(reward_vk) = &config.reward_vk {
            messages.push(self.set_own_vk(&reward_vk)?);
        }
        if let Some(lp_token) = &config.lp_token {
            self.set(pool::LP_TOKEN, &self.canonize(lp_token.clone())?);
        }
        if let Some(ratio) = &config.ratio {
            self.set(pool::RATIO, &ratio);
        }
        if let Some(threshold) = &config.threshold {
            self.set(pool::THRESHOLD, &threshold);
        }
        if let Some(cooldown) = &config.cooldown {
            self.set(pool::COOLDOWN, &cooldown);
        }

        Ok(HandleResponse { messages, log: vec![], data: None })

    }

    /// Store a viewing key for the reward balance and
    /// generate a transaction to set it in the corresponding token contract
    fn set_own_vk (&mut self, vk: &String) -> StdResult<CosmosMsg> {
        self.set(pool::REWARD_VK, &vk);
        ISnip20::attach(&self.get(pool::REWARD_TOKEN)?).set_viewing_key(&vk)
    }

    /// Deposit LP tokens from user into pool
    fn handle_deposit (&mut self, env: Env, deposited: Amount) -> StdResult<HandleResponse> {

        let (mut pool, mut user) = self.before_user_action(&env)?;

        // Increment user and pool liquidity
        user.locked += deposited;
        pool.locked += deposited;
        let id = self.canonize(env.message.sender.clone())?;
        self.update_locked(&pool, &user, &id);

        // Update stored value
        self.after_user_action(env.block.time, &pool, &user, &id)?;

        // Transfer liquidity provision tokens from the user to the contract
        let lp_link  = &self.humanize(self.get::<ContractLink<CanonicalAddr>>(pool::LP_TOKEN)?)?;
        let lp_token = ISnip20::attach(lp_link);
        let transfer = lp_token.transfer_from(&env.message.sender, &env.contract.address, deposited)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    /// Withdraw deposited LP tokens from pool back to the user
    fn handle_withdraw (&mut self, env: Env, withdrawn: Uint128) -> StdResult<HandleResponse> {

        let (mut pool, mut user) = self.before_user_action(&env)?;

        // User must have enough locked to retrieve
        if user.locked < withdrawn {
            return Err(StdError::generic_err(format!(
                "not enough locked ({} < {})", user.locked, withdrawn
            )))
        }

        // If pool does not have enough lp tokens then something has gone badly wrong
        if pool.locked < withdrawn {
            return Err(StdError::generic_err(format!(
                "FATAL: not enough tokens in pool ({} < {})", pool.locked, withdrawn
            )))
        }

        // Decrement user and pool liquidity
        pool.locked = (pool.locked - withdrawn)?;
        user.locked = (user.locked - withdrawn)?;
        let id = self.canonize(env.message.sender.clone())?;
        self.update_locked(&pool, &user, &id);

        self.after_user_action(env.block.time, &pool, &user, &id)?;

        // Transfer liquidity provision tokens from the contract to the user
        let lp_link  = &self.humanize(self.get::<ContractLink<CanonicalAddr>>(pool::LP_TOKEN)?)?;
        let lp_token = ISnip20::attach(lp_link);
        let transfer = lp_token.transfer(&env.message.sender.clone(), withdrawn)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    /// Transfer rewards to user if eligible
    fn handle_claim (&mut self, env: Env) -> StdResult<HandleResponse> {

        let (mut pool, mut user) = self.before_user_action(&env)?;

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

        // Increment claimed counters
        user.claimed += user.claimable;
        pool.claimed += user.claimable;
        let id = self.canonize(env.message.sender.clone())?;
        self.update_claimed(&pool, &user, &id);

        // Update user timestamp, and the things synced to it.
        self.after_user_action(env.block.time, &pool, &user, &id)?;

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
        let reward_link  = self.humanize(self.get::<ContractLink<CanonicalAddr>>(pool::REWARD_TOKEN)?)?;
        let reward_token = ISnip20::attach(&reward_link);
        let transfer     = reward_token.transfer(&env.message.sender, user.claimable)?;
        Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})

    }

    /// Admin can mark pool as closed
    fn handle_close_pool (&mut self, env: Env, message: String) -> StdResult<HandleResponse> {
        Auth::assert_admin(self, &env);
        self.set(pool::CLOSED, Some((env.block.time, message)));
        Ok(HandleResponse::default())
    }

    /// Get user and pool status, prevent replays
    fn before_user_action (&mut self, env: &Env) -> StdResult<(Pool, User)> {

        // Get pool state
        let now = env.block.time;
        let pool = self.get_pool_status(now)?;
        if pool.updated > now {
            return Err(StdError::generic_err("no time travel"))
        }

        // Get user state
        let id = self.canonize(env.message.sender.clone())?;
        let user = self.get_user_status(&pool, id)?;
        if user.updated > now {
            return Err(StdError::generic_err("no time travel"))
        }

        Ok((pool, user))
    }

    /// Commit amount of locked tokens for user and pool
    fn update_locked (
        &mut self, pool: &Pool, user: &User, id: &CanonicalAddr
    ) -> StdResult<()> {
        self.set_ns(user::LOCKED, id.as_slice(), user.locked)?;
        self.set(pool::LOCKED, pool.locked)
    }

    /// Commit amount of rewards claimed for user and pool, and reset the cooldown
    fn update_claimed (
        &mut self, pool: &Pool, user: &User, id: &CanonicalAddr
    ) -> StdResult<()> {
        self.set_ns(user::CLAIMED, id.as_slice(), user.claimed)?;
        self.set(pool::CLAIMED, pool.claimed)?;
        self.set_ns(user::COOLDOWN, id.as_slice(), pool.cooldown)
    }

    /// Commit remaining values to storage
    fn after_user_action (
        &mut self, now: Time, pool: &Pool, user: &User, id: &CanonicalAddr
    ) -> StdResult<()> {

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
        self.set(pool::LIQUID,   pool.liquid)?;
        self.set(pool::LIFETIME, pool.lifetime)?;
        self.set(pool::UPDATED,  now)?;

        self.set_ns(user::EXISTED,  id.as_slice(), user.existed)?;
        self.set_ns(user::PRESENT,  id.as_slice(), user.present)?;
        self.set_ns(user::COOLDOWN, id.as_slice(), user.cooldown)?;
        self.set_ns(user::LIFETIME, id.as_slice(), user.lifetime)?;
        self.set_ns(user::UPDATED,  id.as_slice(), now)?;

        Ok(())

    }

    /// Handle queries
    fn query (&self, msg: RewardsQuery) -> StdResult<RewardsResponse> {
        match msg {
            RewardsQuery::Status { at, address, key } =>
                self.query_status(at, address, key)
        }
    }

    /// Report pool status and optionally user status, at a given time
    fn query_status (
        &self, now: Time, address: Option<HumanAddr>, key: Option<String>
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
            Auth::check_viewing_key(self, &ViewingKey(key), id.as_slice())?;
            Some(self.get_user_status(&pool, id)?)
        } else {
            None
        };

        Ok(RewardsResponse::Status { time: now, pool, user })

    }

    /// Compute pool status
    fn get_pool_status (&self, now: Time) -> StdResult<Pool> {

        let updated = self.get(pool::UPDATED)?;
        if now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }
        let elapsed = now - updated;

        let locked   = self.get(pool::LOCKED)?;
        let lifetime = accumulate(self.get(pool::LIFETIME)?, elapsed, locked)?;

        let seeded  = self.get::<Option<Time>>(pool::SEEDED)?;
        let existed = if let Some(seeded) = seeded {
            if now < seeded {
                return Err(StdError::generic_err("no time travel"))
            }
            Some(now - seeded)
        } else {
            None
        };

        let liquid = self.get::<Time>(pool::LIQUID)? +
            if locked > Amount::zero() { elapsed } else { 0 };

        let reward_link = self.humanize(
            self.get::<ContractLink<CanonicalAddr>>(pool::REWARD_TOKEN)?
        )?;
        let self_link   = self.humanize(self.get::<ContractLink<CanonicalAddr>>(pool::SELF)?)?;
        let rewards_vk  = self.get::<ViewingKey>(pool::REWARD_VK)?.0;
        let mut balance = ISnip20::attach(&reward_link)
            .query_balance(self.querier(), &self_link.address, &rewards_vk)?;

        let lp_link = self.humanize(
            self.get::<ContractLink<CanonicalAddr>>(pool::LP_TOKEN)?
        )?;

        if reward_link == lp_link {
            // separate balances for single-sided staking
            balance = (balance - locked)?;
        }

        let claimed = self.get(pool::CLAIMED)?;

        let vested  = claimed + balance;

        Ok(Pool {
            now,
            seeded,
            updated,
            existed,
            liquid,

            vested,
            claimed,
            balance,

            locked,
            lifetime,

            deployed:     self.get(pool::DEPLOYED)?,
            cooldown:     self.get(pool::COOLDOWN)?,
            threshold:    self.get(pool::THRESHOLD)?,
            global_ratio: self.get(pool::RATIO)?,
            closed:       self.get(pool::CLOSED)?,
        })
    }

    /// Compute user status
    fn get_user_status (&self, pool: &Pool, id: CanonicalAddr) -> StdResult<User> {

        let now = pool.now;

        let registered = self.get_ns(user::REGISTERED, id.as_slice())?;

        if now < registered {
            return Err(StdError::generic_err("no time travel"))
        }

        let existed = now - registered;

        let updated = self.get_ns(user::UPDATED, id.as_slice())?;

        if now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed = now - updated;

        let locked = self.get_ns(user::LOCKED, id.as_slice())?;

        let lifetime = accumulate(
            self.get_ns(user::LIFETIME, id.as_slice())?,
            elapsed,
            locked)?;

        let present = self.get_ns::<Time>(user::PRESENT, id.as_slice())? +
            if locked > Amount::zero() { elapsed } else { 0 };

        let budget = Amount::from(pool.vested)
            .diminish_or_zero(pool.liquid, pool.existed.unwrap_or(0))?
            .diminish_or_zero(pool.global_ratio.0, pool.global_ratio.1)?;

        let earned = Volume::from(budget)
            .diminish_or_zero(lifetime, pool.lifetime)?
            .diminish_or_zero(present, existed)?
            .low_u128().into();

        let claimed = self.get_ns(user::CLAIMED, id.as_slice())?;

        let mut reason: Option<&str> = None;

        let cooldown = self.get_ns(user::COOLDOWN, id.as_slice())?;

        let claimable = if present < pool.threshold {
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

        Ok(User {
            registered,

            existed,
            present,
            presence: (present, existed),

            updated,
            locked,
            momentary_share: (locked, pool.locked),

            lifetime,
            lifetime_share: (lifetime, pool.lifetime),

            earned,
            claimed,
            claimable,

            cooldown,

            reason: reason.map(|x|x.to_string())
        })
    }

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
        pool: Pool,
        user: Option<User>
    }
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Pool {
    /// "For what point in time do the following values hold true?"
    /// Passed on instantiation.
    now:          Time,

    /// "When was this pool deployed?"
    /// Set to current time on init.
    deployed:     Time,

    /// "Is this pool closed, and if so, when and why?"
    /// Set irreversibly via handle method.
    closed:       Option<String>,

    /// "When were LP tokens first locked?"
    /// Set to current time on first lock.
    seeded:       Option<Time>,

    /// "For how many units of time has this pool existed?"
    /// Computed as now - seeded
    existed:      Option<Time>,

    /// "When was the last time someone locked or unlocked tokens?"
    /// Set to current time on lock/unlock.
    updated:      Time,

    /// Used to compute what portion of the time the pool was not empty.
    /// Before lock/unlock, if locked > 0, this is incremented
    /// by pool.elapsed
    liquid:       Time,

    /// "What liquidity has this pool contained up to this point?"
    /// Before lock/unlock, if locked > 0, this is incremented
    /// by pool.elapsed * pool.locked
    lifetime:     Volume,

    /// "What liquidity is there in the whole pool right now?"
    /// Incremented/decremented on lock/unlock.
    locked:       Amount,

    /// "What reward balance is there in the pool?"
    /// Queried from reward token.
    balance:      Amount,

    /// "What rewards has everyone received so far?"
    /// Incremented on claim.
    claimed:      Amount,

    /// "What rewards were unlocked for this pool so far?"
    /// Computed as balance + claimed.
    vested:       Amount,

    /// "What is the initial bonding period?"
    /// User needs to lock LP tokens for this amount of time
    /// before rewards can be claimed. Configured on init.
    threshold:    Time,

    /// "How much must the user wait between claims?"
    /// Configured on init.
    /// User cooldowns are reset to this value on claim.
    cooldown:     Time,

    global_ratio:    (Amount, Amount),
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct User {

    /// When did this user first provide liquidity?
    /// Set to current time on first update.
    registered:      Time,

    /// How much time has passed since this user became known to the contract?
    /// Computed as pool.now - user.registered
    existed:         Time,

    /// When did this user's liquidity amount last change?
    /// Set to current time on update.
    updated:         Time,

    /// For how much time this user has provided non-zero liquidity?
    /// Incremented on update by user.elapsed if user.locked > 0
    present:         Time,

    presence: (Time, Time),

    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    locked:          Amount,

    /// What portion of the pool is this user currently contributing?
    /// Computed as user.locked / pool.locked
    momentary_share: (Amount, Amount),

    /// How much liquidity has this user provided since they first appeared?
    /// Incremented on update by user.locked * elapsed if user.locked > 0
    lifetime:        Volume,

    /// What portion of all the liquidity has this user ever contributed?
    /// Computed as user.lifetime / pool.lifetime
    lifetime_share:  (Volume, Volume),

    /// How much rewards has this user earned?
    /// Computed as user.lifetime_share * pool.vested
    earned:          Amount,

    /// How much rewards has this user claimed so far?
    /// Incremented on claim by the amount claimed.
    claimed:         Amount,

    /// How much rewards can this user claim?
    /// Computed as user.earned - user.claimed, clamped at 0.
    claimable:       Amount,

    /// User-friendly reason why claimable is 0
    reason:          Option<String>,

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
