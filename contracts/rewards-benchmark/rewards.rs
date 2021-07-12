use fadroma::scrt::{
    BLOCK_SIZE,
    contract::*,
    addr::{Humanize, Canonize},
    callback::ContractInstance,
    toolkit::snip20,
    utils::viewing_key::ViewingKey,
    storage::{load, save},
    cosmwasm_std::ReadonlyStorage
};
use sienna_reward_schedule::stateful::{
    RewardPoolController as Pool,
    RewardPoolCalculations,
    Monotonic,
    Ratio
};
use composable_auth::{auth_handle, authenticate, AuthHandleMsg, DefaultHandleImpl};

macro_rules! tx_ok {
    ($($msg:expr),*) => { Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None }) }
}

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) }
}

contract! {

    [NoGlobalState] {}

    [Init] (deps, env, msg: {
        lp_token:     Option<ContractInstance<HumanAddr>>,
        reward_token: ContractInstance<HumanAddr>,
        viewing_key:  ViewingKey,
        ratio:        Option<Ratio>,
        threshold:    Option<Monotonic>
    }) {
        // TODO proposed action against the triple generic:
        // scrt-contract automatically aliases `storage`, `api`,
        // and `querier` (also maybe the contents of `env`?)
        // so that it becomes less verbose to pass just the ones you use
        // (...that, or let's just give contracts a `self` already?)

        // configure admin
        set_admin(&mut deps.storage, &deps.api, &env, &env.message.sender)?;

        // save self reference - used to check own balance

        // to check reward token balance:
        // save references to self and the reward token
        // and set ourselves a viewing key so we know how much we're dividing
        save_self_reference(&mut deps.storage, &deps.api, &ContractInstance {
            address: env.contract.address,
            code_hash: env.contract_code_hash
        })?;
        save_reward_token(&mut deps.storage, &deps.api, &reward_token)?;
        save_viewing_key(&mut deps.storage, &viewing_key)?;
        let set_vk = snip20::set_viewing_key_msg(
            viewing_key.0,
            None, BLOCK_SIZE,
            reward_token.code_hash, reward_token.address
        )?;

        // needed to start calculating rewards
        // but can be provided later
        if let Some(lp_token) = lp_token {
            save_lp_token(&mut deps.storage, &deps.api, &lp_token)?;
        }

        // save address of reward token, reward ratio, and minimum age
        Pool::new(&mut deps.storage)
            .save_ratio(&ratio.unwrap_or((1u128.into(), 1u128.into())))?
            .save_threshold(&threshold.unwrap_or(17280u64))?; // ~24h @ 5s/block

        // TODO remove global state from scrt-contract
        // define field! and addr_field! macros instead -
        // problem here is identifier concatenation
        // and making each field a module is ugly
        save_state!(NoGlobalState {});

        InitResponse { messages: vec![set_vk], log: vec![] }
    }

    [Query] (deps, _state, msg) -> Response {

        /// Overall pool status
        PoolInfo (now: Monotonic) {
            let lp_token = load_lp_token(&deps.storage, &deps.api)?;
            let (volume, total, since) = Pool::new(&deps.storage).pool_status(now)?;
            Ok(Response::PoolInfo {
                lp_token,
                volume,
                total,
                since,
                now,
            })
        }

        /// Requires the user's viewing key.
        UserInfo (now: Monotonic, address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;
            let token = load_reward_token(&deps.storage, &deps.api)?;
            let balance = snip20::balance_query(
                &deps.querier,
                load_self_reference(&deps.storage, &deps.api)?.address,
                load_viewing_key(&deps.storage)?.0.clone(),
                BLOCK_SIZE,
                token.code_hash.clone(), token.address
            )?.amount;
            let (unlocked, claimed, claimable) = Pool::new(&deps.storage)
                .user_reward(now, balance, &address)?;
            Ok(Response::UserInfo {
                age:      0,
                volume:   Uint128::zero(),
                lifetime: Uint128::zero(),
                unlocked,
                claimed,
                claimable
            })
        }

        /// Keplr integration
        TokenInfo () {
            let token = load_lp_token(&deps.storage, &deps.api)?;
            let info = snip20::token_info_query(
                &deps.querier,
                BLOCK_SIZE,
                token.code_hash, token.address
            )?;
            Ok(Response::TokenInfo {
                name:         format!("Sienna Rewards: {}", info.name),
                symbol:       "SRW".into(),
                decimals:     1,
                total_supply: None
            })
        }

        /// Keplr integration
        Balance (address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;
            Ok(Response::Balance {
                amount: Pool::new(&deps.storage).user_volume(&address)?
            })
        }

    }

    [Response] {

        /// Response from `Query::PoolInfo`
        PoolInfo {
            lp_token: ContractInstance<HumanAddr>,
            volume:   Uint128,
            total:    Uint128,
            since:    Monotonic,
            now:      Monotonic
        }

        /// Response from `Query::UserInfo`
        UserInfo {
            age:       Monotonic,
            volume:    Uint128,
            lifetime:  Uint128,
            unlocked:  Uint128,
            claimed:   Uint128,
            claimable: Uint128
        }

        /// Keplr integration
        TokenInfo {
            name:         String,
            symbol:       String,
            decimals:     u8,
            total_supply: Option<Uint128>
        }

        /// Keplr integration
        Balance {
            amount: Uint128
        }

    }

    [Handle] (deps, env /* it's not unused :( */, _state, msg) -> Response {

        /// Set the active asset token.
        // Resolves circular reference in benchmark -
        // they need to know each other's addresses to use initial allowances
        SetProvidedToken (address: HumanAddr, code_hash: String) {
            is_admin(&deps.storage, &deps.api, &env)?;
            save_lp_token(&mut deps.storage, &deps.api, &ContractInstance { address, code_hash })?;
            Ok(HandleResponse::default())
        }

        /// Provide some liquidity.
        Lock (amount: Uint128) {
            let token    = load_lp_token(&deps.storage, &deps.api)?;
            let transfer = snip20::transfer_from_msg(
                env.message.sender.clone(),
                env.contract.address,
                Pool::new(&mut deps.storage).user_lock(
                    env.block.height,
                    deps.api.canonical_address(&env.message.sender)?,
                    amount
                )?,
                None, BLOCK_SIZE, token.code_hash, token.address
            )?;
            tx_ok!(transfer)
        }

        /// Get some tokens back.
        Retrieve (amount: Uint128) {
            let token    = load_lp_token(&deps.storage, &deps.api)?;
            let transfer = snip20::transfer_msg(
                env.message.sender.clone(),
                Pool::new(&mut deps.storage).user_retrieve(
                    env.block.height,
                    deps.api.canonical_address(&env.message.sender)?,
                    amount
                )?,
                None, BLOCK_SIZE, token.code_hash, token.address
            )?;
            tx_ok!(transfer)
        }

        /// User can receive rewards after having provided liquidity.
        Claim () {
            let token = load_reward_token(&deps.storage, &deps.api)?;
            let balance = snip20::balance_query(
                &deps.querier, env.contract.address,
                load_viewing_key(&deps.storage)?.0.clone(),
                BLOCK_SIZE,
                token.code_hash.clone(), token.address.clone()
            )?.amount;
            let claimable = Pool::new(&mut deps.storage).user_claim(
                env.block.height,
                balance,
                &deps.api.canonical_address(&env.message.sender)?
            )?;
            tx_ok!(snip20::transfer_msg(
                env.message.sender,
                claimable,
                None, BLOCK_SIZE, token.code_hash, token.address
            )?)
        }

        /// User can request a new viewing key for oneself.
        CreateViewingKey (entropy: String, padding: Option<String>) {
            let msg = AuthHandleMsg::CreateViewingKey { entropy, padding: None };
            auth_handle(deps, env, msg, DefaultHandleImpl)
        }

        /// User can set own viewing key to a known value.
        SetViewingKey (key: String, padding: Option<String>) {
            let msg = AuthHandleMsg::SetViewingKey { key, padding: None };
            auth_handle(deps, env, msg, DefaultHandleImpl)
        }
    }
}

const POOL_ADMIN: &[u8] = b"admin";

fn is_admin (
    storage: &impl ReadonlyStorage,
    api: &impl Api,
    env: &Env
) -> StdResult<()> {
    if load(storage, POOL_ADMIN)? == Some(api.canonical_address(&env.message.sender)?) {
        Ok(())
    } else {
        Err(StdError::unauthorized())
    }
}

fn set_admin (
    storage: &mut impl Storage,
    api: &impl Api,
    env: &Env, new_admin: &HumanAddr
) -> StdResult<()> {
    let current_admin = load(storage, POOL_ADMIN)?;
    if current_admin == None || current_admin == Some(api.canonical_address(&env.message.sender)?) {
        save(storage, POOL_ADMIN, &api.canonical_address(new_admin)?)
    } else {
        Err(StdError::unauthorized())
    }
}

const POOL_SELF_REFERENCE: &[u8] = b"self";

fn load_self_reference(
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractInstance<HumanAddr>> {
    let result: Option<ContractInstance<CanonicalAddr>> = load(storage, POOL_SELF_REFERENCE)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing self reference")
    }
}

fn save_self_reference (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractInstance<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_SELF_REFERENCE, &link.canonize(api)?)
}

const POOL_LP_TOKEN: &[u8] = b"lp_token";

fn load_lp_token (
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractInstance<HumanAddr>> {
    let result: Option<ContractInstance<CanonicalAddr>> = load(storage, POOL_LP_TOKEN)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing liquidity provision token")
    }
}

fn save_lp_token (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractInstance<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_LP_TOKEN, &link.canonize(api)?)
}

const POOL_REWARD_TOKEN: &[u8] = b"reward_token";

fn load_reward_token (
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractInstance<HumanAddr>> {
    let result: Option<ContractInstance<CanonicalAddr>> = load(storage, POOL_REWARD_TOKEN)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing liquidity provision token")
    }
}

fn save_reward_token (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractInstance<HumanAddr>
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
