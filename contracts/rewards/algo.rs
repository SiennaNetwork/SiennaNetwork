use fadroma::{*, scrt_uint256::Uint256};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use crate::auth::Auth;

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

/// 100% with 6 digits after the decimal
pub const HUNDRED_PERCENT: u128 = 100000000u128;

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

    fn before_user_action (&mut self, env: &Env) -> StdResult<(Pool, User)> {
        // Compute pool state
        let now = env.block.time;
        let mut pool = self.get_pool_status(now)?;
        if pool.updated > now {
            return Err(StdError::generic_err("no time travel"))
        }
        // Compute user state
        let id = self.canonize(env.message.sender.clone())?;
        let mut user = self.get_user_status(&pool, &id)?;
        if user.updated > now {
            return Err(StdError::generic_err("no time travel"))
        }
        Ok((pool, user))
    }

    fn after_user_action (
        &mut self, env: &Env, pool: &mut Pool, user: &mut User, id: &CanonicalAddr
    ) -> StdResult<()> {
        // Commit pool state
        match pool.seeded {
            None => {
                // If this is the first time someone is locking tokens in this pool,
                // store the timestamp. This is used to start the pool liquidity ratio
                // calculation from the time of first lock instead of from the time
                // of contract init.
                // * Using is_none here fails type inference.
                self.set(pool::SEEDED, pool.now)?;
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
        self.set(pool::UPDATED,  pool.now)?;
        // Commit user state
        self.set_ns(user::EXISTED,  id.as_slice(), user.existed)?;
        self.set_ns(user::PRESENT,  id.as_slice(), user.liquid)?;
        self.set_ns(user::COOLDOWN, id.as_slice(), user.cooldown)?;
        self.set_ns(user::LIFETIME, id.as_slice(), user.lifetime)?;
        self.set_ns(user::UPDATED,  id.as_slice(), pool.now)?;
        Ok(())
    }

    /// Deposit LP tokens from user into pool
    fn handle_deposit (&mut self, env: &Env, deposited: Amount) -> StdResult<HandleResponse> {
        let (mut pool, mut user) = self.before_user_action(&env)?;
        if pool.closed.is_some() {
            self.return_stake(env, &mut pool, &mut user)
        } else {
            // Set user registration date if this is their first deposit
            let id = self.canonize(env.message.sender.clone())?;
            if self.get_ns::<Moment>(user::REGISTERED, id.as_slice())?.is_none() {
                self.set_ns(user::REGISTERED, id.as_slice(), pool.now)?
            }
            // Increment user and pool liquidity
            user.locked += deposited;
            pool.locked += deposited;
            self.update_locked(&pool, &user, &id)?;
            self.after_user_action(env, &mut pool, &mut user, &id)?;
            // Transfer liquidity provision tokens from the user to the contract
            HandleResponse::default()
                .msg(self.lp_token()?.transfer_from(
                    &env.message.sender,
                    &env.contract.address,
                    deposited
                )?)
        }
    }

    /// Withdraw deposited LP tokens from pool back to the user
    fn handle_withdraw (&mut self, env: &Env, withdrawn: Uint128) -> StdResult<HandleResponse> {
        let (mut pool, mut user) = self.before_user_action(&env)?;
        if pool.closed.is_some() {
            self.return_stake(env, &mut pool, &mut user)
        } else if user.locked < withdrawn {
            self.err_withdraw(user.locked, withdrawn)
        } else if pool.locked < withdrawn {
            self.err_withdraw_fatal(pool.locked, withdrawn)
        } else {
            // Decrement user and pool liquidity
            pool.locked = (pool.locked - withdrawn)?;
            user.locked = (user.locked - withdrawn)?;
            let id = self.canonize(env.message.sender.clone())?;
            self.update_locked(&pool, &user, &id)?;
            self.after_user_action(env, &mut pool, &mut user, &id)?;
            // Transfer liquidity provision tokens from the contract to the user
            HandleResponse::default()
                .msg(self.lp_token()?.transfer(
                    &env.message.sender,
                    withdrawn
                )?)
        }
    }

    /// Commit amount of locked tokens for user and pool
    fn update_locked (
        &mut self, pool: &Pool, user: &User, id: &CanonicalAddr
    ) -> StdResult<()> {
        self.set_ns(user::LOCKED, id.as_slice(), user.locked)?;
        self.set(pool::LOCKED, pool.locked)
    }

    fn err_withdraw (&self, locked: Amount, withdrawn: Amount) -> StdResult<HandleResponse> {
        // User must have enough locked to retrieve
        Err(StdError::generic_err(format!(
            "not enough locked ({} < {})", locked, withdrawn
        )))
    }

    fn err_withdraw_fatal (&self, locked: Amount, withdrawn: Amount) -> StdResult<HandleResponse> {
        // If pool does not have enough lp tokens then something has gone badly wrong
        Err(StdError::generic_err(format!(
            "FATAL: not enough tokens in pool ({} < {})", locked, withdrawn
        )))
    }

    /// Transfer rewards to user if eligible
    fn handle_claim (&mut self, env: &Env) -> StdResult<HandleResponse> {
        let (mut pool, mut user) = self.before_user_action(&env)?;
        if user.liquid < pool.threshold {
            self.err_claim_threshold(pool.threshold, user.liquid)
        } else if user.cooldown > 0 {
            self.err_claim_cooldown(user.cooldown)
        } else if pool.balance == Amount::zero() {
            self.err_claim_pool_empty()
        } else if pool.global_ratio.0 == Amount::zero() {
            self.err_claim_global_ratio_zero()
        } else if user.claimed > user.earned {
            self.err_claim_crowded_out()
        } else if user.claimable == Amount::zero() {
            self.err_claim_zero_claimable()
        } else {
            let id = self.canonize(env.message.sender.clone())?;
            // Increment claimed counters
            user.claimed += user.claimable;
            pool.claimed += user.claimable;
            self.set_ns(user::CLAIMED, id.as_slice(), user.claimed)?;
            self.set(pool::CLAIMED, pool.claimed)?;
            // Reset user cooldown countdown to pool cooldown value
            user.cooldown = pool.cooldown;
            self.after_user_action(env, &mut pool, &mut user, &id)?;
            if user.locked == Amount::zero() {
                self.reset_user_data(&id);
            }
            // Transfer reward tokens from the contract to the user
            HandleResponse::default()
                .msg(self.reward_token()?.transfer(&env.message.sender, user.claimable)?)
        }
    }

    fn reset_user_data (&mut self, id: &CanonicalAddr) -> StdResult<()> {
        self.set_ns(user::LIFETIME, id.as_slice(), Volume::zero());
        self.set_ns(user::CLAIMED,  id.as_slice(), Amount::zero())
    }

    fn err_claim_threshold (&self, threshold: Duration, liquid: Duration) -> StdResult<HandleResponse> {
        Err(StdError::generic_err(format!(
            "You must keep some tokens locked for {} more seconds \
            before you are able to claim for the first time.",
            threshold - liquid
        )))
    }

    fn err_claim_cooldown (&self, cooldown: Duration) -> StdResult<HandleResponse> {
        Err(StdError::generic_err(format!(
            "You must keep some tokens locked for {} more seconds \
            before you are able to claim again.",
            cooldown
        )))
    }

    fn err_claim_pool_empty (&self) -> StdResult<HandleResponse> {
        Err(StdError::generic_err(
            "This pool is currently empty. \
            However, liquidity shares continue to accumulate."
        ))
    }

    fn err_claim_global_ratio_zero (&self) -> StdResult<HandleResponse> {
        Err(StdError::generic_err(
            "Rewards from this pool are currently stopped. \
            However, liquidity shares continue to accumulate."
        ))
    }

    fn err_claim_crowded_out (&self) -> StdResult<HandleResponse> {
        Err(StdError::generic_err(
            "Your liquidity share has steeply diminished \
            since you last claimed. Lock more tokens to get \
            to the front of the queue faster."
        ))
    }

    fn err_claim_zero_claimable (&self) -> StdResult<HandleResponse> {
        Err(StdError::generic_err(
            "You have already claimed your exact share of the rewards."
        ))
    }

    /// Admin can mark pool as closed
    fn handle_close (&mut self, env: &Env, message: String) -> StdResult<HandleResponse> {
        Auth::assert_admin(self, &env)?;
        self.set(pool::CLOSED, Some((env.block.time, message)))?;
        Ok(HandleResponse::default())
    }

    /// Closed pools return all funds upon request and prevent further deposits
    fn return_stake (
        &mut self, env: &Env, pool: &mut Pool, user: &mut User
    ) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = pool.closed {
            let withdraw_all = user.locked;
            user.locked = 0u128.into();
            pool.locked = (pool.locked - withdraw_all)?;
            let id = self.canonize(env.message.sender.clone())?;
            self.update_locked(&pool, &user, &id)?;
            HandleResponse::default()
                .msg(self.lp_token()?.transfer(&env.message.sender.clone(), withdraw_all)?)?
                .log("closed", &format!("{} {}", when, why))
        } else {
            Err(StdError::generic_err("pool not closed"))
        }
    }

    /// Closed pools can be drained for manual redistribution of erroneously locked funds.
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
        HandleResponse::default()
            .msg(snip20.increase_allowance(&recipient, allowance, duration)?)?
            .msg(snip20.set_viewing_key(&key)?)
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

    /// Compute pool status.
    fn get_pool_status (&self, now: Moment) -> StdResult<Pool> {
        let seeded: Option<Moment> =
            self.get(pool::SEEDED)?;
        let updated: Moment =
            self.get(pool::UPDATED)?.unwrap_or(now);

        let existed: Option<Duration> = if let Some(seeded) = seeded {
            if now < seeded {
                return Err(StdError::generic_err("no time travel"))
            }
            Some(now - seeded)
        } else {
            None
        };
        if now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }
        let elapsed: Duration =
            now - updated;
        let locked: Amount =
            self.get(pool::LOCKED)?.unwrap_or(Amount::zero());
        let last_lifetime: Volume =
            self.get(pool::LIFETIME)?.unwrap_or(Volume::zero());
        let lifetime: Volume =
            accumulate(last_lifetime, elapsed, locked)?;
        let liquid: Duration =
            self.get(pool::LIQUID)?.unwrap_or(0) + if locked > Amount::zero() {
                elapsed
            } else {
                0
            };

        let lp_token =
            self.lp_token()?;
        let reward_token =
            self.reward_token()?;
        let mut balance =
            reward_token.query_balance(
                self.querier(), &self.self_link()?.address, &self.reward_vk()?
            )?;
        if reward_token.link == lp_token.link {
            // separate balances for single-sided staking
            balance = (balance - locked)?;
        }

        let claimed =
            self.get(pool::CLAIMED)?.unwrap_or(Amount::zero());
        let vested =
            claimed + balance;

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
    fn get_user_status (&self, pool: &Pool, id: &CanonicalAddr) -> StdResult<User> {
        let registered: Moment =
            self.get_ns(user::REGISTERED, id.as_slice())?.unwrap_or(pool.now);
        let updated: Moment =
            self.get_ns(user::UPDATED, id.as_slice())?.unwrap_or(pool.now);

        if pool.now < registered {
            return Err(StdError::generic_err("no time travel"))
        }
        let existed: Duration =
            pool.now - registered;
        if pool.now < updated {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        let elapsed =
            pool.now - updated;
        let locked: Amount =
            self.get_ns(user::LOCKED, id.as_slice())?.unwrap_or(Amount::zero());
        let last_lifetime: Volume =
            self.get_ns(user::LIFETIME, id.as_slice())?.unwrap_or(Volume::zero());
        let lifetime: Volume =
            accumulate(last_lifetime, elapsed, locked)?;
        let liquid: Duration = self.get_ns(user::PRESENT, id.as_slice())?.unwrap_or(0) +
            if locked > Amount::zero() { elapsed } else { 0 };

        let budget = if pool.existed.unwrap_or(0) == 0 {
            Amount::zero()
        } else if pool.global_ratio.1 == Amount::zero() {
            Amount::zero()
        } else {
            Amount::from(pool.vested)
                .multiply_ratio(pool.liquid, pool.existed.unwrap_or(0))
                .multiply_ratio(pool.global_ratio.0, pool.global_ratio.1)
        };

        let earned = if pool.lifetime == Volume::zero() {
            Amount::zero()
        } else if existed == 0 {
            Amount::zero()
        } else {
            Volume::from(budget)
                .multiply_ratio(lifetime, pool.lifetime)?
                .multiply_ratio(liquid, existed)?
                .low_u128().into()
        };

        let claimed = self.get_ns(user::CLAIMED, id.as_slice())?.unwrap_or(Amount::zero());

        let mut cooldown = self.get_ns(user::COOLDOWN, id.as_slice())?.unwrap_or(0);
        if locked > Amount::zero() {
            cooldown = cooldown - u64::min(elapsed, cooldown)
        };

        let mut reason: Option<&str> = None;
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
///     * `seeded`. The moment of the first deposit.
///       * Set to current time on first successful deposit tx.
///
///     * `existed`. The number of moments since the first deposit.
///       * Equal to `now - seeded`.
///
///     * `liquid`. The number of moments since first deposit,
///       for which the pool was not empty.
///       * Incremented on update if pool is not empty.
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
/// 3. Pool liquidity ratio
///
///     Rewards should only be distributed for the time liquidity was provided.
///
///     For the moments the pool is empty, no rewards should be distributed.
///
///     This is represented by the pool liquidity ratio, equal to `liquid / existed`.
///
///     * A pool that has been liquid 100% of the time must
///       distribute 100% of the rewards per epoch.
///
///     * A pool that was empty for 10% of the time will distribute
///       90% of the rewards per epoch.
///
///     * To get the maximum of rewards per epoch, users are thus incentivized
///       to keep the liquidity pools non-empty by depositing LP tokens,
///       in order to keep the liquidity ratio as close to 99% as possible.
///
///     This is a good candidate for synchronizing to an epoch clock.
///
/// 4. Liquidity in pool
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
///     * `locked` is the total number of LP tokens
///       that are currently locked in the pool.
///       * Incremented and decremented on withdraws and deposits.
///       * Should be equal to this contract's balance in the
///         LP token contract.
///
///     * `lifetime`. The total amount of liquidity
///       contained by the pool over its lifetime.
///       * Incremented by `elapsed * locked` on deposits and withdrawals.
///       * Computed as `last_value + elapsed * locked` on queries.
///
/// 5. Reward budget
///
///     The pool queries its `balance` in reward tokens from the reward token
///     contract.
///
///     * In the case of **single-sided staking** (e.g. staking SIENNA to earn SIENNA)
///       the value of `locked` is subtracted from this balance in order to separate
///       the tokens locked by users from the reward budget.
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
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Pool {
    /// "For what point in time do the following values hold true?"
    /// Passed on instantiation.
    pub now:          Moment,

    /// "When was this pool deployed?"
    /// Set to current time on init.
    pub deployed:     Moment,

    /// "Is this pool closed, and if so, when and why?"
    /// Set irreversibly via handle method.
    pub closed:       Option<CloseSeal>,

    /// "When were LP tokens first locked?"
    /// Set to current time on first lock.
    pub seeded:       Option<Moment>,

    /// "For how many units of time has this pool existed?"
    /// Computed as now - seeded
    pub existed:      Option<Duration>,

    /// "When was the last time someone locked or unlocked tokens?"
    /// Set to current time on lock/unlock.
    pub updated:      Moment,

    /// Used to compute what portion of the time the pool was not empty.
    /// Before lock/unlock, if locked > 0, this is incremented
    /// by pool.elapsed
    pub liquid:       Duration,

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
    pub threshold:    Duration,

    /// "How much must the user wait between claims?"
    /// Configured on init.
    /// User cooldowns are reset to this value on claim.
    pub cooldown:     Duration,

    pub global_ratio:    (Amount, Amount),
}

/// User status
///
/// 1. Timestamps
///
///     Each user earns rewards as a function of their liquidity contribution over time.
///     The following points and durations in time are stored for each user:
///
///     * `registered` is set to the current time the first time the user locks liquidity
///     * `existed` is the time since `registered`
///     * `updated` is the time of last update (deposit, withdraw or claim by this user)
///     * `liquid` is the number of moments for which this user has locked >0 LP.
///
/// 2. Liquidity ratio
///
///     The variable `presence` is equal to `liquid / existed`.
///     If a user provides liquidity intermittently,
///     their rewards are diminished by this proportion.
///
/// 3. Liquidity and liquidity share
///
///     * `locked` is the number of LP tokens locked by this user in this pool.
///     * The user's **momentary share** is defined as `locked / pool.locked`.
///     * `lifetime` is the lifetime liquidity contributed by this user.
///       It is incremented by `locked` for every moment elapsed.
///     * The user's **lifetime share** is defined as `lifetime / pool.lifetime`.
///       It represents the user's overall contribution, and should move in the
///       direction of the user's momentary share.
///
/// 4. Rewards claimable
///
///     * `earned` rewards are equal to `lifetime_share * user_liquidity_ratio *
///     pool_liquidity_ratio * global_ratio * pool.budget`.
///     * `claimed` rewards are incremented on each claim, by the amount claimed.
///     * `claimable` is equal to `earned - claimed`.
///
///     As the user's lifetime share increases (as a result of providing liquidity)
///     or the pool's budget increases (as a result of new reward portions being
///     unlocked from the TGE budget), new rewards are `earned` and become `claimable`.
///
///     `earned` may become less than `claimed` if the user's lifetime share
///     goes down too steeply:
///
///         * as a result of that user withdrawing liquidity;
///
///         * or as a result of an influx of liquidity by other users
///
///     This means the user has been *crowded out* - they have already claimed
///     fair rewards for their contribution up to this point, but have become
///     ineligible for further rewards until their lifetime share increases:
///
///         * as a result of that user providing a greater amount of liquidity
///
///         * as a result of other users withdrawing liquidity
///
///     and/or until the pool's balance increases:
///
///         * as a result of incoming reward portions from the TGE budget.
///
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct User {
    /// When did this user first provide liquidity?
    /// Set to current time on first update.
    pub registered:      Moment,

    /// How much time has passed since this user became known to the contract?
    /// Computed as pool.now - user.registered
    pub existed:         Duration,

    /// When did this user's liquidity amount last change?
    /// Set to current time on update.
    pub updated:         Moment,

    /// For how much time this user has provided non-zero liquidity?
    /// Incremented on update by user.elapsed if user.locked > 0
    pub liquid:          Duration,

    pub presence: (Duration, Duration),

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
    pub cooldown:        Duration,
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
