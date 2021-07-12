use fadroma::scrt::{
    BLOCK_SIZE,
    contract::*,
    addr::{Humanize, Canonize},
    callback::ContractInstance,
    toolkit::snip20,
    utils::viewing_key::ViewingKey,
    storage::{load, save},
    cosmwasm_std::ReadonlyStorage as Readonly
};
use sienna_reward_schedule::stateful::RewardPoolController as Pool;
use composable_auth::{auth_handle, authenticate, AuthHandleMsg, DefaultHandleImpl};

macro_rules! tx_ok {
    ($($msg:expr),*) => { Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None }) }
}

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) }
}

contract! {

    [GlobalState] {
        provided_token: Option<ContractInstance<CanonicalAddr>>,
        rewarded_token: ContractInstance<CanonicalAddr>,
        viewing_key:    ViewingKey
    }

    [Init] (deps, env, msg: {
        provided_token: Option<ContractInstance<HumanAddr>>,
        rewarded_token: ContractInstance<HumanAddr>,
        viewing_key:    ViewingKey
    }) {
        // TODO proposed action against the triple generic:
        // scrt-contract automatically aliases `storage`, `api`,
        // and `querier` (also maybe the contents of `env`?)
        // so that it becomes less verbose to pass just the ones you use
        // (...that, or let's give contracts a `self` already.)
        set_admin(&mut deps.storage, &deps.api, &env, &env.message.sender)?;
        save_contract_info(deps, &ContractInstance {
            address: env.contract.address,
            code_hash: env.contract_code_hash
        })?;

        // canonize the asset token if it is provided
        // how do I unwrap option and result simultaneously?
        let provided_token = match provided_token {
            None                 => None,
            Some(provided_token) => Some(provided_token.canonize(&deps.api)?)
        };

        // store the initial configuration
        save_state!(GlobalState {
            provided_token,
            rewarded_token: rewarded_token.canonize(&deps.api)?,
            viewing_key:    viewing_key.clone()
        });

        // set ourselves a viewing key in the reward token
        // so we can check our balance and distribute portions of it
        let set_vk = snip20::set_viewing_key_msg(
            viewing_key.0,
            None, BLOCK_SIZE,
            rewarded_token.code_hash, rewarded_token.address
        )?;

        InitResponse { messages: vec![set_vk], log: vec![] }
    }

    [Query] (deps, state, msg) -> Response {
        Status (now: u64) {
            if let Some(_) = state.provided_token {
                let (volume, total, since) = Pool::status(deps)?;
                Ok(Response::Status { volume, total, since })
            } else {
                error!("not configured")
            }
        }

        Claimable (address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;
            let this_contract = load_contract_info(deps)?;
            let token = state.rewarded_token.humanize(&deps.api)?;
            Ok(Response::Claimable {
                reward_amount: Pool::get_claim_amount(deps, &address, snip20::balance_query(
                    &deps.querier,
                    this_contract.address, state.viewing_key.0.clone(),
                    BLOCK_SIZE,
                    token.code_hash.clone(), token.address
                )?.amount)?
            })
        }

        Pool () {
            Ok(Response::Pool {
                lp_token: get_provided_token(&state, &deps.api)?,
                volume:   Pool::get_volume(deps)?
            })
        }

        // Keplr integration
        TokenInfo () {
            let token = get_provided_token(&state, &deps.api)?;
            let info = snip20::token_info_query(
                &deps.querier,
                BLOCK_SIZE,
                token.code_hash, token.address
            )?;

            Ok(Response::TokenInfo {
                name: format!("Sienna Rewards: {}", info.name),
                symbol: "SRW".into(),
                decimals: 1,
                total_supply: None
            })
        }

        Balance (address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;
            Ok(Response::Balance { amount: Pool::get_balance(deps, &address)? })
        }
    }

    [Response] {
        Status { volume: Uint128, total: Uint128, since: u64 }
        TokenInfo { name: String, symbol: String, decimals: u8, total_supply: Option<Uint128> }
        Pool { lp_token: ContractInstance<HumanAddr>, volume: Uint128 }
        Balance { amount: Uint128 } // Keplr integration
        Claimable { reward_amount: Uint128 }
    }

    [Handle] (deps, env /* it's not unused :( */, state, msg) -> Response {

        /// Set the active asset token.
        // Resolves circular reference in benchmark -
        // they need to know each other's addresses to use initial allowances
        SetProvidedToken (address: HumanAddr, code_hash: String) {
            is_admin(&deps.storage, &deps.api, &env)?;
            state.provided_token = Some(ContractInstance {
                address: deps.api.canonical_address(&address)?,
                code_hash
            });
            save_state!();
            Ok(HandleResponse::default())
        }

        /// Provide some liquidity.
        Lock (amount: Uint128) {
            let token    = get_provided_token(&state, &deps.api)?;
            let address  = deps.api.canonical_address(&env.message.sender)?;
            let mut pool = Pool::new(deps);
            let locked   = pool.lock(env.block.height, address, amount)?;
            let transfer = snip20::transfer_from_msg(
                env.message.sender, env.contract.address, locked,
                None, BLOCK_SIZE,
                token.code_hash, token.address
            )?;
            save_state!();
            tx_ok!(transfer)
        }

        /// Get some tokens back.
        Retrieve (amount: Uint128) {
            let token     = get_provided_token(&state, &deps.api)?;
            let address   = deps.api.canonical_address(&env.message.sender)?;
            let mut pool  = Pool::new(deps);
            let retrieved = pool.retrieve(env.block.height, address, amount)?;
            let transfer  = snip20::transfer_msg(
                env.message.sender, retrieved,
                None, BLOCK_SIZE,
                token.code_hash, token.address
            )?;
            save_state!();
            tx_ok!(transfer)
        }

        /// User can receive rewards after having provided liquidity.
        Claim () {
            let token   = state.rewarded_token.humanize(&deps.api)?;
            let balance = snip20::balance_query(
                &deps.querier, env.contract.address, state.viewing_key.0.clone(),
                BLOCK_SIZE,
                token.code_hash.clone(), token.address.clone()
            )?;
            let address  = deps.api.canonical_address(&env.message.sender)?;
            let transfer = snip20::transfer_msg(
                env.message.sender, Pool::new(deps).claim(&address, balance.amount)?,
                None, BLOCK_SIZE,
                token.code_hash.clone(), token.address.clone()
            )?;
            save_state!();
            tx_ok!(transfer)
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

const KEY_ADMIN: &[u8] = b"admin";

fn is_admin (
    storage: &impl Readonly, api: &impl Api,
    env: &Env
) -> StdResult<()> {
    if load(storage, KEY_ADMIN)? == Some(api.canonical_address(&env.message.sender)?) {
        Ok(())
    } else {
        Err(StdError::unauthorized())
    }
}

fn set_admin (
    storage: &mut impl Storage, api: &impl Api,
    env: &Env, new_admin: &HumanAddr
) -> StdResult<()> {
    let current_admin = load(storage, KEY_ADMIN)?;
    if current_admin == None || current_admin == Some(api.canonical_address(&env.message.sender)?) {
        save(storage, KEY_ADMIN, &api.canonical_address(new_admin)?)
    } else {
        Err(StdError::unauthorized())
    }
}

const KEY_THIS_CONTRACT: &[u8] = b"self";

fn load_contract_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<ContractInstance<HumanAddr>> {
    let result: ContractInstance<CanonicalAddr> =
        load(&deps.storage, KEY_THIS_CONTRACT)?.unwrap();

    Ok(result.humanize(&deps.api)?)
}

fn save_contract_info<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    this_contract: &ContractInstance<HumanAddr>
) -> StdResult<()> {
    let this_contract = this_contract.canonize(&deps.api)?;

    save(&mut deps.storage, KEY_THIS_CONTRACT, &this_contract)
}

fn get_provided_token (
    state: &GlobalState,
    api:   &impl Api,
) -> StdResult<ContractInstance<HumanAddr>> {
    match &state.provided_token {
        Some(token) => Ok(token.humanize(api)?),
        None => error!("Contract hasn't been launched yet.")
    }
}
