//! Since there is a limited amount of rewards for each day,
//! they need to be distributed among the top liquidity providers.
//!
//! By locking funds, the user starts accruing a lifetime share of the pool
//! which entitles them to an equal percent of the total rewards,
//! which are distributed daily and the user can claim one per day.
//!
//! This lifetime share fluctuates as a result of the other users
//! locking and unlocking amounts of funds for different amounts of time.
//! If it remains constant or increases, users are guaranteed a new reward
//! every day. If they fall behind, they may be able to claim rewards
//! less frequently, and need to lock more tokens to restore their place
//! in the queue.

// Endgame mode: give exactly this many tokens to the contract to
// pay out remaining rewards.

#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
#[cfg(test)] #[macro_use] extern crate kukumba;
#[cfg(any(test, browser))] mod rewards_harness;
#[cfg(test)] mod rewards_test;

pub mod rewards_math; use rewards_math::*;
mod rewards_algo;     use rewards_algo::*;
mod rewards_config;   use rewards_config::*;

use fadroma::scrt::{
    callback::{ContractInstance as ContractLink},
    contract::*,
    snip20_api::ISnip20,
    vk::{
        ViewingKey,
        auth_handle, authenticate, AuthHandleMsg,
        DefaultHandleImpl as AuthHandle
    },
    admin::{
        DefaultHandleImpl as AdminHandle,
        admin_handle, AdminHandleMsg, load_admin,
        assert_admin, save_admin
    }
};

macro_rules! tx_ok {
    () => {
        Ok(HandleResponse::default())
    };
    ($($msg:expr),*) => {
        Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None })
    };
}

pub const DAY: Time = 17280; // blocks over ~24h @ 5s/block

contract! {

    [NoGlobalState] {}

    [Init] (deps, env, msg: {
        admin:        Option<HumanAddr>,
        lp_token:     Option<ContractLink<HumanAddr>>,
        reward_token: ContractLink<HumanAddr>,
        viewing_key:  ViewingKey,
        ratio:        Option<Ratio>,
        threshold:    Option<Time>,
        cooldown:     Option<Time>
    }) {
        // Contract has an admin who can do admin stuff.
        save_admin(deps, &admin.unwrap_or(env.message.sender))?;
        // Contract accepts transactions in `lp_token`s.
        // The address of the `lp_token` can be provided later
        // to avoid a circular dependency during deployment.
        if let Some(lp_token) = lp_token {
            save_lp_token(&mut deps.storage, &deps.api, &lp_token)?; }
        // Contract distributes rewards in Reward Tokens.
        // For this, it must know its own balance in the `reward_token`s.
        // For that, it needs a reference to its own address+code_hash
        // and a viewing key in `reward_token`.
        let set_vk = ISnip20::attach(&reward_token).set_viewing_key(&viewing_key.0)?;
        save_reward_token(&mut deps.storage, &deps.api, &reward_token)?;
        save_viewing_key(&mut deps.storage, &viewing_key)?;
        save_self_reference(&mut deps.storage, &deps.api, &ContractLink {
            address: env.contract.address,
            code_hash: env.contract_code_hash })?;
        // Reward pool has configurable parameters:
        // - Ratio (to reduce everyone's rewards equally)
        // - Threshold (to incentivize users to lock tokens for longer)
        Pool::new(&mut deps.storage)
            .configure_ratio(&ratio.unwrap_or((1u128.into(), 1u128.into())))?
            .configure_threshold(&threshold.unwrap_or(DAY))?
            .configure_cooldown(&cooldown.unwrap_or(DAY))?;
        // TODO remove global state from scrt-contract
        // define field! and addr_field! macros instead -
        // problem here is identifier concatenation
        // and making each field a module is ugly
        save_state!(NoGlobalState {});
        InitResponse { messages: vec![set_vk], log: vec![] }
    }

    [Query] (deps, _state, msg) -> Response {

        /// Who is admin? TODO do we need this in prod?
        Admin () {
            Ok(Response::Admin { address: load_admin(&deps)? }) }

        /// Overall pool status
        PoolInfo (at: Time) {
            let pool = Pool::new(&deps.storage).at(at);
            let pool_last_update = pool.timestamp()?;
            if at < pool_last_update {
                return Err(StdError::generic_err("this contract does not store history"))
            }
            Ok(Response::PoolInfo {
                it_is_now: at,

                lp_token: load_lp_token(&deps.storage, &deps.api)?,
                reward_token: load_reward_token(&deps.storage, &deps.api)?,

                pool_last_update,
                pool_lifetime:    pool.lifetime()?,
                pool_locked:      pool.locked()?,
                pool_claimed:     pool.claimed()?,
                pool_balance:     pool.balance(),
                pool_threshold:   pool.threshold()?,
                pool_cooldown:    pool.cooldown()?,

                // todo add balance/claimed/total in rewards token
            }) }

        /// Requires the user's viewing key.
        UserInfo (at: Time, address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;

            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;

            let reward_token_link = load_reward_token(&deps.storage, &deps.api)?;
            let reward_token      = ISnip20::attach(&reward_token_link);
            let reward_balance    = reward_token.query(&deps.querier).balance(
                &load_self_reference(&deps.storage, &deps.api)?.address,
                &load_viewing_key(&deps.storage)?.0, )?;

            let pool = Pool::new(&deps.storage).at(at).with_balance(reward_balance);
            let pool_last_update = pool.timestamp()?;
            if at < pool_last_update {
                return Err(StdError::generic_err("no data"))
            }
            let pool_lifetime = pool.lifetime()?;
            let pool_locked = pool.locked()?;

            let user = pool.user(address);
            let user_last_update = user.timestamp()?;
            if at < pool_last_update {
                return Err(StdError::generic_err("no data"))
            }

            let user_lifetime = user.lifetime()?;
            let user_share = if pool_lifetime > Volume::zero() {
                Volume::from(100000000u128)
                    .multiply_ratio(user_lifetime, pool_lifetime)?
                    .low_u128()
            } else {
                0u128
            }.into();

            Ok(Response::UserInfo {
                it_is_now: at,

                pool_last_update,
                pool_lifetime,
                pool_locked,

                user_last_update,
                user_lifetime,
                user_locked:    user.locked()?,
                user_age:       user.age()?,
                user_share,
                user_earned:    user.earned()?,
                user_claimed:   user.claimed()?,
                user_claimable: user.claimable()?,
                user_cooldown:  user.cooldown()?
            }) }

        /// Keplr integration
        TokenInfo () {
            let lp_token      = load_lp_token(&deps.storage, &deps.api)?;
            let lp_token_info = ISnip20::attach(&lp_token).query(&deps.querier).token_info()?;
            let lp_token_name = format!("Sienna Rewards: {}", lp_token_info.name);
            Ok(Response::TokenInfo {
                name:         lp_token_name,
                symbol:       "SRW".into(),
                decimals:     1,
                total_supply: None
            }) }

        /// Keplr integration
        Balance (address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;
            Ok(Response::Balance {
                amount: Pool::new(&deps.storage).user(address).locked()?
            }) }

    }

    [Response] {

        /// Response from `Query::PoolInfo`
        PoolInfo {
            lp_token:         ContractLink<HumanAddr>,
            reward_token:     ContractLink<HumanAddr>,

            it_is_now:        Time,

            pool_last_update: Time,
            pool_lifetime:    Volume,
            pool_locked:      Amount,

            pool_balance:     Amount,
            pool_claimed:     Amount,

            pool_threshold:   Time,
            pool_cooldown:    Time
        }

        /// Response from `Query::UserInfo`
        UserInfo {
            it_is_now:        Time,

            pool_last_update: Time,
            pool_lifetime:    Volume,
            pool_locked:      Amount,

            user_last_update: Option<Time>,
            user_lifetime:    Volume,
            user_locked:      Amount,
            user_share:       Amount,
            user_age:         Time,
            user_earned:      Amount,
            user_claimed:     Amount,
            user_claimable:   Amount,
            user_cooldown:    Time
        }

        Admin {
            address: HumanAddr
        }

        /// Keplr integration
        TokenInfo {
            name:         String,
            symbol:       String,
            decimals:     u8,
            total_supply: Option<Amount>
        }

        /// Keplr integration
        Balance {
            amount: Amount
        }

    }

    [Handle] (deps, env /* it's not unused :( */, _state, msg) -> Response {

        // actions that can only be performed by an admin -----------------------------------------

        /// Set the contract admin.
        ChangeAdmin (address: HumanAddr) {
            let msg = AdminHandleMsg::ChangeAdmin { address };
            admin_handle(deps, env, msg, AdminHandle) }

        /// Set the active asset token.
        // Resolves circular reference when initializing the benchmark -
        // they need to know each other's addresses to use initial allowances
        SetProvidedToken (address: HumanAddr, code_hash: String) {
            assert_admin(&deps, &env)?;
            save_lp_token(&mut deps.storage, &deps.api, &ContractLink { address, code_hash })?;
            tx_ok!() }

        // actions that are performed by users ----------------------------------------------------

        /// User can request a new viewing key for oneself.
        CreateViewingKey (entropy: String, padding: Option<String>) {
            let msg = AuthHandleMsg::CreateViewingKey { entropy, padding: None };
            auth_handle(deps, env, msg, AuthHandle) }

        /// User can set own viewing key to a known value.
        SetViewingKey (key: String, padding: Option<String>) {
            let msg = AuthHandleMsg::SetViewingKey { key, padding: None };
            auth_handle(deps, env, msg, AuthHandle) }

        /// User can lock some liquidity provision tokens.
        Lock (amount: Amount) {
            tx_ok!(ISnip20::attach(&load_lp_token(&deps.storage, &deps.api)?).transfer_from(
                &env.message.sender,
                &env.contract.address,
                Pool::new(&mut deps.storage)
                    .at(env.block.height)
                    .user(deps.api.canonical_address(&env.message.sender)?)
                    .lock_tokens(amount)? )? ) }

        /// User can always get their liquidity provision tokens back.
        Retrieve (amount: Amount) {
            tx_ok!(ISnip20::attach(&load_lp_token(&deps.storage, &deps.api)?).transfer(
                &env.message.sender,
                Pool::new(&mut deps.storage)
                    .at(env.block.height)
                    .user(deps.api.canonical_address(&env.message.sender)?)
                    .retrieve_tokens(amount)? )?) }

        /// User can receive rewards after having provided liquidity.
        Claim () {
            // TODO reset age on claim, so user can claim only once per reward period?
            let reward_token_link = load_reward_token(&deps.storage, &deps.api)?;
            let reward_token = ISnip20::attach(&reward_token_link);
            let reward_vk = load_viewing_key(&deps.storage)?.0;
            let reward_balance = reward_token.query(&deps.querier)
                .balance(&env.contract.address, &reward_vk)?;
            let reward = Pool::new(&mut deps.storage)
                .at(env.block.height)
                .with_balance(reward_balance)
                .user(deps.api.canonical_address(&env.message.sender)?)
                .claim_reward()?;
            let messages = vec![reward_token.transfer(&env.message.sender, reward)?];
            let log = vec![LogAttribute { key: "reward".into(), value: reward.into() }];
            Ok(HandleResponse { messages, log, data: None }) } } }
