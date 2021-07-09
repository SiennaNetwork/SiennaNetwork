use fadroma::scrt::{
    BLOCK_SIZE,
    contract::*,
    addr::{Humanize, Canonize},
    callback::ContractInstance,
    toolkit::snip20,
    utils::viewing_key::ViewingKey,
    storage::{load, save}
};
use sienna_reward_schedule::stateful::RewardPoolController;
use composable_auth::{
    auth_handle, authenticate, AuthHandleMsg, DefaultHandleImpl
};

macro_rules! tx_ok {
    ($($msg:expr),*) => { Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None }) }
}

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) }
}

const KEY_THIS_CONTRACT: &[u8] = b"thi
use fadroma::scrt::addr::Humanize;


use serde_json::de;

s_contract";

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

contract! {

    [State] {
        provided_token: Option<ContractInstance<CanonicalAddr>>,
        rewarded_token: ContractInstance<CanonicalAddr>,
        viewing_key:    ViewingKey
    }

    [Init] (deps, env, msg: {
        provided_token: Option<ContractInstance<HumanAddr>>,
        rewarded_token: ContractInstance<HumanAddr>,
        viewing_key:    ViewingKey
    }) {
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
        save_state!(State {
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
                let (volume, total, since) = RewardPoolController::status(deps)?;
                Ok(Response::Status { volume, total, since })
            } else {
                error!("not configured")
            }
        }

        ClaimSimulation(address: HumanAddr, key: String) {
            let address = deps.api.canonical_address(&address)?;
            authenticate(&deps.storage, &ViewingKey(key), address.as_slice())?;

            let this_contract = load_contract_info(deps)?;
            let balance = snip20::balance_query(
                &deps.querier,
                this_contract.address,
                state.viewing_key.0.clone(),
                BLOCK_SIZE,
                state.rewarded_token.code_hash.clone(),
                deps.api.human_address(&state.rewarded_token.address)?
            )?;

            let reward_amount = RewardPoolController::get_claim_amount(deps, &address, balance.amount)?;

            Ok(Response::ClaimSimulation {
                reward_amount
            })
        }

        Pool() {
            let token = state.provided_token.ok_or_else(||
                StdError::generic_err("Contract hasn't been launched yet.")
            )?;

            Ok(Response::Pool {
                lp_token: token.humanize(&deps.api)?,
                volume: RewardPoolController::get_volume(deps)?
            })
        }

        // Keplr integration
        TokenInfo () {
            let token = state.provided_token.ok_or_else(||
                StdError::generic_err("Contract hasn't been launched yet.")
            )?;

            let info = snip20::token_info_query(
                &deps.querier,
                BLOCK_SIZE,
                token.code_hash,
                deps.api.human_address(&token.address)?
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

            Ok(Response::Balance {
                amount: RewardPoolController::get_balance(deps, &address)?
            })
        }
    }

    [Response] { 
        Status {
            volume: Uint128,
            total: Uint128,
            since: u64
        }

        TokenInfo {
            name: String,
            symbol: String,
            decimals: u8,
            total_supply: Option<Uint128>
        }

        Pool { 
            lp_token: ContractInstance<HumanAddr>,
            volume: Uint128
        }

        // Keplr integration
        Balance {
            amount: Uint128
        }

        ClaimSimulation {
            reward_amount: Uint128
        }
    }

    [Handle] (deps, env /* it's not unused :( */, state, msg) -> Response {

        /// Set the active asset token.
        // Resolves circular reference in benchmark -
        // they need to know each other's addresses to use initial allowances
        SetProvidedToken (address: HumanAddr, code_hash: String) {
            state.provided_token = Some(ContractInstance {
                address: deps.api.canonical_address(&address)?,
                code_hash
            });
            save_state!();
            Ok(HandleResponse::default())
        }

        /// Provide some liquidity.
        Lock (amount: Uint128) {
            if state.provided_token.is_none() { return error!("not configured") }
            let provided_token = state.provided_token.clone().unwrap();
            let address  = deps.api.canonical_address(&env.message.sender)?;
            let mut pool = RewardPoolController::new(deps);
            let locked   = pool.lock(env.block.height, address, amount)?;
            let transfer = snip20::transfer_from_msg(
                env.message.sender,
                env.contract.address,
                locked,
                None,
                BLOCK_SIZE,
                provided_token.code_hash,
                deps.api.human_address(&provided_token.address)?
            )?;
            save_state!();
            tx_ok!(transfer)
        }

        /// Get some tokens back.
        Retrieve (amount: Uint128) {
            if state.provided_token.is_none() { return error!("not configured") }
            let provided_token = state.provided_token.clone().unwrap();
            let address   = deps.api.canonical_address(&env.message.sender)?;
            let mut pool  = RewardPoolController::new(deps);
            let retrieved = pool.retrieve(env.block.height, address, amount)?;
            let transfer  = snip20::transfer_msg(
                env.message.sender,
                retrieved,
                None,
                BLOCK_SIZE,
                provided_token.code_hash,
                deps.api.human_address(&provided_token.address)?
            )?;
            save_state!();
            tx_ok!(transfer)
        }

        /// Receive rewards.
        Claim () {
            let balance = snip20::balance_query(
                &deps.querier,
                env.contract.address,
                state.viewing_key.0.clone(),
                BLOCK_SIZE,
                state.rewarded_token.code_hash.clone(),
                deps.api.human_address(&state.rewarded_token.address)?
            )?;

            let address  = deps.api.canonical_address(&env.message.sender)?;
            let mut pool = RewardPoolController::new(deps);
            let claimed  = pool.claim(&address, balance.amount)?;
            let transfer = snip20::transfer_msg(
                env.message.sender,
                claimed,
                None,
                BLOCK_SIZE,
                state.rewarded_token.code_hash.clone(),
                deps.api.human_address(&state.rewarded_token.address)?
            )?;
            save_state!();
            tx_ok!(transfer)
        }

        CreateViewingKey (
            entropy: String,
            padding: Option<String>
        ) {
            auth_handle(
                deps,
                env,
                AuthHandleMsg::CreateViewingKey { entropy, padding: None },
                DefaultHandleImpl
            )
        }

        SetViewingKey (
            key: String,
            padding: Option<String>
        ) {
            auth_handle(
                deps,
                env,
                AuthHandleMsg::SetViewingKey { key, padding: None },
                DefaultHandleImpl
            )
        }
    }
}
