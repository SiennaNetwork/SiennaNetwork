use fadroma::*;
use crate::{auth::Auth, errors};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

/// A moment in time, as represented by the current value of env.block.time
pub type Moment   = u64;
/// A duration of time, represented as a number of moments
pub type Duration = u64;
/// Seconds in 24 hours
pub const DAY: Duration = 86400;

/// Amount of funds
pub type Amount = Uint128;
/// Amount multiplied by duration.
pub type Volume = Uint256;
/// A ratio, represented as tuple (nom, denom)
pub type Ratio  = (Uint128, Uint128);

/// Project current value of an accumulating parameter based on stored value,
/// time since it was last updated, and rate of change, i.e.
/// `current = stored + (elapsed * rate)`
///
/// * The need to store detailed history (and iterate over it, unboundedly)
///   is avoided by using continuously accumulating values.
///
/// * The state can't be updated outside of a transaction,
///   the current values of the accumulators need to be computed as
///   `last value + (elapsed * rate)`, where:
///
///   * `last value` is fetched from storage
///
///   * `elapsed` is `now - last update`
///
///     * v2 measures time in blocks
///
///     * v3 measures time in seconds
///
///     * For transactions, `now` is `env.block.time`.
///
///     * For queries, `now` has to be passed by the client.
///
///   * `rate` depends on what is being computed:
///
///     * `total.volume` grows by `total.staked` every moment.
///
///     * `user.volume` grows by `user.staked` every moment.
///
///     * `user.bonding` decreases by 1 every moment, until it reaches 0.
pub fn accumulate (
    total_before_last_update: Volume,
    time_since_last_update:   Duration,
    value_after_last_update:  Amount
) -> StdResult<Volume> {
    total_before_last_update + Volume::from(value_after_last_update)
        .multiply_ratio(time_since_last_update, 1u128)?
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
        time:    Moment,
        total:   Totals,
        account: Option<Account>
    }
}

pub type CloseSeal = (Moment, String);

pub mod config {
}

#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct RewardsConfig {
    pub lp_token:     Option<ContractLink<HumanAddr>>,
    pub reward_token: Option<ContractLink<HumanAddr>>,
    pub reward_vk:    Option<String>,
    pub ratio:        Option<(Uint128, Uint128)>,
    pub bonding:      Option<Duration>
}
impl RewardsConfig {
    pub const SELF:         &'static[u8] = b"/config/self";
    pub const LP_TOKEN:     &'static[u8] = b"/config/lp_token";
    pub const REWARD_TOKEN: &'static[u8] = b"/config/reward_token";
    pub const REWARD_VK:    &'static[u8] = b"/config/reward_vk";
    pub const CLOSED:       &'static[u8] = b"/config/closed";
    pub const RATIO:        &'static[u8] = b"/config/ratio";
    pub const BONDING:      &'static[u8] = b"/config/bonding";
    fn commit <S: Storage, A: Api, Q: Querier> (
        &self, contract: &mut impl Rewards<S, A, Q>
    ) -> StdResult<Vec<CosmosMsg>> {
        let mut messages = vec![];
        if let Some(reward_token) = &self.reward_token {
            contract.set(Self::REWARD_TOKEN, &contract.canonize(reward_token.clone())?)?;
        }
        if let Some(reward_vk) = &self.reward_vk {
            contract.set(Self::REWARD_VK, &reward_vk)?;
            messages.push(contract.reward_token()?.set_viewing_key(&reward_vk)?);
        }
        if let Some(lp_token) = &self.lp_token {
            contract.set(Self::LP_TOKEN, &contract.canonize(lp_token.clone())?)?;
        }
        if let Some(ratio) = &self.ratio {
            contract.set(Self::RATIO, &ratio)?;
        }
        if let Some(bonding) = &self.bonding {
            contract.set(Self::BONDING, &bonding)?;
        }
        Ok(messages)
    }
}

pub trait Rewards<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q> + Auth<S, A, Q> {

    /// Initialize the rewards module
    fn init (&mut self, env: &Env, config: RewardsConfig) -> StdResult<Vec<CosmosMsg>> where Self: Sized {
        let reward_token = config.reward_token.ok_or(
            StdError::generic_err("need to provide link to reward token")
        )?;
        self.set(RewardsConfig::SELF, &self.canonize(ContractLink {
            address:   env.contract.address.clone(),
            code_hash: env.contract_code_hash.clone()
        })?)?;
        RewardsConfig {
            lp_token:     config.lp_token,
            reward_token: Some(reward_token),
            reward_vk:    Some(config.reward_vk.unwrap_or("".into())),
            ratio:        Some(config.ratio.unwrap_or((1u128.into(), 1u128.into()))),
            bonding:      Some(config.bonding.unwrap_or(DAY))
        }.commit(self)
    }

    /// Handle transactions
    fn handle (&mut self, env: &Env, msg: RewardsHandle) -> StdResult<HandleResponse> where Self: Sized {
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
                        self.handle_configure(config)
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
    fn handle_configure (&mut self, config: RewardsConfig) -> StdResult<HandleResponse> where Self: Sized {
        let messages = config.commit(self)?;
        Ok(HandleResponse { messages, log: vec![], data: None })
    }

    /// Admin can mark pool as closed
    fn handle_close (&mut self, time: Moment, message: String) -> StdResult<HandleResponse> {
        self.set(RewardsConfig::CLOSED, Some((time, message)))?;
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
            self.set(RewardsConfig::REWARD_VK, key.clone())?
        }
        let allowance = Uint128(u128::MAX);
        let duration  = Some(time + DAY * 10000);
        let snip20    = ISnip20::attach(snip20);
        HandleResponse::default()
            .msg(snip20.increase_allowance(&recipient, allowance, duration)?)?
            .msg(snip20.set_viewing_key(&key)?)
    }

    fn self_link (&self) -> StdResult<ContractLink<HumanAddr>> {
        let link = self.get::<ContractLink<CanonicalAddr>>(RewardsConfig::SELF)?
            .ok_or(StdError::generic_err("no self link"))?;
        Ok(self.humanize(link)?)
    }

    fn lp_token (&self) -> StdResult<ISnip20> {
        let link = self.get::<ContractLink<CanonicalAddr>>(RewardsConfig::LP_TOKEN)?
            .ok_or(StdError::generic_err("no lp token"))?;
        Ok(ISnip20::attach(self.humanize(link)?))
    }

    fn reward_token (&self) -> StdResult<ISnip20> {
        let link = self.get::<ContractLink<CanonicalAddr>>(RewardsConfig::REWARD_TOKEN)?
            .ok_or(StdError::generic_err("no reward token"))?;
        Ok(ISnip20::attach(self.humanize(link)?))
    }

    fn reward_vk (&self) -> StdResult<String> {
        Ok(self.get::<ViewingKey>(RewardsConfig::REWARD_VK)?
            .ok_or(StdError::generic_err("no reward viewing key"))?
            .0)
    }

    fn get_status (&mut self, env: &Env) -> StdResult<(Totals, Account)> where Self: Sized {
        let total   = Totals::status(self, env.block.time)?;
        let account = Account::status(self, &total, env.message.sender.clone())?;
        Ok((total, account))
    }

    /// Deposit LP tokens from user into pool
    fn handle_deposit (&mut self, env: &Env, amount: Amount) -> StdResult<HandleResponse> where Self: Sized {
        let (ref mut total, ref mut account) = self.get_status(&env)?;
        if total.closed.is_some() {
            return self.force_exit(env, total, account)
        }
        account.commit(self, total)?;
        total.commit(self)?;
        self.increment_stake(total, account, amount)?;
        HandleResponse::default()
            .msg(self.lp_token()?.transfer_from(&env.message.sender, &env.contract.address, amount)?)
    }

    /// Withdraw deposited LP tokens from pool back to the account
    fn handle_withdraw (&mut self, env: &Env, amount: Uint128) -> StdResult<HandleResponse> where Self: Sized {
        let (ref mut total, ref mut account) = self.get_status(&env)?;
        let response = if total.closed.is_some() {
            self.force_exit(env, total, account)?
        } else if account.staked < amount {
            return errors::withdraw(account.staked, amount)
        } else if total.staked < amount {
            return errors::withdraw_fatal(total.staked, amount)
        } else {
            HandleResponse::default().msg(
                self.lp_token()?.transfer(&env.message.sender, amount)?
            )?
        };
        self.decrement_stake(total, account, amount)?;
        total.commit(self)?;
        account.commit(self, total)?;
        Ok(response)
    }

    /// Transfer rewards to account if eligible
    fn handle_claim (&mut self, env: &Env) -> StdResult<HandleResponse> where Self: Sized {
        let (ref mut total, ref mut account) = self.get_status(&env)?;
        let response = if account.bonding > 0 {
            return errors::claim_bonding(account.bonding)
        } else if total.budget == Amount::zero() {
            return errors::claim_pool_empty()
        } else if total.global_ratio.0 == Amount::zero() {
            return errors::claim_global_ratio_zero()
        } else if account.earned == Amount::zero() {
            return errors::claim_zero_claimable()
        } else if total.closed.is_some() {
            self.force_exit(env, total, account)?
        } else {
            HandleResponse::default()
        };
        let response = response.msg(
            self.reward_token()?.transfer(&env.message.sender, account.earned)?
        )?;
        self.commit_claim(total, account)?;
        total.commit(self)?;
        account.reset(self, total)?;
        Ok(response)
    }

    fn force_exit (
        &mut self,
        env:  &Env,
        total: &mut Totals,
        account: &mut Account,
    ) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = total.closed {
            let amount = account.staked;
            let response = HandleResponse::default()
                .msg(self.lp_token()?.transfer(&env.message.sender, amount)?)?
                .log("close_time",   &format!("{}", when))?
                .log("close_reason", &format!("{}", why))?;
            self.decrement_stake(total, account, amount)?;
            Ok(response)
        } else {
            Err(StdError::generic_err("pool not closed"))
        }
    }

    fn increment_stake (&mut self, total: &mut Totals, account: &mut Account, amount: Amount) -> StdResult<()> {
        account.staked += amount;
        total.staked += amount;
        self.commit_stake_change(total, account)
    }

    fn decrement_stake (&mut self, total: &mut Totals, account: &mut Account, amount: Amount) -> StdResult<()> {
        account.staked = (account.staked - amount)?;
        total.staked = (total.staked - amount)?;
        self.commit_stake_change(total, account)
    }

    fn commit_stake_change (&mut self, total: &Totals, account: &Account) -> StdResult<()> {
        self.set_ns(Account::STAKED, account.id.as_slice(), account.staked)?;
        self.set(Totals::STAKED, total.staked)
    }

    fn commit_claim (&mut self, total: &mut Totals, account: &mut Account) -> StdResult<()> {
        total.distributed += account.earned;
        self.set(Totals::CLAIMED, total.distributed)?;
        Ok(())
    }

    /// Handle queries
    fn query (&self, msg: RewardsQuery) -> StdResult<RewardsResponse> where Self: Sized {
        match msg {
            RewardsQuery::Status { at, address, key } =>
                self.query_status(at, address, key)
        }
    }

    /// Report pool status and optionally account status, at a given time
    fn query_status (
        &self, now: Moment, address: Option<HumanAddr>, key: Option<String>
    ) -> StdResult<RewardsResponse> where Self: Sized {
        if address.is_some() && key.is_none() {
            return Err(StdError::generic_err("no viewing key"))
        }
        let total = Totals::status(self, now)?;
        if now < total.updated {
            return Err(StdError::generic_err("no history"))
        }
        let account = if let (Some(address), Some(key)) = (address, key) {
            let id = self.canonize(address.clone())?;
            Auth::check_vk(self, &ViewingKey(key), id.as_slice())?;
            Some(Account::status(self, &total, address)?)
        } else {
            None
        };
        Ok(RewardsResponse::Status { time: now, total, account })

    }

}

#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
/// Totals status
pub struct Totals {
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
    /// by total.elapsed * total.staked
    pub volume:       Volume,
    /// "What liquidity is there in the whole pool right now?"
    /// Incremented/decremented on lock/unlock.
    pub staked:       Amount,
    /// "What amount of rewards is currently available for users?"
    /// Queried from reward token.
    pub budget:       Amount,
    /// "What rewards has everyone received so far?"
    /// Incremented on claim.
    pub distributed:  Amount,
    /// "What rewards were unstaked for this pool so far?"
    /// Computed as balance + claimed.
    pub unlocked:     Amount,
    /// "How much must the user wait between claims?"
    /// Configured on init.
    /// Account bondings are reset to this value on claim.
    pub bonding:      Duration,
    /// Used to throttle the pool.
    pub global_ratio: (Amount, Amount),
}

impl Totals {
    pub const VOLUME:  &'static[u8] = b"/total/volume";
    pub const UPDATED: &'static[u8] = b"/total/updated";
    pub const STAKED:  &'static[u8] = b"/total/size";
    pub const CLAIMED: &'static[u8] = b"/total/claimed";
    fn status <S: Storage, A: Api, Q: Querier> (
        contract: &impl Rewards<S, A, Q>,
        now:      Moment
    ) -> StdResult<Self> {
        let mut total = Self::default();
        // # I. Timestamps
        total.now = now;
        total.updated = contract.get(Self::UPDATED)?.unwrap_or(now);
        if total.now < total.updated {
            return errors::no_time_travel()
        }
        // # II. Liquidity
        // * When users lock tokens in the pool, liquidity accumulates.
        //   Pool liquidity is internally represented by two variables:
        //   * `staked` is the total number of LP tokens that are
        //      currently staked in the pool.
        //     * Incremented and decremented on withdraws and deposits.
        //     * Should be equal to this contract's balance in the
        //       LP token contract.
        //   * `volume`. The total amount of liquidity contained by the pool
        //     over its entire lifetime. Liquidity is defined as amount of tokens
        //     multiplied by time.
        //     * Incremented by `elapsed * staked` on deposits and withdrawals.
        //     * Computed as `last_value + elapsed * staked` on queries.
        // * Starting with a new pool, lock 10 LP for 20 moments.
        //   The pool will have a liquidity of 200.
        //   Lock 10 more and 5 moments later the liquidity will be 300.
        let last_volume = contract.get(Self::VOLUME)?.unwrap_or(Volume::zero());
        let elapsed     = now - total.updated;
        total.staked    = contract.get(Self::STAKED)?.unwrap_or(Amount::zero());
        total.volume    = accumulate(last_volume, elapsed, total.staked)?;
        // # III. Budget
        // * The pool queries its `balance` in reward tokens from the reward token
        //   contract. Rewards are computed on the basis of this balance.
        // * TODO: Couple budget to epoch clock in order to
        //   define a maximum amount of rewards per epoch.
        let reward_token = contract.reward_token()?;
        let ref address  = contract.self_link()?.address;
        let ref vk       = contract.reward_vk()?;
        total.budget     = reward_token.query_balance(contract.querier(), address, vk)?;
        // * In the case of **single-sided staking** (e.g. staking SIENNA to earn SIENNA)
        //   the value of `staked` is subtracted from this balance in order to separate
        //   the tokens staked by users from the reward budget.
        let lp_token = contract.lp_token()?;
        if reward_token.link == lp_token.link {
            total.budget = (total.budget - total.staked)?;
        }
        // * The pool keeps track of how much rewards have been distributed,
        //   in the `distributed` variable which is incremented on successful claims.
        total.distributed = contract.get(Self::CLAIMED)?.unwrap_or(Amount::zero());
        // * The `unlocked` field is equal to `budget + claimed` and is informative.
        //   It should be equal to the sum released from RPT for this total.
        total.unlocked    = total.distributed + total.budget;
        // # IV. Throttles
        // * This can be configured by the admin to
        //   manually boost or reduce reward distribution.
        total.global_ratio = contract.get(RewardsConfig::RATIO)?
            .ok_or(StdError::generic_err("missing global ratio"))?;
        // * Bonding period: user must wait this much before each claim.
        total.bonding = contract.get(RewardsConfig::BONDING)?.unwrap_or(0u64);
        // * Closing the pool stops its time and makes it
        //   return all funds upon any user action.
        total.closed = contract.get(RewardsConfig::CLOSED)?;
        Ok(total)
    }
    fn commit <S: Storage, A: Api, Q: Querier> (
        &mut self,
        contract: &mut impl Rewards<S, A, Q>
    ) -> StdResult<()> {
        contract.set(Self::VOLUME,  self.volume)?;
        contract.set(Self::UPDATED, self.now)?;
        Ok(())
    }
}

/// Account status
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Account {
    /// What was the volume liquidity of the pool when the user entered?
    /// Account's reward share is computed from liquidity accumulated over that amount.
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
    /// Computed as user.reward_share * pool.unlocked
    pub earned:       Amount,
    /// Account-friendly reason why earned is 0
    pub reason:       Option<String>,
    /// How many units of time remain until the user can claim again?
    /// Decremented on lock/unlock, reset to pool.bonding on claim.
    pub bonding:      Duration,
    /// What portion of the pool is this user currently contributing?
    /// Computed as user.staked / pool.staked
    pub pool_share:   (Amount, Amount),
    /// What portion of all the liquidity has this user ever contributed?
    /// Computed as user.volume / pool.volume
    pub reward_share: (Volume, Volume),
    
    /// Passed around internally, not presented to user.
    #[serde(skip)] pub address: HumanAddr,
    /// Passed around internally, not presented to user.
    #[serde(skip)] pub id: CanonicalAddr,
}

impl Account {
    pub const ENTRY:   &'static[u8] = b"/user/entry/";
    pub const STAKED:  &'static[u8] = b"/user/current/";
    pub const UPDATED: &'static[u8] = b"/user/updated/";
    pub const VOLUME:  &'static[u8] = b"/user/volume/";
    pub const CLAIMED: &'static[u8] = b"/user/claimed/";
    pub const BONDING: &'static[u8] = b"/user/bonding/";

    fn get <S: Storage, A: Api, Q: Querier, T: serde::de::DeserializeOwned> (
        &self,
        contract: &impl Rewards<S, A, Q>,
        key:      &[u8],
        default:  T
    ) -> StdResult<T> {
        contract.get_ns(key, self.id.as_slice())?.unwrap_or(Ok(default))
    }

    pub fn status <S: Storage, A: Api, Q: Querier> (
        contract: &impl Rewards<S, A, Q>,
        total:    &Totals,
        address:  HumanAddr
    ) -> StdResult<Self> {
        let now = total.now;
        // 1. Timestamps
        //
        //     Each user earns rewards as a function of their liquidity contribution over time.
        //     The following points and durations in time are stored for each user:
        //
        //     * `updated` is the time of last update (deposit, withdraw or claim by this user)
        let mut account = Self::default();
        account.address = address.clone();
        account.id      = contract.canonize(address)?;
        account.updated = account.get(contract, Self::UPDATED, total.now)?;
        if total.now < account.updated {
            return errors::no_time_travel()
        }
        // 2. Liquidity and liquidity share
        //
        //     * `staked` is the number of LP tokens staked by this user in this pool.
        //     * The user's **momentary share** is defined as `staked / total.staked`.
        //     * `volume` is the volume liquidity contributed by this user.
        //       It is incremented by `staked` for every moment elapsed.
        //     * The user's **volume share** is defined as `volume / total.volume`.
        //       It represents the user's overall contribution, and should move in the
        //       direction of the user's momentary share.
        account.entry = account.get(contract, Self::ENTRY, total.volume)?;
        if account.entry > total.volume {
            return errors::no_time_travel()
        }
        account.staked = account.get(contract, Self::STAKED, Amount::zero())?;
        account.pool_share = (account.staked, total.staked);
        let last_volume = account.get(contract, Self::VOLUME, Volume::zero())?;
        let elapsed: Duration = total.now - account.updated;
        account.volume = accumulate(last_volume, elapsed, account.staked)?;
        account.reward_share = (account.volume, (total.volume - account.entry)?);
        // 3. Rewards claimable
        //
        //     * `earned` rewards are equal to `volume_share * user_liquidity_ratio *
        //     pool_liquidity_ratio * global_ratio * total.budget`.
        //     * `claimed` rewards are incremented on each claim, by the amount claimed.
        //     * `claimable` is equal to `earned - claimed`.
        //
        //     As the user's volume share increases (as a result of providing liquidity)
        //     or the pool's budget increases (as a result of new reward portions being
        //     unstaked from the TGE budget), new rewards are `earned` and become `claimable`.
        //
        //     `earned` may become less than `claimed` if the user's volume share
        //     goes down too steeply:
        //
        //         * as a result of that user withdrawing liquidity;
        //
        //         * or as a result of an influx of liquidity by other users
        //
        //     This means the user has been *crowded out* - they have already claimed
        //     fair rewards for their contribution up to this point, but have become
        //     ineligible for further rewards until their volume share increases:
        //
        //         * as a result of that user providing a greater amount of liquidity
        //
        //         * as a result of other users withdrawing liquidity
        //
        //     and/or until the pool's balance increases:
        //
        //         * as a result of incoming reward portions from the TGE budget.
        account.earned = if account.reward_share.1 == Volume::zero() {
            Amount::zero()
        } else {
            Volume::from(total.budget)
                .multiply_ratio(account.reward_share.0, account.reward_share.1)?
                .low_u128().into()
        };
        // 4. Bonding period
        account.bonding = account.get(contract, Self::BONDING, total.bonding)?;
        if account.staked > Amount::zero() {
            account.bonding = account.bonding - u64::min(elapsed, account.bonding)
        };
        Ok(account)
    }

    pub fn commit <S: Storage, A: Api, Q: Querier> (
        &mut self,
        contract: &mut impl Rewards<S, A, Q>, 
        total:    &Totals,
    ) -> StdResult<()> {
        if self.staked == Amount::zero() {
            self.reset(contract, total)
        } else {
            contract.set_ns(Self::BONDING, self.id.as_slice(), self.bonding)?;
            contract.set_ns(Self::VOLUME,  self.id.as_slice(), self.volume)?;
            contract.set_ns(Self::UPDATED, self.id.as_slice(), total.now)?;
            Ok(())
        }
    }

    /// Reset the user's liquidity conribution
    pub fn reset <S: Storage, A: Api, Q: Querier> (
        &mut self,
        contract: &mut impl Rewards<S, A, Q>, 
        total:    &Totals,
    ) -> StdResult<()> {
        self.entry   = total.volume;
        self.bonding = total.bonding;
        self.volume  = Volume::zero();
        self.updated = 0;
        contract.set_ns(Self::ENTRY,   self.id.as_slice(), self.entry)?;
        contract.set_ns(Self::BONDING, self.id.as_slice(), self.bonding)?;
        contract.set_ns(Self::VOLUME,  self.id.as_slice(), self.volume)?;
        contract.set_ns(Self::UPDATED, self.id.as_slice(), self.updated)?;
        Ok(())
    }

}
