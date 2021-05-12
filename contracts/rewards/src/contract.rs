use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdResult, Storage, Uint128, StdError, log
};
use composable_admin::require_admin;
use composable_admin::admin::{
    save_admin, admin_handle, admin_query, DefaultHandleImpl, DefaultQueryImpl,
    assert_admin
};
use secret_toolkit::snip20;
use cosmwasm_utils::viewing_key::ViewingKey;

use crate::{msg::{HandleMsg, InitMsg, QueryMsg, OVERFLOW_MSG}, state::load_config};
use crate::state::{
    save_config, Config, add_pools, get_pool, get_account, save_account,
    save_pool, delete_account
};
use crate::data::RewardPool;

const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice());

    let admin = msg.admin.unwrap_or(env.message.sender);
    save_admin(deps, &admin)?;

    let mut config = Config {
        sienna_token: msg.sienna_token,
        viewing_key,
        total_share: 0,
        claim_interval: msg.claim_interval
    };

    if let Some(pools) = msg.reward_pools {
        config.add_shares_checked(&pools)?;
        add_pools(deps, &pools)?;
    }

    save_config(deps, &config)?;

    Ok(InitResponse {
        messages: vec![
            snip20::set_viewing_key_msg(
                config.viewing_key.0,
                None,
                BLOCK_SIZE,
                config.sienna_token.code_hash,
                config.sienna_token.address
            )?
        ],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::LockTokens { amount, lp_token } => lock_tokens(deps, env, amount, lp_token),
        HandleMsg::RetrieveTokens { amount, lp_token } => retrieve_tokens(deps, env, amount, lp_token),
        HandleMsg::Claim { lp_tokens } => claim(deps, env, lp_tokens),
        HandleMsg::AddPools { pools } => add_more_pools(deps, env, pools),
        HandleMsg::RemovePools { lp_tokens } => remove_some_pools(deps, env, lp_tokens), // Great naming btw
        HandleMsg::ChangeClaimInterval { interval } => change_claim_interval(deps, env, interval),
        HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultHandleImpl)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl)
    }
}

fn lock_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    lp_token_addr: HumanAddr
) -> StdResult<HandleResponse> {
    let pool = get_pool(deps, &lp_token_addr)?;

    if pool.is_none() {
        return  Err(StdError::generic_err(format!("LP token {} is not eligible for rewards.", lp_token_addr)));
    }

    let mut pool = pool.unwrap(); // Safe, checked above
    let mut account = get_account(deps, &env.message.sender, &lp_token_addr)?;

    account.locked_amount = 
        account.locked_amount.checked_add(amount.u128())
        .ok_or_else(|| StdError::generic_err(OVERFLOW_MSG))?;

    pool.size += pool.size.checked_add(amount.u128())
        .ok_or_else(|| StdError::generic_err(OVERFLOW_MSG))?;

    save_account(deps, &account)?;
    save_pool(deps, &pool)?;
    
    Ok(HandleResponse{
        messages: vec![
            snip20::transfer_from_msg(
                account.owner.clone(),
                env.contract.address,
                amount,
                None,
                BLOCK_SIZE,
                pool.lp_token.code_hash,
                pool.lp_token.address
            )?
        ],
        log: vec![
            log("action", "lock_tokens"),
            log("amount_locked", amount),
            log("locked_by", account.owner),
            log("lp_token", account.lp_token_addr),
            log("new_pool_size", pool.size)
        ],
        data: None
    })
}

fn retrieve_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    lp_token_addr: HumanAddr
) -> StdResult<HandleResponse> {
    let pool = get_pool(deps, &lp_token_addr)?;

    if pool.is_none() {
        return  Err(StdError::generic_err(format!("LP token {} is not eligible for rewards.", lp_token_addr)));
    }

    let mut pool = pool.unwrap(); // Safe, checked above
    let mut account = get_account(deps, &env.message.sender, &lp_token_addr)?;

    account.locked_amount = account.locked_amount.checked_sub(amount.u128())
        .ok_or_else(|| StdError::generic_err("Insufficient balance."))?;

    pool.size += pool.size.saturating_sub(amount.u128());

    if account.locked_amount == 0 {
        delete_account(deps, &account)?;
    } else {
        save_account(deps, &account)?;
    }

    save_pool(deps, &pool)?;
    
    Ok(HandleResponse{
        messages: vec![
            snip20::transfer_msg(
                account.owner.clone(),
                amount,
                None,
                BLOCK_SIZE,
                pool.lp_token.code_hash,
                pool.lp_token.address
            )?
        ],
        log: vec![
            log("action", "retrieve_tokens"),
            log("amount_retrieved", amount),
            log("retrieved_by", account.owner),
            log("lp_token", account.lp_token_addr),
            log("new_pool_size", pool.size)
        ],
        data: None
    })
}

fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    lp_tokens: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    let config = load_config(deps)?;

    let mut messages = vec![];

    for addr in lp_tokens {
        let mut account = get_account(deps, &env.message.sender, &addr)?;

        if account.locked_amount == 0 {
            return Err(StdError::generic_err(format!("This account has no tokens locked in {}", addr)));
        }

        let pool = get_pool(deps, &addr)?;

        let portions = if account.last_claimed == 0{
            1 // This account is claiming for the first time
        } else {
            // Could do (current_time - last_claimed) / interval
            // but can't use floats so...
            let mut result = 0;

            let gap = env.block.time - account.last_claimed;
            let mut acc = config.claim_interval;

            while gap > acc {
                acc += config.claim_interval;
                result += 1;
            }

            result
        };

        if portions == 0 {
            return Err(StdError::generic_err(format!(
                "Need to wait {} more time before claiming.",
                env.block.time - account.last_claimed
            )));
        }

        account.last_claimed = env.block.time;
        save_account(deps, &account)?;
    }

    Ok(HandleResponse {
        messages,
        log: vec![],
        data: None
    })
}

#[require_admin]
fn add_more_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pools: Vec<RewardPool>
) -> StdResult<HandleResponse>{
    unimplemented!()
}

#[require_admin]
fn remove_some_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<HumanAddr>
) -> StdResult<HandleResponse>{
    unimplemented!()
}

#[require_admin]
fn change_claim_interval<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    interval: u64
) -> StdResult<HandleResponse> {
    unimplemented!()
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
}
*/
