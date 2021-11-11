use fadroma::*;
use crate::{auth::Auth, errors};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;
pub trait Rewards<S: Storage, A: Api, Q: Querier>:
    Composable<S, A, Q> // to compose with other modules
    + Auth<S, A, Q>     // to authenticate txs/queries
    + Sized             // to pass mutable self-reference to Total and Account
{
    /// Configure the rewards module
    fn init (&mut self, env: &Env, mut config: RewardsConfig) -> StdResult<Vec<CosmosMsg>> {
        config.initialize(self, env)
    }
    /// Handle transactions
    fn handle (&mut self, env: Env, msg: RewardsHandle) -> StdResult<HandleResponse> {
        msg.dispatch_handle(self, env)
    }
    /// Handle queries
    fn query (&self, msg: RewardsQuery) -> StdResult<RewardsResponse> {
        msg.dispatch_query(self)
    }
}
/// Reward pool configuration
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct RewardsConfig {
    pub lp_token:     Option<ContractLink<HumanAddr>>,
    pub reward_token: Option<ContractLink<HumanAddr>>,
    pub reward_vk:    Option<String>,
    pub bonding:      Option<Duration>,
    pub timekeeper:   Option<HumanAddr>
}
impl RewardsConfig {
    pub const SELF:         &'static[u8] = b"/config/self";
    pub const LP_TOKEN:     &'static[u8] = b"/config/lp_token";
    pub const REWARD_TOKEN: &'static[u8] = b"/config/reward_token";
    pub const REWARD_VK:    &'static[u8] = b"/config/reward_vk";
    pub const CLOSED:       &'static[u8] = b"/config/closed";
    pub const BONDING:      &'static[u8] = b"/config/bonding";
    pub const TIMEKEEPER:   &'static[u8] = b"/config/keeper";
}
pub trait IRewardsConfig <S, A, Q, C> where
    S: Storage, A: Api, Q: Querier, C: Composable<S, A, Q>
{
    /// Commit initial contract configuration to storage.
    fn initialize   (&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>>;
    /// Commit contract configuration to storage.
    fn store        (&self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    /// Get this contract's address (used in queries where Env is unavailable).
    fn self_link    (core: &C) -> StdResult<ContractLink<HumanAddr>>;
    /// Get an interface to the LP token.
    fn lp_token     (core: &C) -> StdResult<ISnip20>;
    /// Get an interface to the reward token.
    fn reward_token (core: &C) -> StdResult<ISnip20>;
    /// Get the reward viewing key.
    fn reward_vk    (core: &C) -> StdResult<String>;
    /// Get the address authorized to increment the epoch
    fn timekeeper   (core: &C) -> StdResult<HumanAddr>;
}
impl<S, A, Q, C> IRewardsConfig<S, A, Q, C> for RewardsConfig where
    S: Storage, A: Api, Q: Querier, C: Rewards<S, A, Q>
{
    fn initialize (&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        if self.reward_token.is_none() {
            Err(StdError::generic_err("need to provide link to reward token"))
        } else {
            core.set(RewardsConfig::SELF, &core.canonize(ContractLink {
                address:   env.contract.address.clone(),
                code_hash: env.contract_code_hash.clone()
            })?)?;
            if self.reward_vk.is_none() {
                self.reward_vk = Some("".into())
            }
            if self.bonding.is_none() {
                self.bonding = Some(DAY)
            }
            if self.timekeeper.is_none() {
                self.timekeeper = Some(env.message.sender.clone())
            }
            self.store(core)
        }
    }
    fn store (&self, core: &mut C) -> StdResult<Vec<CosmosMsg>> {
        let RewardsConfig { timekeeper, lp_token, bonding, reward_token, reward_vk } = self;
        let mut messages = vec![];
        if let Some(lp_token) = lp_token {
            core.set(Self::LP_TOKEN, &core.canonize(lp_token.clone())?)?;
        }
        if let Some(bonding) = bonding {
            core.set(Self::BONDING, &bonding)?;
        }
        if let Some(reward_token) = reward_token {
            core.set(Self::REWARD_TOKEN, &core.canonize(reward_token.clone())?)?;
            if let Some(reward_vk) = reward_vk {
                core.set(Self::REWARD_VK, &reward_vk)?;
                messages.push(ISnip20::attach(reward_token.clone()).set_viewing_key(&reward_vk)?);
            }
        } else if let Some(reward_vk) = reward_vk {
            core.set(Self::REWARD_VK, &reward_vk)?;
            let reward_token = RewardsConfig::reward_token(core)?;
            messages.push(reward_token.set_viewing_key(&reward_vk)?);
        }
        if let Some(timekeeper) = timekeeper {
            core.set(Self::TIMEKEEPER, &core.canonize(timekeeper.clone())?)?;
        }
        Ok(messages)
    }
    fn self_link (core: &C) -> StdResult<ContractLink<HumanAddr>> {
        let link = core.get::<ContractLink<CanonicalAddr>>(Self::SELF)?
            .ok_or(StdError::generic_err("no self link"))?;
        Ok(core.humanize(link)?)
    }
    fn lp_token (core: &C) -> StdResult<ISnip20> {
        let link = core.get::<ContractLink<CanonicalAddr>>(Self::LP_TOKEN)?
            .ok_or(StdError::generic_err("no lp token"))?;
        Ok(ISnip20::attach(core.humanize(link)?))
    }
    fn reward_token (core: &C) -> StdResult<ISnip20> {
        let link = core.get::<ContractLink<CanonicalAddr>>(Self::REWARD_TOKEN)?
            .ok_or(StdError::generic_err("no reward token"))?;
        Ok(ISnip20::attach(core.humanize(link)?))
    }
    fn reward_vk (core: &C) -> StdResult<String> {
        Ok(core.get::<ViewingKey>(Self::REWARD_VK)?
            .ok_or(StdError::generic_err("no reward viewing key"))?.0)
    }
    fn timekeeper (core: &C) -> StdResult<HumanAddr> {
        Ok(core.get::<HumanAddr>(Self::TIMEKEEPER)?
            .ok_or(StdError::generic_err("no timekeeper address"))?)
    }
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsHandle {
    // Public transactions
    Lock     { amount: Amount },
    Retrieve { amount: Amount },
    Claim    {},
    // Authorized transactions
    IncrementEpoch { next_epoch: Moment },
    // Admin-only transactions
    Configure(RewardsConfig),
    Close    { message: String },
}
impl<S, A, Q, C> HandleDispatch<S, A, Q, C> for RewardsHandle where
    S: Storage, A: Api, Q: Querier, C: Rewards<S, A, Q>
{
    fn dispatch_handle (self, core: &mut C, env: Env) -> StdResult<HandleResponse> {
        match self {
            // Public transactions
            RewardsHandle::Lock { amount } =>
                Account::from_env(core, env)?.deposit(core, amount),
            RewardsHandle::Retrieve { amount } =>
                Account::from_env(core, env)?.withdraw(core, amount),
            RewardsHandle::Claim {} =>
                Account::from_env(core, env)?.claim(core),
            // Authorized transactions
            RewardsHandle::IncrementEpoch { next_epoch } =>
                Clock::increment(core, &env, next_epoch),
            // Admin-only transactions
            _ => {
                Auth::assert_admin(core, &env)?;
                match self {
                    RewardsHandle::Configure(config) => {
                        Ok(HandleResponse { messages: config.store(core)?, log: vec![], data: None })
                    },
                    RewardsHandle::Close { message } => {
                        core.set(RewardsConfig::CLOSED, Some((env.block.time, message)))?;
                        Ok(HandleResponse::default())
                    },
                    _ => unreachable!()
                }
            }
        }
    }
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsQuery {
    UserInfo { at: Moment, address: HumanAddr, key: String },
    PoolInfo { at: Moment },
}
impl<S, A, Q, C> QueryDispatch<S, A, Q, C, RewardsResponse> for RewardsQuery where
    S: Storage, A: Api, Q: Querier, C: Rewards<S, A, Q>
{
    fn dispatch_query (self, core: &C) -> StdResult<RewardsResponse> {
        match self {
            RewardsQuery::UserInfo { at, address, key } =>
                RewardsResponse::user_info(core, at, address, key),
            RewardsQuery::PoolInfo { at } =>
                RewardsResponse::pool_info(core, at)
        }
    }
}
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum RewardsResponse {
    UserInfo(Account),
    PoolInfo(Total),
}
pub trait IRewardsResponse<S, A, Q, C>: Sized where
    S: Storage, A: Api, Q: Querier, C: Composable<S, A, Q>
{
    /// Get account + pool + epoch info
    fn user_info (core: &C, time: Moment, address: HumanAddr, key: String) -> StdResult<Self>;
    /// Get pool + epoch info
    fn pool_info (core: &C, time: Moment) -> StdResult<Self>;
}
impl<S, A, Q, C> IRewardsResponse<S, A, Q, C> for RewardsResponse where
    S: Storage, A: Api, Q: Querier, C: Rewards<S, A, Q>
{
    /// Report pool status and optionally account status, at a given time
    fn user_info (core: &C, time: Moment, address: HumanAddr, key: String) -> StdResult<Self> {
        let id = core.canonize(address.clone())?;
        Auth::check_vk(core, &ViewingKey(key), id.as_slice())?;
        let clock = Clock::get(core, time)?;
        let total = Total::get(core, clock)?;
        let account = Account::get(core, total, address)?;
        Ok(RewardsResponse::UserInfo(account))
    }
    fn pool_info (core: &C, time: Moment) -> StdResult<RewardsResponse> {
        let clock = Clock::get(core, time)?;
        let total = Total::get(core, clock)?;
        Ok(RewardsResponse::PoolInfo(total))
    }
}
/// Reward epoch state. Epoch is incremented after each RPT vesting.
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Clock {
    /// "For what point in time do these values hold true?"
    /// Got from env.block time on transactions, passed by client in queries.
    pub now:     Moment,
    /// "What is the current reward epoch?"
    /// Incremented by external periodic call.
    pub number:  Moment,
    /// "When did the epoch last increment?"
    /// Set to current time on epoch increment.
    pub started: Moment,
    /// "What was the total pool liquidity at the epoch start?"
    /// Set to `total.volume` on epoch increment.
    pub volume:  Volume
    //// "what rewards were unlocked for this pool at the epoch start?"
    //// computed as balance + claimed.
    //pub unlocked_at_epoch: Amount,
}
impl Clock {
    pub const NUMBER:   &'static[u8] = b"/epoch/number";
    pub const START:    &'static[u8] = b"/epoch/start";
    pub const VOLUME:   &'static[u8] = b"/epoch/volume";
    pub const UNLOCKED: &'static[u8] = b"/epoch/unlocked";
}
pub trait IClock <S, A, Q, C> where
    S: Storage, A: Api, Q: Querier, C: Composable<S, A, Q>
{
    /// Get the current state of the epoch clock.
    fn get (core: &C, now: Moment) -> StdResult<Clock>;
    /// Increment the epoch and commit liquidity so far
    fn increment (core: &mut C, env: &Env, next_epoch: Moment) -> StdResult<HandleResponse>;
}
impl<S, A, Q, C> IClock<S, A, Q, C> for Clock where
    S: Storage, A: Api, Q: Querier, C: Rewards<S, A, Q>
{
    fn get (core: &C, now: Moment) -> StdResult<Clock> {
        Ok(Clock {
            now,
            number:  core.get(Self::NUMBER)?.unwrap_or(0u64),
            started: core.get(Self::START)?.unwrap_or(0u64),
            volume:  core.get(Self::VOLUME)?.unwrap_or(Volume::zero()),
        })
    }
    fn increment (core: &mut C, env: &Env, next_epoch: Moment) -> StdResult<HandleResponse> {
        if env.message.sender != RewardsConfig::timekeeper(core)? {
            return Err(StdError::unauthorized())
        }
        let epoch: Moment = core.get(Self::NUMBER)?.unwrap_or(0u64);
        if next_epoch != epoch + 1 {
            return Err(StdError::generic_err(format!(
                "The current epoch is {}. The 'next_epoch' field must be set to {} instead of {}.",
                epoch,
                epoch + 1,
                next_epoch
            )))
        }
        let now = env.block.time;
        core.set(Self::NUMBER, next_epoch)?;
        core.set(Self::START, now)?;
        core.set(Self::VOLUME, accumulate(
            core.get(Total::VOLUME)?.unwrap_or(Volume::zero()),
            now - core.get(Total::UPDATED)?.unwrap_or(now),
            core.get(Total::STAKED)?.unwrap_or(Amount::zero())
        )?)?;
        Ok(HandleResponse::default())
    }
}
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
/// Pool totals
pub struct Total {
    pub clock:       Clock,
    /// "When was the last time someone staked or unstaked tokens?"
    /// Set to current time on lock/unlock.
    pub updated:     Moment,
    /// "What liquidity is there in the whole pool right now?"
    /// Incremented/decremented on lock/unlock.
    pub staked:      Amount,
    /// "What liquidity has this pool contained up to this point?"
    /// Before lock/unlock, if staked > 0, this is incremented
    /// by total.elapsed * total.staked
    pub volume:      Volume,
    /// "What amount of rewards is currently available for users?"
    /// Queried from reward token.
    pub budget:      Amount,
    /// "What rewards has everyone received so far?"
    /// Incremented on claim.
    pub distributed: Amount,
    /// "what rewards were unlocked for this pool so far?"
    /// computed as balance + claimed.
    pub unlocked:    Amount,
    /// "how much must the user wait between claims?"
    /// Configured on init.
    /// Account bondings are reset to this value on claim.
    pub bonding:     Duration,
    /// "Is this pool closed, and if so, when and why?"
    /// Set irreversibly via handle method.
    pub closed:      Option<CloseSeal>,
}
pub trait ITotal <S, A, Q, C>: Sized where
    S: Storage, A: Api, Q: Querier, C: Composable<S, A, Q>
{
    fn get (core: &C, clock: Clock) -> StdResult<Self>;
}
impl Total {
    pub const VOLUME:  &'static[u8] = b"/total/volume";
    pub const UPDATED: &'static[u8] = b"/total/updated";
    pub const STAKED:  &'static[u8] = b"/total/size";
    pub const CLAIMED: &'static[u8] = b"/total/claimed";
}
impl<S, A, Q, C> ITotal<S, A, Q, C> for Total where
    S: Storage, A: Api, Q: Querier, C: Rewards<S, A, Q>
{
    fn get (core: &C, clock: Clock) -> StdResult<Self> {
        let mut total = Self::default();
        let get_time = |key, default: u64| -> StdResult<u64> {
            Ok(core.get(key)?.unwrap_or(default))
        };
        let get_amount = |key, default: Amount| -> StdResult<Amount> {
            Ok(core.get(key)?.unwrap_or(default))
        };
        let get_volume = |key, default: Volume| -> StdResult<Volume> {
            Ok(core.get(key)?.unwrap_or(default))
        };
        // # 1. Timestamps
        total.clock = clock;
        total.updated = get_time(Total::UPDATED, total.clock.now)?;
        if total.clock.now < total.updated { return errors::no_time_travel() }
        // # 2. Liquidity
        // When users lock tokens in the pool, liquidity accumulates.
        // Pool liquidity is internally represented by two variables:
        // * `staked` is the total number of LP tokens that are
        //   currently staked in the pool.
        //   * Incremented and decremented on withdraws and deposits.
        //   * Should be equal to this contract's balance in the
        //     LP token contract.
        // * `volume`. The total amount of liquidity contained by the pool
        //   over its entire lifetime. Liquidity is defined as amount of tokens
        //   multiplied by time.
        //   * Incremented by `elapsed * staked` on deposits and withdrawals.
        //   * Computed as `last_value + elapsed * staked` on queries.
        // > EXAMPLE:
        //   Starting with a new pool, lock 10 LP for 20 moments.
        //   The pool will have a liquidity of 200.
        //   Lock 10 more; 5 moments later, the liquidity will be 300.
        let last_volume  = get_volume(Total::VOLUME, Volume::zero())?;
        let elapsed      = total.clock.now - total.updated;
        total.staked     = get_amount(Total::STAKED, Amount::zero())?;
        total.volume     = accumulate(last_volume, elapsed, total.staked)?;
        let reward_token = RewardsConfig::reward_token(core)?;
        let ref address  = RewardsConfig::self_link(core)?.address;
        let ref vk       = RewardsConfig::reward_vk(core)?;
        // # 3. Budget
        // * The pool queries its `balance` in reward tokens from the reward token
        //   contract. Rewards are computed on the basis of this balance.
        // * TODO: Couple budget to epoch clock in order to
        //   define a maximum amount of rewards per epoch.
        // * In the case of **single-sided staking** (e.g. staking SIENNA to earn SIENNA)
        //   the value of `staked` is subtracted from this balance in order to separate
        //   the tokens staked by users from the reward budget.
        // * The pool keeps track of how much rewards have been distributed,
        //   in the `distributed` variable which is incremented on successful claims.
        // * The `unlocked` field is equal to `budget + claimed` and is informative.
        //   It should be equal to the sum released from RPT for this total.
        total.budget = reward_token.query_balance(core.querier(), address, vk)?;
        let lp_token = RewardsConfig::lp_token(core)?;
        let is_single_sided = reward_token.link == lp_token.link;
        if is_single_sided {
            total.budget = (total.budget - total.staked)?;
        }
        total.distributed = get_amount(Total::CLAIMED, Amount::zero())?;
        total.unlocked    = total.distributed + total.budget;
        // # 4. Throttles
        // * Bonding period: user must wait this much before each claim.
        // * Closing the pool stops its time and makes it
        //   return all funds upon any user action.
        total.bonding     = get_time(RewardsConfig::BONDING, 0u64)?;
        total.closed      = core.get(RewardsConfig::CLOSED)?;
        Ok(total)
    }
}

/// Account status
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Account {
    /// Passed around internally, not presented to user.
    #[serde(skip)] pub address: HumanAddr,
    /// Passed around internally, not presented to user.
    #[serde(skip)] pub id:      CanonicalAddr,
    /// Passed around internally, not presented to user.
    #[serde(skip)] pub total:   Total,
    /// When did this user's liquidity amount last change?
    /// Set to current time on update.
    pub updated:                  Moment,
    /// How much liquidity does this user currently provide?
    /// Incremented/decremented on lock/unlock.
    pub staked:                   Amount,
    /// What portion of the pool is currently owned by this user?
    /// Computed as user.staked / pool.staked
    pub pool_share:               (Amount, Amount),
    /// How much liquidity has this user provided since they first appeared?
    /// Incremented on update by staked * elapsed if staked > 0
    pub volume:                   Volume,
    /// What was the volume of the pool when the user entered?
    /// Set to `total.volume` on initial deposit.
    pub starting_pool_volume:     Volume,
    /// How much has `total.volume` grown, i.e. how much liquidity
    /// has accumulated in the pool since this user entered?
    /// Used as basis of reward share calculation.
    pub accumulated_pool_volume:  Volume,
    /// What portion of all the liquidity accumulated since this user's entry
    /// is due to this particular user's stake? Computed as user.volume / pool.volume
    pub reward_share:             (Volume, Volume),
    /// How much rewards were already unlocked when the user entered?
    /// Set to `total.unlocked` on initial deposit.
    pub starting_pool_rewards:    Amount,
    /// How much has `total.unlocked` grown, i.e. how much rewards
    /// have been unlocked since this user entered?
    /// Multiply this by the reward share to compute earnings.
    pub accumulated_pool_rewards: Amount,
    /// How much rewards has this user earned?
    /// Computed as user.reward_share * pool.unlocked
    pub earned:                   Amount,
    /// How many units of time remain until the user can claim?
    /// Decremented on update, reset to pool.bonding on claim.
    pub bonding:                  Duration,
    /// User-friendly reason why earned is 0
    pub reason:                   Option<String>,
}
pub trait IAccount <S, A, Q, C>: Sized where
    S: Storage, A: Api, Q: Querier, C: Composable<S, A, Q>
{
    /// Get the transaction initiator's account at current time
    fn from_env (core: &C, env: Env) -> StdResult<Self>;
    /// Get an account with up-to-date values
    fn get (core: &C, total: Total, address: HumanAddr) -> StdResult<Self>;
    /// Commit to storage the values that were updated by the passing of time
    fn commit_elapsed (&mut self, core: &mut C) -> StdResult<()>;
    /// Check if a deposit is possible, then perform it
    fn deposit (&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse>;
    /// Store the results of a deposit
    fn increment_stake (&mut self, core: &mut C, amount: Amount) -> StdResult<HandleResponse>;
    /// Check if a withdrawal is possible, then perform it
    fn withdraw (&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse>;
    /// Store the results of a withdrawal
    fn decrement_stake (&mut self, core: &mut C, amount: Amount) -> StdResult<HandleResponse>;
    /// Check if a claim is possible, then perform it
    fn claim (&mut self, core: &mut C) -> StdResult<HandleResponse>;
    /// Store the results of a claim
    fn commit_claim (&mut self, core: &mut C) -> StdResult<HandleResponse>;
    /// Return the user's stake if trying to interact with a closed pool
    fn force_exit (&mut self, core: &mut C) -> StdResult<HandleResponse>;
    /// Reset the user's liquidity conribution
    fn reset (&mut self, core: &mut C) -> StdResult<()>;
}
impl<S, A, Q, C> IAccount<S, A, Q, C> for Account where
    S: Storage, A: Api, Q: Querier, C: Rewards<S, A, Q>
{
    fn from_env (core: &C, env: Env) -> StdResult<Self> {
        Self::get(core, Total::get(core, Clock::get(core, env.block.time)?)?, env.message.sender)
    }
    fn get (core: &C, total: Total, address: HumanAddr) -> StdResult<Self> {
        let id         = core.canonize(address.clone())?;
        let get_time   = |key, default: u64| -> StdResult<u64> {
            Ok(core.get_ns(key, &id.as_slice())?.unwrap_or(default))
        };
        let get_amount = |key, default: Amount| -> StdResult<Amount> {
            Ok(core.get_ns(key, &id.as_slice())?.unwrap_or(default))
        };
        let get_volume = |key, default: Volume| -> StdResult<Volume> {
            Ok(core.get_ns(key, &id.as_slice())?.unwrap_or(default))
        };
        let mut account = Self::default();
        account.address = address;
        // 1. Timestamps
        // Each user earns rewards as a function of their liquidity contribution over time.
        // The following points and durations in time are stored for each user:
        // * `updated` is the time of last update (deposit, withdraw or claim by this user)
        account.updated = get_time(Account::UPDATED, total.clock.now)?;
        if total.clock.now < account.updated { return errors::no_time_travel() }
        // 2. Liquidity and liquidity share
        // * `staked` is the number of LP tokens staked by this user in this pool.
        // * The user's **momentary share** is defined as `staked / total.staked`.
        // * `volume` is the volume liquidity contributed by this user.
        //   It is incremented by `staked` for every moment elapsed.
        // * The user's **volume share** is defined as `volume / total.volume`.
        //   It represents the user's overall contribution, and should move in the
        //   direction of the user's momentary share.
        account.starting_pool_volume = get_volume(Account::ENTRY_VOL, total.volume)?;
        if account.starting_pool_volume > total.volume { return errors::no_time_travel() }

        account.accumulated_pool_volume = (total.volume - account.starting_pool_volume)?;

        account.starting_pool_rewards = get_amount(Account::ENTRY_REW, total.unlocked)?;
        if account.starting_pool_rewards > total.unlocked { return errors::no_time_travel() }

        account.accumulated_pool_rewards = (total.unlocked - account.starting_pool_rewards)?;

        account.staked  = get_amount(Account::STAKED, Amount::zero())?;
        let last_volume = get_volume(Account::VOLUME, Volume::zero())?;
        let elapsed: Duration = total.clock.now - account.updated;
        account.volume       = accumulate(last_volume, elapsed, account.staked)?;
        account.pool_share   = (account.staked, total.staked);
        account.reward_share = (account.volume, account.accumulated_pool_volume);
        // 3. Rewards claimable
        // `earned` rewards are equal to `total.budget * reward_share`.
        // As the user's volume share increases (as a result of providing liquidity)
        // or the pool's budget increases (as a result of new reward portions being
        // unstaked from the TGE budget), new rewards are `earned` and become `claimable`.
        // `earned` may become less than `claimed` if the user's volume share
        // goes down too steeply:
        // * as a result of that user withdrawing liquidity;
        // * or as a result of an influx of liquidity by other users
        // This means the user has been *crowded out* - they have already claimed
        // fair rewards for their contribution up to this point, but have become
        // ineligible for further rewards until their volume share increases:
        // * as a result of that user providing a greater amount of liquidity
        // * as a result of other users withdrawing liquidity
        // and/or until the pool's balance increases:
        // * as a result of incoming reward portions from the TGE budget.
        account.earned = if account.reward_share.1 == Volume::zero() {
            Amount::zero()
        } else {
            Volume::from(account.accumulated_pool_rewards) // TODO unlocked_since_entry
                .multiply_ratio(account.reward_share.0, account.reward_share.1)?
                .low_u128().into()
        };
        // 4. Bonding period
        // This decrements by `elapsed` if `staked > 0`.
        account.bonding = get_time(Self::BONDING, total.bonding)?;
        if account.staked > Amount::zero() {
            account.bonding = account.bonding.saturating_sub(elapsed)
        };
        // These are used above, then moved into the account struct at the end
        account.id    = id;
        account.total = total;
        Ok(account)
    }
    fn reset (&mut self, core: &mut C) -> StdResult<()> {
        self.updated                   = self.total.clock.now;
        self.starting_pool_volume      = self.total.volume;
        self.starting_pool_rewards = self.total.unlocked;
        self.volume                    = Volume::zero();
        self.bonding                   = self.total.bonding;
        core.set_ns(Account::UPDATED,   self.id.as_slice(), self.updated)?;
        core.set_ns(Account::ENTRY_VOL, self.id.as_slice(), self.starting_pool_volume)?;
        core.set_ns(Account::ENTRY_REW, self.id.as_slice(), self.starting_pool_rewards)?;
        core.set_ns(Account::VOLUME,    self.id.as_slice(), self.volume)?;
        core.set_ns(Account::BONDING,   self.id.as_slice(), self.bonding)?;
        Ok(())
    }
    fn commit_elapsed (&mut self, core: &mut C) -> StdResult<()> {
        core.set(Total::VOLUME,  self.total.volume)?;
        core.set(Total::UPDATED, self.total.clock.now)?;
        if self.staked == Amount::zero() {
            self.reset(core)
        } else {
            core.set_ns(Account::BONDING, self.id.as_slice(), self.bonding)?;
            core.set_ns(Account::VOLUME,  self.id.as_slice(), self.volume)?;
            core.set_ns(Account::UPDATED, self.id.as_slice(), self.total.clock.now)
        }
    }
    fn deposit (&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse> {
        if self.total.closed.is_some() {
            return self.force_exit(core)
        } else {
            self.increment_stake(core, amount)
        }
    }
    fn increment_stake (&mut self, core: &mut C, amount: Amount) -> StdResult<HandleResponse> {
        self.commit_elapsed(core)?;

        self.staked += amount;
        core.set_ns(Account::STAKED, self.id.as_slice(), self.staked)?;

        self.total.staked += amount;
        core.set(Total::STAKED, self.total.staked)?;

        let lp_token  = RewardsConfig::lp_token(core)?;
        let self_link = RewardsConfig::self_link(core)?;
        HandleResponse::default().msg(
            lp_token.transfer_from(&self.address, &self_link.address, amount)?
        )
    }
    fn withdraw (&mut self, core: &mut C, amount: Uint128) -> StdResult<HandleResponse> {
        if self.total.closed.is_some() {
            self.force_exit(core)
        } else if self.staked < amount {
            errors::withdraw(self.staked, amount)
        } else if self.total.staked < amount {
            errors::withdraw_fatal(self.total.staked, amount)
        } else {
            self.decrement_stake(core, amount)
        }
    }
    fn decrement_stake (&mut self, core: &mut C, amount: Amount) -> StdResult<HandleResponse> {
        self.commit_elapsed(core)?;

        self.staked = (self.staked - amount)?;
        core.set_ns(Account::STAKED, self.id.as_slice(), self.staked)?;

        self.total.staked = (self.total.staked - amount)?;
        core.set(Total::STAKED, self.total.staked)?;

        if self.staked == Amount::zero() { // hairy, fixme
            if self.bonding == 0 {
                self.commit_claim(core)?
            } else {
                self.reset(core)?;
                HandleResponse::default()
            }
        } else {
            HandleResponse::default()
        }.msg(
            RewardsConfig::lp_token(core)?.transfer(&self.address, amount)?
        )
    }
    fn claim (&mut self, core: &mut C) -> StdResult<HandleResponse> {
        if self.total.closed.is_some() {
            self.force_exit(core)
        } else if self.bonding > 0 {
            errors::claim_bonding(self.bonding)
        } else if self.total.budget == Amount::zero() {
            errors::claim_pool_empty()
        } else if self.earned == Amount::zero() {
            errors::claim_zero_claimable()
        } else {
            self.commit_claim(core)
        }
    }
    fn commit_claim (&mut self, core: &mut C) -> StdResult<HandleResponse> {
        let earned = self.earned;
        if earned == Amount::zero() { return Ok(HandleResponse::default()) }

        self.reset(core)?;

        self.total.distributed += earned;
        core.set(Total::CLAIMED, self.total.distributed)?;

        HandleResponse::default()
            .msg(RewardsConfig::reward_token(core)?.transfer(&self.address, earned)?)
    }
    fn force_exit (&mut self, core: &mut C) -> StdResult<HandleResponse> {
        if let Some((ref when, ref why)) = self.total.closed {
            let amount = self.staked;
            let response = HandleResponse::default()
                .msg(RewardsConfig::lp_token(core)?.transfer(&self.address, amount)?)?
                .log("close_time",   &format!("{}", when))?
                .log("close_reason", &format!("{}", why))?;
            self.decrement_stake(core, amount)?;
            Ok(response)
        } else {
            Err(StdError::generic_err("pool not closed"))
        }
    }
}
impl Account {
    pub const ENTRY_VOL: &'static[u8] = b"/user/entry_vol/";
    pub const ENTRY_REW: &'static[u8] = b"/user/entry_rew/";
    pub const STAKED:    &'static[u8] = b"/user/current/";
    pub const UPDATED:   &'static[u8] = b"/user/updated/";
    pub const VOLUME:    &'static[u8] = b"/user/volume/";
    pub const CLAIMED:   &'static[u8] = b"/user/claimed/";
    pub const BONDING:   &'static[u8] = b"/user/bonding/";
}
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
/// When and why was the pool closed
pub type CloseSeal = (Moment, String);
/// Project current value of an accumulating parameter based on stored value,
/// time since it was last updated, and rate of change, i.e.
/// `current = stored + (elapsed * rate)`
// * The need to store detailed history (and iterate over it, unboundedly)
//   is avoided by using continuously accumulating values.
// * The state can't be updated outside of a transaction,
//   the current values of the accumulators need to be computed as
//   `last value + (elapsed * rate)`, where:
//   * `last value` is fetched from storage
//   * `elapsed` is `now - last update`
//     * v2 measures time in blocks
//     * v3 measures time in seconds
//     * For transactions, `now` is `env.block.time`.
//     * For queries, `now` has to be passed by the client.
//   * `rate` depends on what is being computed:
//     * `total.volume` grows by `total.staked` every moment.
//     * `user.volume` grows by `user.staked` every moment.
//     * `user.bonding` decreases by 1 every moment, until it reaches 0.
pub fn accumulate (
    total_before_last_update: Volume,
    time_since_last_update:   Duration,
    value_after_last_update:  Amount
) -> StdResult<Volume> {
    let increment = Volume::from(value_after_last_update).multiply_ratio(
        time_since_last_update,
        1u128
    )?;
    total_before_last_update + increment
}
