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
    HandleMsg, InitMsg, QueryMsg, OVERFLOW_MSG,
    QueryMsgResponse, ClaimSimulationResult, ClaimError,
    GetBalanceError
};
use crate::state::{
    save_config, Config, load_pool, get_account, save_account,
    get_or_create_account, save_pool, load_config
};
use crate::data::{RewardPool, Account, PendingBalance};
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
    save_pool(deps, &msg.pool.into())?;

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
        HandleMsg::LockTokens { amount } => lock_tokens(deps, env, amount),
        HandleMsg::RetrieveTokens { amount } => retrieve_tokens(deps, env, amount),
        HandleMsg::Claim => claim(deps, env),
        HandleMsg::ChangePoolShare { new_share } => change_pool_share(deps, env, new_share),
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
        QueryMsg::ClaimSimulation { viewing_key, address, current_time } => 
            claim_simulation(deps, address, ViewingKey(viewing_key), current_time),
        QueryMsg::Account { address, viewing_key } =>
            query_account(deps, address, ViewingKey(viewing_key)),
        QueryMsg::Pool => query_pool(deps),
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
    amount: Uint128
) -> StdResult<HandleResponse> {
    if amount == Uint128::zero() {
        return Err(StdError::generic_err("Lock amount is zero."));
    }

    let mut pool = load_pool(&deps)?;
    let mut account = get_or_create_account(deps, &env.message.sender)?;

    account.add_pending_balance(PendingBalance { 
        amount,
        submitted_at: env.block.time 
    })?;

    // Prevent instant claiming for new accounts
    if account.last_claimed == 0 {
        account.last_claimed = env.block.time;
    }

    pool.size = pool.size
        .u128()
        .checked_add(amount.u128())
        .ok_or_else(||
            StdError::generic_err("Pool size overflow detected.")
        )?
        .into();

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
            log("new_pool_size", pool.size)
        ],
        data: None
    })
}

/// Transfer back the specified `amount` of LP tokens to the sender.
/// Also works even if the pool is not eligible for rewards anymore.
pub(crate) fn retrieve_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = load_config(deps)?;
    let mut account = get_account_or_fail(deps, &env.message.sender)?;

    if env.block.time - account.last_claimed < config.claim_interval {
        return Err(StdError::generic_err(format!(
            "Can only retrieve tokens if hasn't claimed in the past {} seconds.",
            config.claim_interval
        )));
    }

    account.unlock_pending(env.block.time, config.claim_interval)?;
    account.subtract_balance(amount.u128())?;

    save_account(deps, &account)?;

    let mut pool = load_pool(&deps)?;
    pool.size = pool.size.u128().saturating_sub(amount.u128()).into();

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
    env: Env
) -> StdResult<HandleResponse> {
    let config = load_config(deps)?;
    let available_balance = get_balance(&deps.querier, &config)?;

    let pool = load_pool(deps)?;

    let mut account = get_account_or_fail(deps, &env.message.sender)?;
    account.unlock_pending(env.block.time, config.claim_interval)?;

    let reward_amount = calc_reward_share(
        account.locked_amount(),
        &pool,
        config.token_decimals
    )?;

    if reward_amount == 0 {
        return Err(StdError::generic_err("Reward amount is currently zero."));
    }

    let portions = calc_portions(
        account.last_claimed,
        config.claim_interval,
        env.block.time
    )?;

    if portions == 0 {
        return Err(StdError::generic_err(format!(
            "Need to wait {} more seconds before claiming.",
            config.claim_interval - (env.block.time - account.last_claimed)
        )));
    }

    account.last_claimed = env.block.time;
    save_account(deps, &account)?;

    // Claim the remaining rewards amount if the current rewards pool,
    // is less than what should be claimed.
    let claim_amount = if reward_amount > available_balance {
        available_balance
    } else {
        reward_amount
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
            log("reward_amount", reward_amount),
            log("claimed_amount", claim_amount)
        ],
        data: None
    })
}

#[require_admin]
pub(crate) fn change_pool_share<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_share: Uint128
) -> StdResult<HandleResponse> {
    let mut pool = load_pool(deps)?;
    pool.share = new_share;

    save_pool(deps, &pool)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "change_pool_share"),
            log("new_pool_share", new_share)
        ],
        data: None
    })
}

/// Dry runs the claim method providing a detailed result for each of
/// the provided LP token addresses. Records the actual would be
/// errors, if any, instead of terminating the function early.
fn claim_simulation<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
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
                    return Ok(to_binary(&QueryMsgResponse::ClaimSimulation(
                        ClaimSimulationResult::error(ClaimError::PoolEmpty)
                    ))?);
                }
            }
        }
    };

    let pool = load_pool(deps)?;

    let mut account = get_account_or_fail(deps, &sender)?;
    account.unlock_pending(current_time, config.claim_interval)?;

    if account.locked_amount() == 0 {
        return to_binary(&QueryMsgResponse::ClaimSimulation(
            ClaimSimulationResult::error(ClaimError::AccountZeroLocked))
        );
    }

    let reward_per_portion = calc_reward_share(
        account.locked_amount(),
        &pool,
        config.token_decimals
    )?;

    if reward_per_portion == 0 {
        return to_binary(&QueryMsgResponse::ClaimSimulation(
            ClaimSimulationResult::error(ClaimError::AccountZeroReward))
        );
    }

    let portions = calc_portions(
        account.last_claimed,
        config.claim_interval,
        current_time
    )?;

    if portions == 0 {
        return to_binary(&QueryMsgResponse::ClaimSimulation(
            ClaimSimulationResult::error(ClaimError::EarlyClaim {
                time_to_wait: config.claim_interval - (current_time - account.last_claimed)
            }))
        );
    }

    let reward_amount = reward_per_portion.saturating_mul(portions as u128);

    let claim_amount = if reward_amount > available_balance {
        available_balance
    } else {
        reward_amount
    };

    to_binary(&QueryMsgResponse::ClaimSimulation(
            ClaimSimulationResult::success(
                reward_amount.into(),
                reward_per_portion.into(),
                claim_amount.into()
            )
        )
    )
}

fn query_pool<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let pool = load_pool(deps)?;

    to_binary(&QueryMsgResponse::Pool(pool))
}

/// Authenticated command. Returns the account for the
/// provided user address.
fn query_account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
    key: ViewingKey
) -> StdResult<Binary> {
    let canonical = deps.api.canonical_address(&address)?;
    authenticate(&deps.storage, &key, canonical.as_slice())?;

    let result = get_account_or_fail(deps, &address)?;

    to_binary(&QueryMsgResponse::Account(result))
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
        StdError::generic_err(format!("Pool size is currently zero."))
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

/// Calculates how many portions should be transferred. The amount of portions
/// depends on when the `claim` method was last called. Ex. given a claim interval
/// of 1 day and user who hasn't claimed their rewards for 3 days, then they should
/// earn 3x of their reward share. So the resulting portions would be 3.
pub(crate) fn calc_portions(
    last_claimed: u64,
    claim_interval: u64,
    block_time: u64
) -> StdResult<u32> {
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
fn get_account_or_fail<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
) -> StdResult<Account<HumanAddr>> { 
    get_account(deps, address)?.ok_or_else(||
        StdError::generic_err(format!(
            "No account for {} currently exists.",
            address,
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
