use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdResult, Storage, Uint128, StdError, log, to_binary
};
use composable_admin::require_admin;
use composable_admin::admin::{
    save_admin, admin_handle, admin_query, DefaultHandleImpl as DefaultAdminHandle,
    DefaultQueryImpl, assert_admin
};
use composable_auth::{auth_handle, authenticate, AuthHandleMsg};
use secret_toolkit::snip20;
use fadroma_scrt_callback::ContractInstance;
use cosmwasm_utils::viewing_key::ViewingKey;
use cosmwasm_utils::convert::{convert_token, get_whole_token_representation};

use crate::msg::{
    HandleMsg, InitMsg, QueryMsg, OVERFLOW_MSG, RewardPoolConfig,
    QueryMsgResponse, ClaimSimulationResult, ClaimResult, ClaimError,
    GetBalanceError
};
use crate::state::{
    save_config, Config, replace_active_pools, get_pool, get_account, save_account,
    get_or_create_account, save_pool, delete_account, load_config, get_pools,
    get_inactive_pool
};
use crate::data::{RewardPool, Account};
use crate::auth::AuthImpl;

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

    let config = Config {
        reward_token: msg.reward_token,
        this_contract: ContractInstance {
            address: env.contract.address,
            code_hash: env.contract_code_hash
        },
        token_decimals: token_info.decimals,
        viewing_key,
        prng_seed: msg.prng_seed,
        claim_interval: msg.claim_interval
    };

    save_config(deps, &config)?;

    if let Some(pools) = msg.reward_pools {
        let pools = into_pools(pools);
        replace_active_pools(deps, &pools)?;
    }

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
        HandleMsg::ChangePools { pools, total_share } => change_pools(deps, env, into_pools(pools), total_share),
        HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultAdminHandle),
        HandleMsg::CreateViewingKey { entropy, .. } =>
            auth_handle(deps, env, AuthHandleMsg::CreateViewingKey { entropy, padding: None }, AuthImpl),
        HandleMsg::SetViewingKey { key, .. } =>
            auth_handle(deps, env, AuthHandleMsg::SetViewingKey { key, padding: None }, AuthImpl)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::ClaimSimulation { lp_tokens, viewing_key, address, current_time } => 
            claim_simulation(deps, lp_tokens, address, ViewingKey(viewing_key), current_time),
        QueryMsg::Accounts { address, viewing_key, lp_tokens } =>
            query_accounts(deps, address, ViewingKey(viewing_key), lp_tokens),
        QueryMsg::Pools => query_pools(deps),

        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl),
        QueryMsg::TotalRewardsSupply => query_supply(deps),
        // Keplr support:
        QueryMsg::TokenInfo { } => to_binary(&QueryMsgResponse::TokenInfo {
            name: "Sienna Rewards".into(),
            symbol: "SRW".into(),
            decimals: 1,
            total_supply: None
        }),
        QueryMsg::Balance { .. } => to_binary(&QueryMsgResponse::Balance {
            amount: Uint128::zero()
        })
    }
}

/// Lock the provided `amount` of tokens associated with the given LP token address.
/// Increases the total locked for the sender as well as the entire pool size by the given `amount`.
/// Returns an error if the token address is not among the reward pools.
pub(crate) fn lock_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    lp_token_addr: HumanAddr
) -> StdResult<HandleResponse> {
    let mut pool = get_pool_or_fail(deps, &lp_token_addr)?;
    let mut account = get_or_create_account(deps, &env.message.sender, &lp_token_addr)?;

    account.locked_amount = 
        account.locked_amount.u128().checked_add(amount.u128())
        .ok_or_else(|| StdError::generic_err(OVERFLOW_MSG))?.into();

    pool.size = pool.size.u128().checked_add(amount.u128())
        .ok_or_else(|| StdError::generic_err(OVERFLOW_MSG))?.into();

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

/// Tranfer back the specified `amount` of LP tokens to the sender.
/// Also works even if the pool is not eligible for rewards anymore.
pub(crate) fn retrieve_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    lp_token_addr: HumanAddr
) -> StdResult<HandleResponse> {
    let mut account = get_account_or_fail(deps, &env.message.sender, &lp_token_addr)?;

    account.locked_amount = account.locked_amount.u128().checked_sub(amount.u128())
        .ok_or_else(|| StdError::generic_err("Insufficient balance."))?.into();

    if account.locked_amount == Uint128::zero() {
        delete_account(deps, &account)?;
    } else {
        save_account(deps, &account)?;
    }

    let pool = if let Some(mut p) = get_pool(deps, &lp_token_addr)? {
        p.size = p.size.u128().saturating_sub(amount.u128()).into();
        save_pool(deps, &p)?;

        Some(p)
    } else {
        get_inactive_pool(deps, &lp_token_addr)?
    }.ok_or_else(||
        StdError::generic_err(
            format!("Pool {} doesn't exist.", lp_token_addr)
        )
    )?;
    
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

/// Calculate and transfer the reward token amount for the specified LP token addresses.
/// If the calculated rewards amount exceeds the current rewards balance, the resulting
/// amount is truncated to fit the available rewards balance.
pub(crate) fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    lp_tokens: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    let config = load_config(deps)?;
    let available_balance = get_balance(&deps.querier, &config)?;

    let mut total_rewards_amount: u128 = 0;

    for addr in lp_tokens {
        let pool = get_pool_or_fail(deps, &addr)?;
        let mut account = get_account_or_fail(deps, &env.message.sender, &addr)?;

        let reward_amount = calc_reward_share(
            account.locked_amount.u128(),
            &pool,
            config.token_decimals
        )?;

        if reward_amount == 0 {
            return Err(StdError::generic_err(format!(
                "Reward amount for {} is zero.", &addr
            )));
        }

        let portions = calc_portions(
            account.last_claimed,
            config.claim_interval,
            env.block.time
        )?;

        if portions == 0 {
            return Err(StdError::generic_err(format!(
                "Need to wait {} more time before claiming.",
                config.claim_interval - (env.block.time - account.last_claimed)
            )));
        }

        account.last_claimed = env.block.time;
        save_account(deps, &account)?;

        total_rewards_amount = total_rewards_amount.saturating_add(
            reward_amount.saturating_mul(portions as u128)
        );
    }

    // Claim the remaining rewards amount if the current rewards pool,
    // is less than what should be claimed.
    let claim_amount = if total_rewards_amount > available_balance {
        available_balance
    } else {
        total_rewards_amount
    };

    Ok(HandleResponse {
        messages: vec![
            snip20::transfer_msg(
                env.message.sender.clone(),
                Uint128(claim_amount),
                None,
                BLOCK_SIZE,
                config.reward_token.code_hash,
                config.reward_token.address
            )?
        ],
        log: vec![
            log("action", "claim"),
            log("claimed_by", env.message.sender),
            log("total_rewards_amount", total_rewards_amount),
            log("claimed_amount", claim_amount)
        ],
        data: None
    })
}

/// Dry runs the claim method providing a detailed result for each of
/// the provided LP token addresses. Records the actual would be
/// errors, if any, instead of terminating the function early.
fn claim_simulation<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    lp_tokens: Vec<HumanAddr>,
    sender: HumanAddr,
    key: ViewingKey,
    current_time: u64
) -> StdResult<Binary> {
    let canonical = deps.api.canonical_address(&sender)?;
    authenticate(&deps.storage, &key, canonical.as_slice())?;

    let config = load_config(deps)?;

    let available_balance = match get_balance(&deps.querier, &config) {
        Ok(balance) => balance,
        Err(err) => {
            match err {
                GetBalanceError::StdError(std_err) => {
                    return Err(std_err);
                }
                GetBalanceError::PoolEmpty => {
                    let mut results = Vec::with_capacity(lp_tokens.len());

                    for addr in lp_tokens {
                        results.push(ClaimResult::error(addr, ClaimError::PoolEmpty))
                    }
                    
                    return Ok(to_binary(&QueryMsgResponse::ClaimSimulation(
                        ClaimSimulationResult {
                            total_rewards_amount: Uint128::zero(),
                            actual_claimed: Uint128::zero(),
                            results
                        }
                    ))?);
                }
            }
        }
    };
    
    let mut total_rewards_amount: u128 = 0;

    let mut results = Vec::with_capacity(lp_tokens.len());

    for addr in lp_tokens {
        let pool = get_pool_or_fail(deps, &addr)?;
        let account = get_or_create_account(deps, &sender, &addr)?;

        if account.locked_amount == Uint128::zero() {
            results.push(ClaimResult::error(addr, ClaimError::AccountZeroLocked));
            continue;
        }

        let reward_per_portion = calc_reward_share(
            account.locked_amount.u128(),
            &pool,
            config.token_decimals
        )?;

        if reward_per_portion == 0 {
            results.push(ClaimResult::error(addr, ClaimError::AccountZeroReward));
            continue;
        }

        let portions = calc_portions(
            account.last_claimed,
            config.claim_interval,
            current_time
        )?;

        if portions == 0 {
            results.push(ClaimResult::error(addr, ClaimError::EarlyClaim {
                time_to_wait: config.claim_interval - (current_time - account.last_claimed)
            }));
            continue;
        }

        let reward_amount = reward_per_portion.saturating_mul(portions as u128);
        results.push(ClaimResult::success(addr, Uint128(reward_amount), Uint128(reward_per_portion)));

        total_rewards_amount = total_rewards_amount.saturating_add(reward_amount);
    }

    let claim_amount = if total_rewards_amount > available_balance {
        available_balance
    } else {
        total_rewards_amount
    };

    Ok(to_binary(&QueryMsgResponse::ClaimSimulation(
        ClaimSimulationResult {
            total_rewards_amount: Uint128(total_rewards_amount),
            actual_claimed: Uint128(claim_amount),
            results
        }
    ))?)
}

/// Admin only command. Replaces the current reward pools with the ones provided.
/// Keeps the existing pool sizes for the ones that should remain. Instead of deleting
/// any newly redundant pools, they are marked as inactive in order to allow liquidity providers
/// to withdraw their shares using the `retrieve_tokens` method.
#[require_admin]
fn change_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pools: Vec<RewardPool<HumanAddr>>,
    total_share: Uint128
) -> StdResult<HandleResponse>{
    let mut sum_total: u128 = 0;

    for pool in pools.iter() {
        sum_total = sum_total.checked_add(pool.share.u128()).ok_or_else(||
            StdError::generic_err(OVERFLOW_MSG)
        )?;
    }

    if total_share.u128() != sum_total {
        return Err(StdError::generic_err(
            format!("Total pool share({}) doesn't match the expected total({}).", sum_total, total_share)
        ))
    }

    replace_active_pools(deps, &pools)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "change_pools"),
        ],
        data: None
    })
}

/// Returns all the currently active reward pools.
fn query_pools<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let pools = get_pools(deps)?;

    Ok(to_binary(&QueryMsgResponse::Pools(pools))?)
}

/// Authenticated command. Returns all the accounts that are
/// associated with the provided `address` given the LP token
/// addresses, since a single address can have multiple
/// accounts - one for each reward pool.
fn query_accounts<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
    key: ViewingKey,
    lp_tokens:Vec<HumanAddr>
) -> StdResult<Binary> {
    let canonical = deps.api.canonical_address(&address)?;
    authenticate(&deps.storage, &key, canonical.as_slice())?;

    let mut result = Vec::with_capacity(lp_tokens.len());

    for addr in lp_tokens {
        let account = get_or_create_account(deps, &address, &addr)?;
        result.push(account);
    }

    Ok(to_binary(&QueryMsgResponse::Accounts(result))?)
}

/// Returns the available balance of reward tokens that
/// this contract currently has to work with.
fn query_supply<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Binary> {
    let config = load_config(deps)?;
    
    let balance = match get_balance(&deps.querier, &config) {
        Ok(balance) => balance,
        Err(err) => {
            match err {
                GetBalanceError::StdError(std_err) => {
                    return Err(std_err);
                }
                GetBalanceError::PoolEmpty => {
                    0
                }
            }
        }
    };

    Ok(to_binary(&QueryMsgResponse::TotalRewardsSupply {
        amount: Uint128(balance)
    })?)
}

/// Given a `pool`, calculates the amount of rewards for a single portion.
pub(crate) fn calc_reward_share(
    mut user_locked: u128,
    pool: &RewardPool<HumanAddr>,
    reward_token_decimals: u8
) -> StdResult<u128> {
    // Multiply by 100 to get a non float percentage
    user_locked = user_locked.checked_mul(100).ok_or_else(||
        StdError::generic_err(OVERFLOW_MSG)
    )?;

    // This error shouldn't really happen since the TX should already have failed.
    let share_percentage = user_locked.checked_div(pool.size.u128()).ok_or_else(|| 
        StdError::generic_err(format!("Pool size for {} is zero.", pool.lp_token.address))
    )?;

    // Convert to actual amount of reward token
    let share = share_percentage.saturating_mul(
        // -2 to compensate for the multiplication above
        get_whole_token_representation(reward_token_decimals - 2)
    );

    // share * pool.share / one reward token
    convert_token(
        share,
        pool.share.u128(),
        reward_token_decimals,
        reward_token_decimals
    )
}

#[inline]
pub(crate) fn into_pools(mut vec: Vec<RewardPoolConfig>) -> Vec<RewardPool<HumanAddr>> {
    vec.drain(..).map(|p| p.into()).collect()
}

/// Calculates how many portions should be transferred. The amount of portions
/// depends on when the `claim` method was last called. Ex. given a claim interval
/// of 1 day and user who hasn't claimed their rewards for 3 days, then they should
/// earn 3x of their reward share. So the resulting portions would be 3.
pub(crate) fn calc_portions(
    last_claimed: u64,
    claim_interval: u64,
    block_time: u64
) -> StdResult<u32> {
    if last_claimed == 0 {
        return Ok(1); // This account is claiming for the first time
    }

    // Could do (current_time - last_claimed) / interval
    // but can't use floats so...
    let mut result = 0;
    
    let gap = block_time.checked_sub(last_claimed).ok_or_else(|| 
        // Will happen if a wrong time has been provided in claim simulation
        StdError::generic_err("Invalid timestamp supplied.")
    )?;
    let mut acc = claim_interval;

    while gap >= acc {
        acc += claim_interval;
        result += 1;
    }

    Ok(result)
}

#[inline]
fn get_pool_or_fail<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<RewardPool<HumanAddr>> { 
    get_pool(deps, address)?.ok_or_else(||
        StdError::generic_err(format!(
            "LP token {} is not eligible for rewards.", address
        ))
    )
}

#[inline]
fn get_account_or_fail<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
    lp_token: &HumanAddr
) -> StdResult<Account<HumanAddr>> { 
    get_account(deps, address, lp_token)?.ok_or_else(||
        StdError::generic_err(format!(
            "No account for {} exists for address {}.",
            address,
            lp_token
        ))
    )
}

/// Returns the available balance of reward tokens that
/// this contract currently has to work with. Returns a 
/// special error type to differentiate between a balance of 0
/// or something else that went wrong.
fn get_balance(querier: &impl Querier, config: &Config<HumanAddr>) -> Result<u128, GetBalanceError> {
    let available_balance = snip20::balance_query(
        querier,
        config.this_contract.address.clone(),
        config.viewing_key.0.clone(),
        BLOCK_SIZE,
        config.reward_token.code_hash.clone(),
        config.reward_token.address.clone()
    )?;

    let available_balance = available_balance.amount.u128();

    if available_balance == 0 {
        return Err(GetBalanceError::PoolEmpty);
    }

    Ok(available_balance)
}
