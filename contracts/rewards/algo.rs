use fadroma::*;
use crate::{auth::Auth, errors};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

/// A moment in time, as represented by the current value of env.block.time
pub type Moment   = u64;

/// A duration of time, represented as a number of moments
pub type Duration = u64;

/// Amount of funds
pub type Amount   = Uint128;

/// Amount multiplied by duration.
pub type Volume   = Uint256;

/// A ratio, represented as tuple (nom, denom)
pub type Ratio    = (Uint128, Uint128);

/// Seconds in 24 hours
pub const DAY: Duration = 86400;

/// Project current value of an accumulating parameter based on stored value,
/// time since it was last updated, and rate of change, i.e.
/// `current = stored + (elapsed * rate)`
pub fn accumulate (
    total_before_last_update: Volume,
    time_since_last_update:   Duration,
    value_after_last_update:  Amount
) -> StdResult<Volume> {
    total_before_last_update + Volume::from(value_after_last_update)
        .multiply_ratio(time_since_last_update, 1u128)?
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

        self.set(config::SELF, &self.canonize(ContractLink {
            address:   env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        })?)?;

        self.handle_configure(&config)?;

        Ok(Some(self.set_own_vk(&config.reward_vk.unwrap())?))

    }

    /// Handle transactions
    fn handle (&mut self, env: &Env, msg: RewardsHandle) -> StdResult<HandleResponse> {
        match msg {
            // Public transactions
            RewardsHandle::Lock     { amount } => self.handle_deposit(env, amount),
            RewardsHandle::Retrieve { amount } => self.handle_withdraw(env, amount),
            RewardsHandle::Claim    {}         => self.handle_claim(env),
            // Admin-only transactions
            _ => {
                Auth::assert_admin(self, env)?;
                match msg {
                    RewardsHandle::Configure(config) => {
                        self.handle_configure(&config)
                    },
                    RewardsHandle::Close { message } => {
                        self.handle_close(env.block.time, message)
                    },
                    RewardsHandle::Drain { snip20, recipient, key } => {
                        self.handle_drain(
                            env.block.time,
                            recipient.unwrap_or(env.message.sender.clone()),
                            snip20,
                            key
                        )
                    },
                    _ => unreachable!()
                }
            }

        }

    }

    /// Store configuration values
    fn handle_configure (&mut self, config: &RewardsConfig) -> StdResult<HandleResponse> {

        let mut messages = vec![];

        if let Some(reward_token) = &config.reward_token {
            self.set(config::REWARD_TOKEN, &self.canonize(reward_token.clone())?)?;
        }
        if let Some(reward_vk) = &config.reward_vk {
            messages.push(self.set_own_vk(&reward_vk)?);
        }
        if let Some(lp_token) = &config.lp_token {
            self.set(config::LP_TOKEN, &self.canonize(lp_token.clone())?)?;
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
        self.set(config::REWARD_VK, &vk)?;
        self.reward_token()?.set_viewing_key(&vk)
    }

    fn self_link (&self) -> StdResult<ContractLink<HumanAddr>> {
        let link = self.get::<ContractLink<CanonicalAddr>>(config::SELF)?
            .ok_or(StdError::generic_err("no self link"))?;
        Ok(self.humanize(link)?)
    }

    fn lp_token (&self) -> StdResult<ISnip20> {
        let link = self.get::<ContractLink<CanonicalAddr>>(config::LP_TOKEN)?
            .ok_or(StdError::generic_err("no lp token"))?;
        Ok(ISnip20::attach(self.humanize(link)?))
    }

    fn reward_token (&self) -> StdResult<ISnip20> {
        let link = self.get::<ContractLink<CanonicalAddr>>(config::REWARD_TOKEN)?
            .ok_or(StdError::generic_err("no reward token"))?;
        Ok(ISnip20::attach(self.humanize(link)?))
    }

    fn reward_vk (&self) -> StdResult<String> {
        Ok(self.get::<ViewingKey>(config::REWARD_VK)?
            .ok_or(StdError::generic_err("no reward viewing key"))?
            .0)
    }

    /// Compute pool status.
    fn get_pool_status (&self, now: Moment) -> StdResult<Pool> {
        let mut pool = Pool::default();
        pool.now = now;
        pool.updated = self.get(pool::UPDATED)?.unwrap_or(now);
        if pool.now < pool.updated {
            return errors::no_time_travel()
        }
        let elapsed = now - pool.updated;
        pool.staked = self.get(pool::STAKED)?.unwrap_or(Amount::zero());
        pool.volume = accumulate(
            self.get(pool::VOLUME)?.unwrap_or(Volume::zero()),
            elapsed,
            pool.staked
        )?;
        let lp_token = self.lp_token()?;
        let reward_token = self.reward_token()?;
        pool.budget = reward_token.query_balance(
            self.querier(),
            &self.self_link()?.address,
            &self.reward_vk()?
        )?;
        if reward_token.link == lp_token.link { // separate balances for single-sided staking
            pool.budget = (pool.budget - pool.staked)?;
        }
        pool.claimed = self.get(pool::CLAIMED)?.unwrap_or(Amount::zero());
        pool.vested = pool.claimed + pool.budget;
        pool.global_ratio = self.get(pool::RATIO)?
            .ok_or(StdError::generic_err("missing global ratio"))?;
        Ok(pool)
    }

    /// Compute user status
    fn get_user_status (&self, pool: &Pool, id: &CanonicalAddr) -> StdResult<User> {
        let mut user = User::default();
        user.updated = self.get_ns(user::UPDATED, id.as_slice())?.unwrap_or(pool.now);
        if pool.now < user.updated {
            return errors::no_time_travel()
        }
        let elapsed: Duration = pool.now - user.updated;
        user.entry = self.get_ns(user::ENTRY, id.as_slice())?.unwrap_or(pool.volume);
        if user.entry > pool.volume {
            return errors::no_time_travel()
        }
        user.staked = self.get_ns(user::STAKED, id.as_slice())?.unwrap_or(Amount::zero());
        user.volume = accumulate(
            self.get_ns(user::VOLUME, id.as_slice())?.unwrap_or(Volume::zero()),
            elapsed,
            user.staked
        )?;
        user.pool_share = (user.staked, pool.staked);
        user.reward_share = (user.volume, (pool.volume - user.entry)?);
        user.earned = if user.reward_share.1 == Volume::zero() {
            Amount::zero()
        } else {
            Volume::from(pool.budget)
                .multiply_ratio(user.reward_share.0, user.reward_share.1)?
                .low_u128().into()
        };
        user.claimed = self.get_ns(user::CLAIMED, id.as_slice())?.unwrap_or(Amount::zero());
        user.cooldown = self.get_ns(user::COOLDOWN, id.as_slice())?.unwrap_or(0);
        if user.staked > Amount::zero() {
            user.cooldown = user.cooldown - u64::min(elapsed, user.cooldown)
        };
        user.claimable = if user.earned == Amount::zero() {
            user.reason = Some("can only claim positive earnings".to_string());
            Amount::zero()
        } else if user.earned <= user.claimed {
            user.reason = Some("can only claim if earned > claimed".to_string());
            Amount::zero()
        } else {
            user.claimable = (user.earned - user.claimed)?;
            if user.claimable > pool.budget {
                user.reason = Some("can't claim more than the remaining pool budget".to_string());
                pool.budget
            } else {
                user.claimable
            }
        };
        Ok(user)
    }

    fn get_status (&mut self, env: &Env) -> StdResult<(Pool, User, CanonicalAddr)> {
        // Compute pool state
        let now = env.block.time;
        let pool = self.get_pool_status(now)?;
        if pool.updated > now {
            return errors::no_time_travel()
        }
        // Compute user state
        let id = self.canonize(env.message.sender.clone())?;
        let user = self.get_user_status(&pool, &id)?;
        if user.updated > now {
            return errors::no_time_travel()
        }
        Ok((pool, user, id))
    }

    /// Deposit LP tokens from user into pool
    fn handle_deposit (&mut self, env: &Env, amount: Amount) -> StdResult<HandleResponse> {
        let (ref mut pool, ref mut user, ref id) = self.get_status(&env)?;
        if pool.closed.is_some() {
            self.return_stake(env, pool, user)
        } else {
            if user.staked == Amount::zero() {
                self.reset(id)?;
                user.entry = pool.volume;
                self.set_ns::<Volume>(user::ENTRY, id.as_slice(), user.entry)?;
            }
            user.staked += amount;
            pool.staked += amount;
            self.commit_staked(pool.staked, user.staked, &id)?;
            self.commit_state(pool, user, id)?;
            HandleResponse::default().msg(self.lp_token()?.transfer_from(
                &env.message.sender,
                &env.contract.address,
                amount
            )?)
        }
    }

    /// Withdraw deposited LP tokens from pool back to the user
    fn handle_withdraw (&mut self, env: &Env, amount: Uint128) -> StdResult<HandleResponse> {
        let (ref mut pool, ref mut user, ref id) = self.get_status(&env)?;
        if pool.closed.is_some() {
            self.return_stake(env, pool, user)
        } else if user.staked < amount {
            errors::withdraw(user.staked, amount)
        } else if pool.staked < amount {
            errors::withdraw_fatal(pool.staked, amount)
        } else {
            user.staked = (user.staked - amount)?;
            pool.staked = (pool.staked - amount)?;
            self.commit_staked(pool.staked, user.staked, &id)?;
            self.commit_state(pool, user, id)?;
            if user.staked == Amount::zero() && user.claimable > Amount::zero() {
            }
            HandleResponse::default().msg(self.lp_token()?.transfer(
                &env.message.sender,
                amount
            )?)
        }
    }

    /// Transfer rewards to user if eligible
    fn handle_claim (&mut self, env: &Env) -> StdResult<HandleResponse> {
        let (ref mut pool, ref mut user, ref id) = self.get_status(&env)?;
        if user.cooldown > 0 {
            errors::claim_cooldown(user.cooldown)
        } else if pool.budget == Amount::zero() {
            errors::claim_pool_empty()
        } else if pool.global_ratio.0 == Amount::zero() {
            errors::claim_global_ratio_zero()
        } else if user.claimed > user.earned {
            errors::claim_crowded_out()
        } else if user.claimable == Amount::zero() {
            errors::claim_zero_claimable()
        } else {
            user.cooldown = pool.cooldown;
            self.commit_claimed(pool, user, id)?;
            self.commit_state(pool, user, id)?;
            self.reset(id)?;
            // Transfer reward tokens from the contract to the user
            HandleResponse::default().msg(self.reward_token()?.transfer(
                &env.message.sender,
                user.claimable)?
            )
        }
    }

    /// Admin can mark pool as closed
    fn handle_close (&mut self, time: Moment, message: String) -> StdResult<HandleResponse> {
        self.set(pool::CLOSED, Some((time, message)))?;
        Ok(HandleResponse::default())
    }

    /// Closed pools can be drained for manual redistribution of erroneously staked funds.
    fn handle_drain (
        &mut self,
        time:      Moment,
        recipient: HumanAddr,
        snip20:    ContractLink<HumanAddr>,
        key:       String
    ) -> StdResult<HandleResponse> {
        // Update the viewing key if the supplied
        // token info for is the reward token
        if self.reward_token()?.link == snip20 {
            self.set(config::REWARD_VK, key.clone())?
        }
        let allowance = Uint128(u128::MAX);
        let duration  = Some(time + DAY * 10000);
        let snip20    = ISnip20::attach(snip20);
        HandleResponse::default()
            .msg(snip20.increase_allowance(&recipient, allowance, duration)?)?
            .msg(snip20.set_viewing_key(&key)?)
    }

    /// Closed pools return all funds upon request and prevent further deposits
    fn return_stake (
        &mut self, env: &Env, pool: &mut Pool, user: &mut User
    ) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = pool.closed {
            let withdraw_all = user.staked;
            user.staked = 0u128.into();
            pool.staked = (pool.staked - withdraw_all)?;
            let id = self.canonize(env.message.sender.clone())?;
            self.commit_staked(pool.staked, user.staked, &id)?;
            HandleResponse::default()
                .msg(self.lp_token()?.transfer(&env.message.sender.clone(), withdraw_all)?)?
                .log("closed", &format!("{} {}", when, why))
        } else {
            Err(StdError::generic_err("pool not closed"))
        }
    }

    /// Commit amount of staked tokens for user and pool
    fn commit_staked (
        &mut self, pool_staked: Amount, user_staked: Amount, id: &CanonicalAddr
    ) -> StdResult<()> {
        self.set_ns(user::STAKED, id.as_slice(), user_staked)?;
        self.set(pool::STAKED, pool_staked)
    }

    fn commit_claimed (
        &mut self, pool: &mut Pool, user: &mut User, id: &CanonicalAddr
    ) -> StdResult<()> {
        user.claimed += user.claimable;
        pool.claimed += user.claimable;
        self.set_ns(user::CLAIMED, id.as_slice(), user.claimed)?;
        self.set(pool::CLAIMED, pool.claimed)?;
        Ok(())
    }

    fn commit_state (
        &mut self, pool: &mut Pool, user: &mut User, id: &CanonicalAddr
    ) -> StdResult<()> {
        // Commit pool state
        self.set(pool::VOLUME, pool.volume)?;
        self.set(pool::UPDATED,  pool.now)?;
        // Commit user state
        self.set_ns(user::COOLDOWN, id.as_slice(), user.cooldown)?;
        self.set_ns(user::VOLUME, id.as_slice(), user.volume)?;
        self.set_ns(user::UPDATED,  id.as_slice(), pool.now)?;
        Ok(())
    }

    /// Reset the user's liquidity conribution
    fn reset (&mut self, id: &CanonicalAddr) -> StdResult<()> {
        self.set_ns(user::ENTRY,    id.as_slice(), Volume::zero())?;
        self.set_ns(user::VOLUME, id.as_slice(), Volume::zero())?;
        self.set_ns(user::CLAIMED,  id.as_slice(), Amount::zero())?;
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
        &self, now: Moment, address: Option<HumanAddr>, key: Option<String>
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
            Some(self.get_user_status(&pool, &id)?)
        } else {
            None
        };
        Ok(RewardsResponse::Status { time: now, pool, user })

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
    pub threshold:    Option<Duration>,
    pub cooldown:     Option<Duration>
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsQuery {
    Status {
        at:      Moment,
        address: Option<HumanAddr>,
        key:     Option<String>
    }
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsResponse {
    Status {
        time: Moment,
        pool: Pool,
        user: Option<User>
    }
}

pub type CloseSeal = (Moment, String);

pub mod config {
    pub const LP_TOKEN:     &[u8] = b"/pool/lp_token";
    pub const REWARD_TOKEN: &[u8] = b"/pool/reward_token";
    pub const REWARD_VK:    &[u8] = b"/pool/reward_vk";
    pub const SELF:         &[u8] = b"/pool/self";
}

/// Pool status
///
/// 1. Timestamps
///
///     This contract acts as a price discovery mechanism by
///     distributing funds over time in response to market activity.
///
///     It refers to the following points in time:
///
///     * `now`. The current moment.
///       * Received from transaction environment
///       * For queries, passed by the user.
///
///     * `updated`. The moment of the last update (lock, retrieve, or claim).
///       * Defaults to current time.
///
///     * `elapsed`. Moments elapsed since last update.
///       * Equal to `now - updated`.
///       * Defaults to zero.
///
///     A "moment" corresponds to a block in v2 and a second in v3.
///
/// 2. Global ratio
///
///    This can be configured by the admin to
///    manually boost or reduce reward distribution.
///
/// 3. Liquidity in pool
///
///     When users lock tokens in the pool, liquidity accumulates.
///
///     Liquidity is defined as amount of tokens multiplied by time.
///
///     * Starting with a new pool, lock 10 LP for 20 moments.
///       The pool will have a liquidity of 200.
///       Lock 10 more and 5 moments later the liquidity will be 300.
///
///     Pool liquidity is internally represented by two variables:
///
///     * `staked` is the total number of LP tokens
///       that are currently staked in the pool.
///       * Incremented and decremented on withdraws and deposits.
///       * Should be equal to this contract's balance in the
///         LP token contract.
///
///     * `volume`. The total amount of liquidity
///       contained by the pool over its volume.
///       * Incremented by `elapsed * staked` on deposits and withdrawals.
///       * Computed as `last_value + elapsed * staked` on queries.
///
/// 4. Reward budget
///
///     The pool queries its `balance` in reward tokens from the reward token
///     contract.
///
///     * In the case of **single-sided staking** (e.g. staking SIENNA to earn SIENNA)
///       the value of `staked` is subtracted from this balance in order to separate
///       the tokens staked by users from the reward budget.
///
///     Rewards are computed on the basis of this balance.
///
///     * This was the cause of issues around the launch of v2, as we had
///       neglected the fact that a large balance had already accumulated.
///       This would've distributed the rewards for a few weeks in one go
///       to the earliest users, rather than computing their fair liquidity share
///       over time.
///
///     The pool also keeps track of how much rewards have been distributed,
///     in the `claimed` variable which is incremented on successful claims.
///
///     `vested` is equal to `balance + claimed` and is informative.
///     
///     This is the other set of variables that can be coupled to an epoch clock,
///     in order to define a maximum amount of rewards per epoch.
///
/// 6. Throttles
///
///     * Age threshold (initial bonding period)
///     * Cooldown (minimum time between claims)
///
///     By default, each is equal to the epoch duration.
///     Other values can be configured by the admin.
///
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Pool {
    /// "For what point in time do the following values hold true?"
    /// Passed on instantiation.
    pub now:          Moment,
    /// "Is this pool closed, and if so, when and why?"
    /// Set irreversibly via handle method.
    pub closed:       Option<CloseSeal>,
    /// "When was the last time someone staked or unstaked tokens?"
    /// Set to current time on lock/unlock.
    pub updated:      Moment,
    /// "What liquidity has this pool contained up to this point?"
    /// Before lock/unlock, if staked > 0, this is incremented
    /// by pool.elapsed * pool.staked
    pub volume:       Volume,
    /// "What liquidity is there in the whole pool right now?"
    /// Incremented/decremented on lock/unlock.
    pub staked:       Amount,
    /// "What amount of rewards is currently available for users?"
    /// Queried from reward token.
    pub budget:       Amount,
    /// "What rewards has everyone received so far?"
    /// Incremented on claim.
    pub claimed:      Amount,
    /// "What rewards were unstaked for this pool so far?"
    /// Computed as balance + claimed.
    pub vested:       Amount,
    /// "What is the initial bonding period?"
    /// User needs to lock LP tokens for this amount of time
    /// before rewards can be claimed. Configured on init.
    pub threshold:    Duration,
    /// "How much must the user wait between claims?"
    /// Configured on init.
    /// User cooldowns are reset to this value on claim.
    pub cooldown:     Duration,
    /// Used to throttle the pool.
    pub global_ratio: (Amount, Amount),
}

pub mod pool {
    pub const CLOSED:    &[u8] = b"/pool/closed";
    pub const RATIO:     &[u8] = b"/pool/ratio";
    pub const COOLDOWN:  &[u8] = b"/pool/cooldown";
    pub const THRESHOLD: &[u8] = b"/pool/threshold";
    pub const VOLUME:    &[u8] = b"/pool/volume";
    pub const UPDATED:   &[u8] = b"/pool/updated";
    pub const STAKED:    &[u8] = b"/pool/size";
    pub const CLAIMED:   &[u8] = b"/pool/claimed";
}

/// User status
///
/// 1. Timestamps
///
///     Each user earns rewards as a function of their liquidity contribution over time.
///     The following points and durations in time are stored for each user:
///
///     * `updated` is the time of last update (deposit, withdraw or claim by this user)
///
/// 2. Liquidity and liquidity share
///
///     * `staked` is the number of LP tokens staked by this user in this pool.
///     * The user's **momentary share** is defined as `staked / pool.staked`.
///     * `volume` is the volume liquidity contributed by this user.
///       It is incremented by `staked` for every moment elapsed.
///     * The user's **volume share** is defined as `volume / pool.volume`.
///       It represents the user's overall contribution, and should move in the
///       direction of the user's momentary share.
///
/// 3. Rewards claimable
///
///     * `earned` rewards are equal to `volume_share * user_liquidity_ratio *
///     pool_liquidity_ratio * global_ratio * pool.budget`.
///     * `claimed` rewards are incremented on each claim, by the amount claimed.
///     * `claimable` is equal to `earned - claimed`.
///
///     As the user's volume share increases (as a result of providing liquidity)
///     or the pool's budget increases (as a result of new reward portions being
///     unstaked from the TGE budget), new rewards are `earned` and become `claimable`.
///
///     `earned` may become less than `claimed` if the user's volume share
///     goes down too steeply:
///
///         * as a result of that user withdrawing liquidity;
///
///         * or as a result of an influx of liquidity by other users
///
///     This means the user has been *crowded out* - they have already claimed
///     fair rewards for their contribution up to this point, but have become
///     ineligible for further rewards until their volume share increases:
///
///         * as a result of that user providing a greater amount of liquidity
///
///         * as a result of other users withdrawing liquidity
///
///     and/or until the pool's balance increases:
///
///         * as a result of incoming reward portions from the TGE budget.
///
pub mod user {
    pub const ENTRY:    &[u8] = b"/user/entry/";
    pub const STAKED:   &[u8] = b"/user/current/";
    pub const UPDATED:  &[u8] = b"/user/updated/";
    pub const VOLUME:   &[u8] = b"/user/volume/";
    pub const CLAIMED:  &[u8] = b"/user/claimed/";
    pub const COOLDOWN: &[u8] = b"/user/cooldown/";
}

#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct User {
    /// What was the volume liquidity of the pool when the user entered?
    /// User's reward share is computed from liquidity accumulated over that amount.
    pub entry:        Volume,
    /// When did this user's liquidity amount last change?
    /// Set to current time on update.
    pub updated:      Moment,
    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    pub staked:       Amount,
    /// How much liquidity has this user provided since they first appeared?
    /// Incremented on update by user.staked * elapsed if user.staked > 0
    pub volume:       Volume,
    /// How much rewards has this user earned?
    /// Computed as user.volume_share * pool.vested
    pub earned:       Amount,
    /// How much rewards has this user claimed so far?
    /// Incremented on claim by the amount claimed.
    pub claimed:      Amount,
    /// How much rewards can this user claim?
    /// Computed as user.earned - user.claimed, clamped at 0.
    pub claimable:    Amount,
    /// User-friendly reason why claimable is 0
    pub reason:       Option<String>,
    /// How many units of time remain until the user can claim again?
    /// Decremented on lock/unlock, reset to pool.cooldown on claim.
    pub cooldown:     Duration,
    /// What portion of the pool is this user currently contributing?
    /// Computed as user.staked / pool.staked
    pub pool_share:   (Amount, Amount),
    /// What portion of all the liquidity has this user ever contributed?
    /// Computed as user.volume / pool.volume
    pub reward_share: (Volume, Volume),
    #[serde(skip)]
    /// Passed around internally, not presented to user.
    pub id: CanonicalAddr,
}
