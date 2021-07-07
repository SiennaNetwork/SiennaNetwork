use fadroma::scrt::{
    BLOCK_SIZE,
    contract::*,
    addr::Canonize,
    callback::ContractInstance,
    toolkit::snip20,
    utils::viewing_key::ViewingKey
};
use sienna_reward_schedule::RewardPool;

macro_rules! tx_ok {
    ($($msg:expr),*) => { Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None }) }
}

macro_rules! error {
    ($info:expr) => { Err(StdError::GenericErr { msg: $info.into(), backtrace: None }) }
}

contract! {

    [State] {
        provided_token: Option<ContractInstance<CanonicalAddr>>,
        rewarded_token: ContractInstance<CanonicalAddr>,
        viewing_key:    ViewingKey,
        pool:           RewardPool
    }

    [Init] (deps, env, msg: {
        provided_token: Option<ContractInstance<HumanAddr>>,
        rewarded_token: ContractInstance<HumanAddr>,
        viewing_key:    ViewingKey
    }) {
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
            pool:           RewardPool::new(env.block.time),
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

    [Query] (_deps, _state, msg) -> Response {
        Status () {
            Ok(Response::Status {})
        }
    }

    [Response] { Status {} }

    [Handle] (deps, env /* it's not unused */, state, msg) -> Response {

        /// Set the active asset token.
        // Resolves circular reference in benchmark -
        // they need to know each other's addresses to use initial allowances
        SetProvidedToken (address: HumanAddr, code_hash: String) {
            let address = deps.api.canonical_address(&address)?;
            state.provided_token = Some(ContractInstance { address, code_hash });
            save_state!();
            Ok(HandleResponse::default())
        }

        /// Provide some liquidity.
        Lock (amount: Uint128) {
            if state.provided_token.is_none() { return error!("not configured") }
            let provided_token = state.provided_token.clone().unwrap();
            let address  = deps.api.canonical_address(&env.message.sender)?;
            let locked   = state.pool.lock(env.block.time, address, amount);
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
            let address = deps.api.canonical_address(&env.message.sender)?;
            let retrieved = state.pool.retrieve(env.block.time, address, amount)?;
            let transfer = snip20::transfer_msg(
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
                env.contract.address.clone(),
                state.viewing_key.0.clone(),
                BLOCK_SIZE,
                state.rewarded_token.code_hash.clone(),
                deps.api.human_address(&state.rewarded_token.address)?
            )?;
            let address = deps.api.canonical_address(&env.message.sender)?;
            let claimed = state.pool.claim(env.block.time, balance.amount, address)?;
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

    }
}
