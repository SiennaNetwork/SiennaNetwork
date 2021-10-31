use fadroma::{*, scrt_uint256::Uint256};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use crate::auth::Auth;

/// A monotonic time counter, such as env.block.time or env.block.height
pub type Time   = u64;

/// Amount of funds
pub type Amount = Uint128;

/// Liquidity = amount (u128) * time (u64)
pub type Volume = Uint256;

/// A ratio represented as tuple (nom, denom)
pub type Ratio  = (Uint128, Uint128);

/// 100% with 6 digits after the decimal
pub const HUNDRED_PERCENT: u128 = 100000000u128;

/// Seconds in 24 hours
pub const DAY: Time = 86400;

/// Project current value of an accumulating parameter based on stored value,
/// time since it was last updated, and rate of change, i.e.
/// `current = stored + (elapsed * rate)`
pub fn accumulate (
    total_before_last_update: Volume,
    time_updated_last_update: Time,
    value_after_last_update:  Amount
) -> StdResult<Volume> {
    total_before_last_update + Volume::from(value_after_last_update)
        .multiply_ratio(time_updated_last_update, 1u128)?
}

pub trait Rewards<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{

    /// Initialize the rewards module
    fn init (&mut self, env: &Env, config: RewardsConfig) -> StdResult<Option<CosmosMsg>> {

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

        self.set(pool::DEPLOYED, &env.block.time)?;

        self.handle_configure(&config)?;

        Ok(Some(self.set_own_vk(&config.reward_vk.unwrap())?))

    }

    /// Handle transactions
    fn handle (&mut self, env: &Env, msg: RewardsHandle) -> StdResult<HandleResponse> {

        match msg {

            RewardsHandle::Configure(config) => {
                Auth::assert_admin(self, env)?;
                self.handle_configure(&config)
            },

            RewardsHandle::Lock { amount } =>
                self.handle_deposit(env, amount),

            RewardsHandle::Retrieve { amount } =>
                self.handle_withdraw(env, amount),

            RewardsHandle::Claim {} =>
                self.handle_claim(env),

            RewardsHandle::Close { message } =>
                self.handle_close(env, message),

            RewardsHandle::Drain { snip20, recipient, key } =>
                self.handle_drain(env, snip20, recipient, key),

        }

    }

    /// Store configuration values
    fn handle_configure (&mut self, config: &RewardsConfig) -> StdResult<HandleResponse> {

        let mut messages = vec![];

        if let Some(reward_token) = &config.reward_token {
            self.set(pool::REWARD_TOKEN, &self.canonize(reward_token.clone())?)?;
        }
        if let Some(reward_vk) = &config.reward_vk {
            messages.push(self.set_own_vk(&reward_vk)?);
        }
        if let Some(lp_token) = &config.lp_token {
            self.set(pool::LP_TOKEN, &self.canonize(lp_token.clone())?)?;
        }
        if let Some(ratio) = &config.ratio {
            self.set(pool::RATIO, &ratio)?;
        }
        if let Some(threshold) = &config.threshold {
            self.set(pool::THRESHOLD, &threshold)?;
        }
        if let Some(cooldown) = &config.cooldown {
            self.set(pool::COOLDOWN, &cooldown)?;
        }

        Ok(HandleResponse { messages, log: vec![], data: None })

    }

    /// Store a viewing key for the reward balance and
    /// generate a transaction to set it in the corresponding token contract
    fn set_own_vk (&mut self, vk: &String) -> StdResult<CosmosMsg> {
        self.set(pool::REWARD_VK, &vk)?;
        self.reward_token()?.set_viewing_key(&vk)
    }

    fn self_link (&self) -> StdResult<ContractLink<HumanAddr>> {
        let link = self.get::<ContractLink<CanonicalAddr>>(pool::SELF)?
            .ok_or(StdError::generic_err("no self link"))?;
        Ok(self.humanize(link)?)
    }

    fn lp_token (&self) -> StdResult<ISnip20> {
        let link = self.get::<ContractLink<CanonicalAddr>>(pool::LP_TOKEN)?
            .ok_or(StdError::generic_err("no lp token"))?;
        Ok(ISnip20::attach(self.humanize(link)?))
    }

    fn reward_token (&self) -> StdResult<ISnip20> {
        let link = self.get::<ContractLink<CanonicalAddr>>(pool::REWARD_TOKEN)?
            .ok_or(StdError::generic_err("no reward token"))?;
        Ok(ISnip20::attach(self.humanize(link)?))
    }

    fn reward_vk (&self) -> StdResult<String> {
        Ok(self.get::<ViewingKey>(pool::REWARD_VK)?
            .ok_or(StdError::generic_err("no reward viewing key"))?
            .0)
    }

    /// Deposit LP tokens from user into pool
    fn handle_deposit (&mut self, env: &Env, deposited: Amount) -> StdResult<HandleResponse> {
        let (mut pool, mut user) = self.before_user_action(&env)?;
        if pool.closed.is_some() {
            self.return_all_funds(env, &mut pool, &mut user)
        } else {
            // Set user registration date if this is their first deposit
            let id = self.canonize(env.message.sender.clone())?;
            if self.get_ns::<Time>(user::REGISTERED, id.as_slice())?.is_none() {
                self.set_ns(user::REGISTERED, id.as_slice(), pool.now)?
            }
            // Increment user and pool liquidity
            user.locked += deposited;
            pool.locked += deposited;
            self.update_locked(&pool, &user, &id)?;
            // Update stored value
            self.after_user_action(env.block.time, &pool, &user, &id)?;
            // Transfer liquidity provision tokens from the user to the contract
            let transfer = self.lp_token()?.transfer_from(&env.message.sender, &env.contract.address, deposited)?;
            Ok(HandleResponse {messages: vec![transfer], log: vec![/*TODO*/], data: None})
        }
    }

    /// Withdraw deposited LP tokens from pool back to the user
    fn handle_withdraw (&mut self, env: &Env, withdrawn: Uint128) -> StdResult<HandleResponse> {
        let (mut pool, mut user) = self.before_user_action(&env)?;
        if pool.closed.is_some() {
            self.return_all_funds(env, &mut pool, &mut user)
        } else {
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
            self.update_locked(&pool, &user, &id)?;
            self.after_user_action(env.block.time, &pool, &user, &id)?;
            // Transfer liquidity provision tokens from the contract to the user
            HandleResponse::default()
                .msg(self.lp_token()?.transfer(&env.message.sender.clone(), withdrawn)?)
        }
    }

    /// Transfer rewards to user if eligible
    fn handle_claim (&mut self, env: &Env) -> StdResult<HandleResponse> {
        let (mut pool, mut user) = self.before_user_action(&env)?;
        // If user must wait before first claim, enforce that here.
        enforce_cooldown(user.liquid, pool.threshold)?;
        // If user must wait between claims, enforce that here.
        enforce_cooldown(0, user.cooldown)?;
        if pool.balance == Amount::zero() {
            return Err(StdError::generic_err(
                "This pool is currently empty. \
                However, liquidity shares continue to accumulate."
            ))
        }
        if pool.global_ratio.0 == Amount::zero() {
            return Err(StdError::generic_err(
                "Rewards from this pool are currently stopped. \
                However, liquidity shares continue to accumulate."
            ))
        }
        if user.claimed > user.earned {
            return Err(StdError::generic_err(
                "Your liquidity share has steeply diminished \
                since you last claimed. Lock more tokens to get \
                to the front of the queue faster."
            ))
        }
        if user.claimable == Amount::zero() {
            return Err(StdError::generic_err(
                "You have already claimed your exact share of the rewards."
            ))
        }

        // Increment claimed counters
        user.claimed += user.claimable;
        pool.claimed += user.claimable;
        let id = self.canonize(env.message.sender.clone())?;
        self.set_ns(user::CLAIMED, id.as_slice(), user.claimed)?;
        self.set(pool::CLAIMED, pool.claimed)?;
        self.set_ns(user::COOLDOWN, id.as_slice(), pool.cooldown)?;

        // Update user timestamp, and the things synced to it.
        self.after_user_action(env.block.time, &pool, &user, &id)?;

        if user.locked == Amount::zero() {
            // Reset the user's `lifetime` and `share` if they currently have 0 tokens locked.
            // The intent is for this to be the user's last reward claim after they've left
            // the pool completely. If they provide exactly 0 liquidity at some point,
            // when they come back they have to start over, which is OK because they can
            // then start claiming rewards immediately, without waiting for threshold, only cooldown.
            self.set_ns(user::LIFETIME, id.as_slice(), Volume::zero())?;
            self.set_ns(user::CLAIMED,  id.as_slice(), Amount::zero())?;
        }

        // Transfer reward tokens from the contract to the user
        HandleResponse::default()
            .msg(self.reward_token()?.transfer(&env.message.sender, user.claimable)?)
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
        self.set_ns(user::PRESENT,  id.as_slice(), user.liquid)?;
        self.set_ns(user::COOLDOWN, id.as_slice(), user.cooldown)?;
        self.set_ns(user::LIFETIME, id.as_slice(), user.lifetime)?;
        self.set_ns(user::UPDATED,  id.as_slice(), now)?;
        Ok(())
    }

    /// Admin can mark pool as closed
    fn handle_close (&mut self, env: &Env, message: String) -> StdResult<HandleResponse> {
        Auth::assert_admin(self, &env)?;
        self.set(pool::CLOSED, Some((env.block.time, message)))?;
        Ok(HandleResponse::default())
    }

    /// Closed pools return all funds upon request and prevent further deposits
    fn return_all_funds (
        &mut self, env: &Env, pool: &mut Pool, user: &mut User
    ) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = pool.closed {
            let withdraw_all = user.locked;
            user.locked = 0u128.into();
            pool.locked = (pool.locked - withdraw_all)?;
            let id = self.canonize(env.message.sender.clone())?;
            self.update_locked(&pool, &user, &id)?;
            self.after_user_action(env.block.time, &pool, &user, &id)?;
            HandleResponse::default()
                .msg(self.lp_token()?.transfer(&env.message.sender.clone(), withdraw_all)?)?
                .log("closed", &format!("{} {}", when, why))
        } else {
            panic!()
        }
    }

    /// Closed pools can be drained for manual redistribution of erroneously locked funds
    fn handle_drain (
        &mut self,
        env:       &Env,
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    ) -> StdResult<HandleResponse> {
        Auth::assert_admin(&*self, &env)?;

        let recipient = recipient.unwrap_or(env.message.sender.clone());

        // Update the viewing key if the supplied
        // token info for is the reward token
        if self.reward_token()?.link == snip20 {
            self.set(pool::REWARD_VK, key.clone())?
        }

        let allowance = Uint128(u128::MAX);
        let duration  = Some(env.block.time + DAY * 10000);
        let snip20    = ISnip20::attach(snip20);
        Ok(HandleResponse {
            messages: vec![
                snip20.increase_allowance(&recipient, allowance, duration)?,
                snip20.set_viewing_key(&key)?
            ],
            log: vec![],
            data: None
        })
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
            Auth::check_vk(self, &ViewingKey(key), id.as_slice())?;
            Some(self.get_user_status(&pool, id)?)
        } else {
            None
        };

        Ok(RewardsResponse::Status { time: now, pool, user })

    }

    /// Compute pool status
    fn get_pool_status (&self, now: Time) -> StdResult<Pool> {
        let updated: Time = self.get(pool::UPDATED)?.unwrap_or(now);
        if now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }
        let elapsed = now - updated;
        let locked: Amount = self.get(pool::LOCKED)?.unwrap_or(Amount::zero());
        let lifetime = accumulate(
            self.get(pool::LIFETIME)?.unwrap_or(Volume::zero()),
            elapsed,
            locked
        )?;
        let seeded: Option<Time> = self.get(pool::SEEDED)?;
        let existed = if let Some(seeded) = seeded {
            if now < seeded {
                return Err(StdError::generic_err("no time travel"))
            }
            Some(now - seeded)
        } else {
            None
        };
        let liquid: Time = self.get(pool::LIQUID)?.unwrap_or(0) +
            if locked > Amount::zero() { elapsed } else { 0 };
        let lp_token     = self.lp_token()?;
        let reward_token = self.reward_token()?;
        let mut balance  = reward_token.query_balance(
            self.querier(), &self.self_link()?.address, &self.reward_vk()?
        )?;
        if reward_token.link == lp_token.link {
            // separate balances for single-sided staking
            balance = (balance - locked)?;
        }
        let claimed = self.get(pool::CLAIMED)?.unwrap_or(Amount::zero());
        let vested  = claimed + balance;
        Ok(Pool {
            now, seeded, updated, existed, liquid,
            locked, lifetime,
            vested, claimed, balance,
            cooldown:     self.get(pool::COOLDOWN)?.ok_or(
                StdError::generic_err("missing cooldown")
            )?,
            threshold:    self.get(pool::THRESHOLD)?.ok_or(
                StdError::generic_err("missing threshold")
            )?,
            global_ratio: self.get(pool::RATIO)?.ok_or(
                StdError::generic_err("missing global ratio")
            )?,
            deployed:     self.get(pool::DEPLOYED)?.ok_or(
                StdError::generic_err("missing deploy timestamp")
            )?,
            closed:       self.get::<CloseSeal>(pool::CLOSED)?,
        })
    }

    /// Compute user status
    fn get_user_status (&self, pool: &Pool, id: CanonicalAddr) -> StdResult<User> {
        let now = pool.now;
        let registered: Time = self.get_ns(user::REGISTERED, id.as_slice())?.unwrap_or(now);
        if now < registered {
            return Err(StdError::generic_err("no time travel"))
        }
        let existed = now - registered;
        let updated = self.get_ns(user::UPDATED, id.as_slice())?.unwrap_or(now);
        if now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }
        let elapsed = now - updated;
        let locked = self.get_ns(user::LOCKED, id.as_slice())?.unwrap_or(Amount::zero());
        let lifetime: Volume = accumulate(
            self.get_ns(user::LIFETIME, id.as_slice())?.unwrap_or(Volume::zero()),
            elapsed,
            locked
        )?;
        let liquid: Time = self.get_ns(user::PRESENT, id.as_slice())?.unwrap_or(0) +
            if locked > Amount::zero() { elapsed } else { 0 };
        let budget = Amount::from(pool.vested)
            .diminish_or_zero(pool.liquid, pool.existed.unwrap_or(0))?
            .diminish_or_zero(pool.global_ratio.0, pool.global_ratio.1)?;
        let earned = Volume::from(budget)
            .diminish_or_zero(lifetime, pool.lifetime)?
            .diminish_or_zero(liquid, existed)?
            .low_u128().into();
        let claimed = self.get_ns(user::CLAIMED, id.as_slice())?.unwrap_or(Amount::zero());
        let mut reason: Option<&str> = None;
        let cooldown = self.get_ns(user::COOLDOWN, id.as_slice())?.unwrap_or(0);
        let claimable = if liquid < pool.threshold {
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
            existed, liquid, presence: (liquid, existed),
            updated, locked, momentary_share: (locked, pool.locked),
            lifetime, lifetime_share: (lifetime, pool.lifetime),
            earned, claimed, claimable, cooldown,
            reason: reason.map(|x|x.to_string())
        })
    }

}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsHandle {
    Configure(RewardsConfig),

    Close { message: String },
    Drain {
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    },

    Lock     { amount: Amount },
    Retrieve { amount: Amount },
    Claim {},
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

pub type CloseSeal = (Time, String);

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Pool {
    /// "For what point in time do the following values hold true?"
    /// Passed on instantiation.
    pub now:          Time,

    /// "When was this pool deployed?"
    /// Set to current time on init.
    pub deployed:     Time,

    /// "Is this pool closed, and if so, when and why?"
    /// Set irreversibly via handle method.
    pub closed:       Option<CloseSeal>,

    /// "When were LP tokens first locked?"
    /// Set to current time on first lock.
    pub seeded:       Option<Time>,

    /// "For how many units of time has this pool existed?"
    /// Computed as now - seeded
    pub existed:      Option<Time>,

    /// "When was the last time someone locked or unlocked tokens?"
    /// Set to current time on lock/unlock.
    pub updated:      Time,

    /// Used to compute what portion of the time the pool was not empty.
    /// Before lock/unlock, if locked > 0, this is incremented
    /// by pool.elapsed
    pub liquid:       Time,

    /// "What liquidity has this pool contained up to this point?"
    /// Before lock/unlock, if locked > 0, this is incremented
    /// by pool.elapsed * pool.locked
    pub lifetime:     Volume,

    /// "What liquidity is there in the whole pool right now?"
    /// Incremented/decremented on lock/unlock.
    pub locked:       Amount,

    /// "What reward balance is there in the pool?"
    /// Queried from reward token.
    pub balance:      Amount,

    /// "What rewards has everyone received so far?"
    /// Incremented on claim.
    pub claimed:      Amount,

    /// "What rewards were unlocked for this pool so far?"
    /// Computed as balance + claimed.
    pub vested:       Amount,

    /// "What is the initial bonding period?"
    /// User needs to lock LP tokens for this amount of time
    /// before rewards can be claimed. Configured on init.
    pub threshold:    Time,

    /// "How much must the user wait between claims?"
    /// Configured on init.
    /// User cooldowns are reset to this value on claim.
    pub cooldown:     Time,

    pub global_ratio:    (Amount, Amount),
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct User {
    /// When did this user first provide liquidity?
    /// Set to current time on first update.
    pub registered:      Time,

    /// How much time has passed since this user became known to the contract?
    /// Computed as pool.now - user.registered
    pub existed:         Time,

    /// When did this user's liquidity amount last change?
    /// Set to current time on update.
    pub updated:         Time,

    /// For how much time this user has provided non-zero liquidity?
    /// Incremented on update by user.elapsed if user.locked > 0
    pub liquid:          Time,

    pub presence: (Time, Time),

    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    pub locked:          Amount,

    /// What portion of the pool is this user currently contributing?
    /// Computed as user.locked / pool.locked
    pub momentary_share: (Amount, Amount),

    /// How much liquidity has this user provided since they first appeared?
    /// Incremented on update by user.locked * elapsed if user.locked > 0
    pub lifetime:        Volume,

    /// What portion of all the liquidity has this user ever contributed?
    /// Computed as user.lifetime / pool.lifetime
    pub lifetime_share:  (Volume, Volume),

    /// How much rewards has this user earned?
    /// Computed as user.lifetime_share * pool.vested
    pub earned:          Amount,

    /// How much rewards has this user claimed so far?
    /// Incremented on claim by the amount claimed.
    pub claimed:         Amount,

    /// How much rewards can this user claim?
    /// Computed as user.earned - user.claimed, clamped at 0.
    pub claimable:       Amount,

    /// User-friendly reason why claimable is 0
    pub reason:          Option<String>,

    /// How many units of time remain until the user can claim again?
    /// Decremented on lock/unlock, reset to pool.cooldown on claim.
    pub cooldown:        Time,
}

fn enforce_cooldown (elapsed: Time, cooldown: Time) -> StdResult<()> {
    if elapsed >= cooldown {
        Ok(())
    } else {
        Err(StdError::generic_err(format!(
            "deposit tokens for {} more blocks to be eligible", cooldown - elapsed
        )))
    }
}

pub mod pool {
    pub const CLAIMED:      &[u8] = b"/pool/claimed";
    pub const CLOSED:       &[u8] = b"/pool/closed";
    pub const COOLDOWN:     &[u8] = b"/pool/cooldown";
    pub const CREATED:      &[u8] = b"/pool/created";
    pub const DEPLOYED:     &[u8] = b"/pool/deployed";
    pub const LIFETIME:     &[u8] = b"/pool/lifetime";
    pub const LIQUID:       &[u8] = b"/pool/not_empty";
    pub const LOCKED:       &[u8] = b"/pool/balance";
    pub const LP_TOKEN:     &[u8] = b"/pool/lp_token";
    pub const RATIO:        &[u8] = b"/pool/ratio";
    pub const REWARD_TOKEN: &[u8] = b"/pool/reward_token";
    pub const REWARD_VK:    &[u8] = b"/pool/reward_vk";
    pub const SEEDED:       &[u8] = b"/pool/seeded";
    pub const SELF:         &[u8] = b"/pool/self";
    pub const THRESHOLD:    &[u8] = b"/pool/threshold";
    pub const UPDATED:      &[u8] = b"/pool/updated";
}

pub mod user {
    pub const CLAIMED:    &[u8] = b"/user/claimed/";
    pub const COOLDOWN:   &[u8] = b"/user/cooldown/";
    pub const EXISTED:    &[u8] = b"/user/existed/";
    pub const LIFETIME:   &[u8] = b"/user/lifetime/";
    pub const LOCKED:     &[u8] = b"/user/current/";
    pub const PRESENT:    &[u8] = b"/user/present/";
    pub const REGISTERED: &[u8] = b"/user/registered/";
    pub const UPDATED:    &[u8] = b"/user/updated/";
}

pub trait Diminish<T: From<u64> + From<Self>, N: Eq + From<u64>>: Copy {

    /// Divide self on num/denom; throw if num > denom or if denom == 0
    fn diminish         (self, num: N, denom: N) -> StdResult<T>;

    /// Diminish, but return 0 if denom == 0
    fn diminish_or_max  (self, num: N, denom: N) -> StdResult<T> {
        if denom == 0u64.into() {
            Ok(self.into())
        } else {
            self.diminish(num, denom)
        }
    }

    /// Diminish, but return self if denom == 0
    fn diminish_or_zero (self, num: N, denom: N) -> StdResult<T> {
        if denom == 0u64.into() {
            Ok(0u64.into())
        } else {
            self.diminish(num, denom)
        }
    }

}

impl Diminish<Self, Time> for Volume {
    fn diminish (self, num: Time, denom: Time) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function"))
        } else {
            Ok(self.multiply_ratio(num, denom)?)
        }
    }
}

impl Diminish<Self, Volume> for Volume {
    fn diminish (self, num: Volume, denom: Volume) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function"))
        } else {
            Ok(self.multiply_ratio(num, denom)?)
        }
    }
}

impl Diminish<Self, Amount> for Amount {
    fn diminish (self, num: Amount, denom: Amount) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function"))
        } else {
            Ok(self.multiply_ratio(num, denom))
        }
    }
}

impl Diminish<Self, Time> for Amount {
    fn diminish (self, num: Time, denom: Time) -> StdResult<Self> {
        if num > denom {
            Err(StdError::generic_err("num > denom in diminish function"))
        } else {
            Ok(self.multiply_ratio(num, denom))
        }
    }
}
