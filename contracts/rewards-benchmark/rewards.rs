#[cfg(test)] #[macro_use] extern crate kukumba;
#[cfg(test)] mod rewards_harness;
#[cfg(test)] mod rewards_test;

pub mod rewards_math; use rewards_math::*;
mod rewards_algo; use rewards_algo::*;

use fadroma::scrt::{
    addr::{Humanize, Canonize},
    callback::{ContractInstance as ContractLink},
    contract::*,
    cosmwasm_std::ReadonlyStorage,
    snip20_api::ISnip20,
    storage::{load, save},
    utils::viewing_key::ViewingKey,
};
use composable_auth::{
    auth_handle, authenticate, AuthHandleMsg,
    DefaultHandleImpl as AuthHandle
};
use composable_admin::admin::{
    DefaultHandleImpl as AdminHandle,
    admin_handle, AdminHandleMsg, load_admin,
    assert_admin, save_admin
};

macro_rules! tx_ok { ($($msg:expr),*) => {
    Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None })
} }

macro_rules! error { ($info:expr) => {
    Err(StdError::GenericErr { msg: $info.into(), backtrace: None })
} }

pub const DAY: Time = 17280; // blocks over ~24h @ 5s/block

contract! {

    [NoGlobalState] {}

    [Init] (deps, env, msg: {
        admin:        Option<HumanAddr>,
        lp_token:     Option<ContractLink<HumanAddr>>,
        reward_token: ContractLink<HumanAddr>,
        viewing_key:  ViewingKey,
        ratio:        Option<Ratio>,
        threshold:    Option<Time>
    }) {

        // Contract has an admin who can do admin stuff.
        save_admin(deps, &admin.unwrap_or(env.message.sender))?;

        // Contract accepts transactions in `lp_token`s.
        // The address of the `lp_token` can be provided later
        // to avoid a circular dependency during deployment.
        if let Some(lp_token) = lp_token {
            save_lp_token(&mut deps.storage, &deps.api, &lp_token)?;
        }

        // Contract distributes rewards in Reward Tokens.
        // For this, it must know its own balance in the `reward_token`s.
        // For that, it needs a reference to its own address+code_hash
        // and a viewing key in `reward_token`.
        let set_vk = ISnip20::connect(reward_token).set_viewing_key(&viewing_key.0)?;
        save_reward_token(&mut deps.storage, &deps.api, &reward_token)?;
        save_viewing_key(&mut deps.storage, &viewing_key)?;
        save_self_reference(&mut deps.storage, &deps.api, &ContractLink {
            address: env.contract.address,
            code_hash: env.contract_code_hash
        })?;

        // Reward pool has configurable parameters:
        // - Ratio (to reduce everyone's rewards equally)
        // - Threshold (to incentivize users to lock tokens for longer)
        Pool::new(&mut deps.storage)
            .save_ratio(&ratio.unwrap_or((1u128.into(), 1u128.into())))?
            .save_threshold(&threshold.unwrap_or(DAY))?;

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
            let (pool_balance, pool_lifetime, pool_last_update) = Pool::new(&deps.storage).at(at).status()?;
            Ok(Response::PoolInfo {
                lp_token: load_lp_token(&deps.storage, &deps.api)?,

                it_is_now: at,

                pool_last_update,
                pool_lifetime,
                pool_balance
            })
        }

        /// Requires the user's viewing key.
        UserInfo (at: Time, address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;
            let token = load_reward_token(&deps.storage, &deps.api)?;
            let user = Pool::new(&deps.storage).at(now).with_balance(ISnip20::connect(token).query(&deps.querier).balance(
                &load_self_reference(&deps.storage, &deps.api)?.address,
                &load_viewing_key(&deps.storage)?.0,
            )?).user(address);
            let user_age = user.age();
            let (user_unlocked, user_claimed, user_claimable) = user.reward(reward_balance)?;
            Ok(Response::UserInfo {
                it_is_now: at,

                pool_last_update,
                pool_lifetime,
                pool_balance,

                user_last_update,
                user_lifetime,
                user_balance,

                user_age,
                user_unlocked,
                user_claimed,
                user_claimable
            }) }

        /// Keplr integration
        TokenInfo () {
            let token = load_lp_token(&deps.storage, &deps.api)?;
            let info  = ISnip20::connect(token).query(&deps.querier).token_info()?;
            Ok(Response::TokenInfo {
                name:         format!("Sienna Rewards: {}", info.name),
                symbol:       "SRW".into(),
                decimals:     1,
                total_supply: None }) }

        /// Keplr integration
        Balance (address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;
            Ok(Response::Balance { amount: Pool::new(&deps.storage).user(address).balance()? }) }

    }

    [Response] {

        /// Response from `Query::PoolInfo`
        PoolInfo {
            lp_token: ContractLink<HumanAddr>,

            it_is_now: Time,

            pool_last_update: Time,
            pool_lifetime:    Volume,
            pool_balance:     Amount
        }

        /// Response from `Query::UserInfo`
        UserInfo {
            it_is_now: Time,

            pool_last_update: Time,
            pool_lifetime:    Volume,
            pool_balance:     Amount,

            user_last_update: Time,
            user_lifetime:    Volume,
            user_balance:     Amount,

            user_age:         Time,
            user_unlocked:    Amount,
            user_claimed:     Amount,
            user_claimable:   Amount
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

        // Admin transactions

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
            Ok(HandleResponse::default()) }

        // User transactions

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
            tx_ok!(ISnip20::connect(load_lp_token(&deps.storage, &deps.api)?).transfer_from(
                &env.message.sender,
                &env.contract.address,
                Pool::new(&mut deps.storage).at(env.block.height)
                    .user(deps.api.canonical_address(&env.message.sender)?)
                    .lock_tokens(amount)? )? ) }

        /// User can always get their liquidity provision tokens back.
        Retrieve (amount: Amount) {
            tx_ok!(ISnip20::connect(load_lp_token(&deps.storage, &deps.api)?).transfer(
                &env.message.sender,
                Pool::new(&mut deps.storage).at(env.block.height)
                    .user(deps.api.canonical_address(&env.message.sender)?)
                    .retrieve_tokens(amount)? )?) }

        /// User can receive rewards after having provided liquidity.
        Claim () {
            // TODO reset age on claim, so user can claim only once per reward period?
            let reward = ISnip20::connect(load_reward_token(&deps.storage, &deps.api)?);
            let vk = load_viewing_key(&deps.storage)?.0;
            tx_ok!(reward.transfer(
                &env.message.sender,
                Pool::new(&mut deps.storage).at(env.block.height)
                    .user(deps.api.canonical_address(&env.message.sender)?)
                    .claim_reward(reward.query(&deps.querier).balance(
                        &env.contract.address,
                        &vk )?)? )?) }

    }
}

const POOL_SELF_REFERENCE: &[u8] = b"self";

fn load_self_reference(
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_SELF_REFERENCE)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing self reference")
    }
}

fn save_self_reference (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractLink<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_SELF_REFERENCE, &link.canonize(api)?)
}

const POOL_LP_TOKEN: &[u8] = b"lp_token";

fn load_lp_token (
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_LP_TOKEN)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing liquidity provision token")
    }
}

fn save_lp_token (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractLink<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_LP_TOKEN, &link.canonize(api)?)
}

const POOL_REWARD_TOKEN: &[u8] = b"reward_token";

fn load_reward_token (
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_REWARD_TOKEN)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing liquidity provision token")
    }
}

fn save_reward_token (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractLink<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_REWARD_TOKEN, &link.canonize(api)?)
}

const POOL_REWARD_TOKEN_VK: &[u8] = b"reward_token_vk";

fn load_viewing_key (
    storage: &impl ReadonlyStorage,
) -> StdResult<ViewingKey> {
    let result: Option<ViewingKey> = load(storage, POOL_REWARD_TOKEN_VK)?;
    match result {
        Some(key) => Ok(key),
        None => error!("missing reward token viewing key")
    }
}

fn save_viewing_key (
    storage: &mut impl Storage,
    key:     &ViewingKey
) -> StdResult<()> {
    save(storage, POOL_REWARD_TOKEN_VK, &key)
}
