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

use crate::msg::{HandleMsg, InitMsg, QueryMsg, UPPER_OVERFLOW_MSG};
use crate::state::{
    save_config, Config, add_pools, get_pool, get_account, save_account
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
        HandleMsg::AddPools { pools } => add_more_pools(deps, env, pools),
        HandleMsg::RemovePools { addresses } => remove_some_pools(deps, env, addresses), // Great naming btw
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

    let mut account = get_account(deps, env.message.sender, lp_token_addr)?;

    account.locked_amount = 
        account.locked_amount.checked_add(amount.u128())
        .ok_or_else(|| StdError::generic_err(UPPER_OVERFLOW_MSG))?;

    save_account(deps, &account)?;

    let pool = pool.unwrap(); // Safe, checked above
    
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
            log("lp_token", account.lp_token_addr)
        ],
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
