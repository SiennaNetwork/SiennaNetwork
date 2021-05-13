use std::u128;

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
use cosmwasm_utils::convert::{convert_token, get_whole_token_representation};

use crate::msg::{HandleMsg, InitMsg, QueryMsg, OVERFLOW_MSG};
use crate::state::{
    save_config, Config, add_pools, get_pool, get_account, save_account,
    save_pool, delete_account, load_config
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

    let token_info = snip20::token_info_query(
        &deps.querier,
        BLOCK_SIZE,
        msg.reward_token.code_hash.clone(),
        msg.reward_token.address.clone()
    )?;

    let mut config = Config {
        reward_token: msg.reward_token,
        token_decimals: token_info.decimals,
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
                config.reward_token.code_hash,
                config.reward_token.address
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
    let mut pool = get_pool_or_fail(deps, &lp_token_addr)?;

    let mut account = get_account(deps, &env.message.sender, &lp_token_addr)?;

    account.locked_amount = 
        account.locked_amount.checked_add(amount.u128())
        .ok_or_else(|| StdError::generic_err(OVERFLOW_MSG))?;

    pool.size = pool.size.checked_add(amount.u128())
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
    let mut pool = get_pool_or_fail(deps, &lp_token_addr)?;

    let mut account = get_account(deps, &env.message.sender, &lp_token_addr)?;

    account.locked_amount = account.locked_amount.checked_sub(amount.u128())
        .ok_or_else(|| StdError::generic_err("Insufficient balance."))?;

    pool.size = pool.size.saturating_sub(amount.u128());

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

    let mut total_rewards_amount = 0;

    for addr in lp_tokens {
        let pool = get_pool_or_fail(deps, &addr)?;

        let mut account = get_account(deps, &env.message.sender, &addr)?;

        if account.locked_amount == 0 {
            return Err(StdError::generic_err(format!("This account has no tokens locked in {}", addr)));
        }

        let reward_amount = calc_reward_share(
            account.locked_amount,
            &pool,
            config.token_decimals
        )?;

        // TODO: not sure if actually possible
        if reward_amount == 0 {
            return Err(StdError::generic_err(format!(
                "Reward amount for {} is zero.", &addr)
            ));
        }

        let portions = if account.last_claimed == 0 {
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
                config.claim_interval - (env.block.time - account.last_claimed)
            )));
        }

        account.last_claimed = env.block.time;
        save_account(deps, &account)?;

        total_rewards_amount += reward_amount * portions;
    }

    let available_balance = snip20::balance_query(
        &deps.querier,
        env.contract.address,
        config.viewing_key.0,
        BLOCK_SIZE,
        config.reward_token.code_hash.clone(),
        config.reward_token.address.clone()
    )?;

    if total_rewards_amount > available_balance.amount.u128() {
        return Err(StdError::generic_err(
            "Insufficient amount of reward token in contract balance."
        ));
    }

    Ok(HandleResponse {
        messages: vec![
            snip20::transfer_msg(
                env.message.sender,
                Uint128(total_rewards_amount),
                None,
                BLOCK_SIZE,
                config.reward_token.code_hash,
                config.reward_token.address
            )?
        ],
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

fn calc_reward_share(
    mut user_locked: u128,
    pool: &RewardPool,
    reward_token_decimals: u8
) -> StdResult<u128> {
    user_locked *= 100; // Multiply by 100 to get a non float percentage

    // This error shouldn't really happen since the TX should already have failed.
    let share_percentage = user_locked.checked_div(pool.size).ok_or_else(|| 
        StdError::generic_err(format!("Pool size for {} is zero.", pool.lp_token.address))
    )?;

    // Convert to actual amount of reward token
    let share = share_percentage.checked_mul(
        // -2 to compensate for the multiplication above
        get_whole_token_representation(reward_token_decimals - 2)
    ).ok_or_else(|| 
        StdError::generic_err(OVERFLOW_MSG)
    )?;

    // share * pool.share / one reward token
    convert_token(
        share,
        pool.share,
        reward_token_decimals,
        reward_token_decimals
    )
}

#[inline]
fn get_pool_or_fail<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<RewardPool> { 
    get_pool(deps, &address)?.ok_or_else(||
        StdError::generic_err(format!(
            "LP token {} is not eligible for rewards.", address
        ))
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{testing::{mock_dependencies, mock_env}};
    use cosmwasm_std::{coins, from_binary, StdError};
    use cosmwasm_utils::ContractInfo;

    fn create_pool(share: u128, size: u128) -> RewardPool {
        RewardPool {
            lp_token: ContractInfo {
                address: HumanAddr::from("lp_token"),
                code_hash: "lp_token".into()
            },
            share,
            size
        }
    }

    #[test]
    fn test_calc_reward_share() {
        let pool = create_pool(
            500_000_000_000_000_000_000,
            1000_000_000
        );

        // If owning 15% of pool share, then receive 15% of 500 = 75
        let result = calc_reward_share(150_000_000, &pool, 18).unwrap();
        assert_eq!(result, 75_000_000_000_000_000_000);

        // Absorb the entire pool if owning 100% of pool share.
        let result = calc_reward_share(1000_000_000, &pool, 18).unwrap();
        assert_eq!(result, 500_000_000_000_000_000_000);
    }
}
