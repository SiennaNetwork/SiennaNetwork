use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdResult, Storage, Uint128, StdError, log, to_binary
};
use composable_admin::require_admin;
use composable_admin::admin::{
    save_admin, admin_handle, admin_query, DefaultHandleImpl as DefaultAdminHandle,
    DefaultQueryImpl, assert_admin
};
use composable_auth::{auth_handle, authenticate};
use secret_toolkit::snip20;
use fadroma_scrt_callback::ContractInstance;
use cosmwasm_utils::viewing_key::ViewingKey;
use cosmwasm_utils::convert::{convert_token, get_whole_token_representation};

use crate::msg::{
    HandleMsg, InitMsg, QueryMsg, OVERFLOW_MSG, RewardPoolConfig,
    QueryMsgResponse, ClaimSimulationResult, ClaimResult, ClaimError
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
        HandleMsg::Auth(auth_msg) => auth_handle(deps, env, auth_msg, AuthImpl)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::ClaimSimulation { lp_tokens, viewing_key, address, current_time } => 
            claim_simulation(deps, lp_tokens, address, ViewingKey(viewing_key), current_time),
        QueryMsg::Pools => query_pools(deps),
        QueryMsg::Accounts { address, viewing_key, lp_tokens } =>
            query_accounts(deps, address, ViewingKey(viewing_key), lp_tokens),
        QueryMsg::TokenInfo { } => query_token_info(),
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

fn retrieve_tokens<S: Storage, A: Api, Q: Querier>(
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

fn claim<S: Storage, A: Api, Q: Querier>(
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
    let available_balance = get_balance(&deps.querier, &config)?;
    
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

#[require_admin]
fn change_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pools: Vec<RewardPool<HumanAddr>>,
    total_share: Uint128
) -> StdResult<HandleResponse>{
    let mut sum_total = 0u128;

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

fn query_pools<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let pools = get_pools(deps)?;

    Ok(to_binary(&QueryMsgResponse::Pools(pools))?)
}

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

fn query_token_info() -> StdResult<Binary> {
    Ok(to_binary(&QueryMsgResponse::TokenInfo {
        name: "Sienna Rewards".into(),
        symbol: "SRW".into(),
        decimals: 1,
        total_supply: None
    })?)
}

fn calc_reward_share(
    mut user_locked: u128,
    pool: &RewardPool<HumanAddr>,
    reward_token_decimals: u8
) -> StdResult<u128> {
    user_locked *= 100; // Multiply by 100 to get a non float percentage

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

#[inline]
fn into_pools(mut vec: Vec<RewardPoolConfig>) -> Vec<RewardPool<HumanAddr>> {
    vec.drain(..).map(|p| p.into()).collect()
}

fn calc_portions(
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

fn get_balance(querier: &impl Querier, config: &Config<HumanAddr>) -> StdResult<u128> {
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
        return Err(StdError::generic_err(
            "The reward token pool is currently empty."
        ));
    }

    Ok(available_balance)
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::time::{SystemTime, UNIX_EPOCH};
    use cosmwasm_std::{to_binary, from_binary};
    use cosmwasm_std::testing::{mock_env, MockStorage, MockApi};
    use rand::{Rng, thread_rng};

    use crate::test_helpers::{
        mock_dependencies, mock_env_with_time, MockSnip20Querier
    };

    #[test]
    fn test_init() {
        let reward_token = ContractInstance {
            address: "reward_token".into(),
            code_hash: "reward_token_hash".into()
        };

        let decimals = 18;
        let claim_interval = 100;
        let prng_seed = to_binary(&"whatever").unwrap();

        let reward_pools = vec![
            RewardPoolConfig {
                share: Uint128(100),
                lp_token: ContractInstance {
                    address: "pool1".into(),
                    code_hash: "pool1_hash".into()
                }
            },
            RewardPoolConfig {
                share: Uint128(200),
                lp_token: ContractInstance {
                    address: "pool2".into(),
                    code_hash: "pool2_hash".into()
                }
            }
        ];

        let ref mut deps = mock_dependencies(20, reward_token.clone(), Uint128(1), decimals);

        let msg = InitMsg {
            admin: None,
            reward_token: reward_token.clone(),
            claim_interval,
            reward_pools: Some(reward_pools.clone()),
            prng_seed: prng_seed.clone(),
            entropy: to_binary(&"whatever").unwrap()
        };

        init(deps, mock_env("admin", &[]), msg).unwrap();

        let config = load_config(deps).unwrap();
        assert_eq!(config.reward_token, reward_token);
        assert_eq!(config.token_decimals, decimals);
        assert_eq!(config.claim_interval, claim_interval);
        assert_eq!(config.prng_seed, prng_seed);

        let pools = into_pools(reward_pools);
        let stored_pools = get_pools(deps).unwrap();

        assert_pools_eq(&pools, &stored_pools);

        let result = query(deps, QueryMsg::TokenInfo { }).unwrap();
        let response: QueryMsgResponse = from_binary(&result).unwrap();

        match response {
            QueryMsgResponse::TokenInfo { name, symbol, decimals, total_supply } => {
                assert_eq!(name, "Sienna Rewards");
                assert_eq!(symbol, "SRW");
                assert_eq!(decimals, 1);
                assert_eq!(total_supply, None);
            },
            _ => panic!("Expected QueryMsgResponse::TokenInfo")
        }
    }

    #[test]
    fn test_change_pools() {
        fn assert_pools_inactive<S: Storage, A: Api, Q: Querier>(
            deps: &Extern<S, A, Q>,
            pools: &Vec<RewardPool<HumanAddr>>
        ) {
            for pool in pools {
                assert_eq!(
                    get_inactive_pool(deps, &pool.lp_token.address).unwrap().unwrap(),
                    *pool
                );
            }
        }

        let reward_token = ContractInstance {
            address: "reward_token".into(),
            code_hash: "reward_token_hash".into()
        };

        let initial_pools = vec![
            RewardPoolConfig {
                share: Uint128(100),
                lp_token: ContractInstance {
                    address: "pool1".into(),
                    code_hash: "pool1_hash".into()
                }
            },
            RewardPoolConfig {
                share: Uint128(200),
                lp_token: ContractInstance {
                    address: "pool2".into(),
                    code_hash: "pool2_hash".into()
                }
            }
        ];

        let ref mut deps = mock_dependencies(20, reward_token.clone(), Uint128(1), 18);

        let msg = InitMsg {
            admin: None,
            reward_token,
            claim_interval: 100,
            reward_pools: Some(initial_pools.clone()),
            prng_seed: to_binary(&"whatever").unwrap(),
            entropy: to_binary(&"whatever").unwrap()
        };

        init(deps, mock_env("admin", &[]), msg).unwrap();

        let err = handle(deps, mock_env("unauthorized", &[]), HandleMsg::ChangePools {
            pools: vec![],
            total_share: Uint128(100)
        }).unwrap_err();

        assert_eq!(err, StdError::unauthorized());

        let third_pool = RewardPoolConfig {
            share: Uint128(300),
            lp_token: ContractInstance {
                address: "pool3".into(),
                code_hash: "pool3_hash".into()
            }
        };

        let new_pools = vec![ 
            initial_pools[0].clone(),
            initial_pools[1].clone(),
            third_pool
        ];

        let err = handle(deps, mock_env("admin", &[]), HandleMsg::ChangePools {
            pools: new_pools.clone(),
            total_share: Uint128(599)
        }).unwrap_err();

        match err {
            StdError::GenericErr { msg, .. } => assert!(msg.starts_with("Total pool share(")),
            _ => panic!("Expected StdError::GenericErr, got: {}", err)
        }
        
        handle(deps, mock_env("admin", &[]), HandleMsg::ChangePools {
            pools: new_pools,
            total_share: Uint128(600)
        }).unwrap();

        let second_pools = vec![ 
            RewardPoolConfig {
                share: Uint128(300),
                lp_token: ContractInstance {
                    address: "pool3".into(),
                    code_hash: "pool3_hash".into()
                }
            },
            RewardPoolConfig {
                share: Uint128(400),
                lp_token: ContractInstance {
                    address: "pool4".into(),
                    code_hash: "pool4_hash".into()
                }
            }
        ];

        handle(deps, mock_env("admin", &[]), HandleMsg::ChangePools {
            pools: second_pools.clone(),
            total_share: Uint128(700)
        }).unwrap();

        assert_pools_eq(&get_pools(deps).unwrap(), &into_pools(second_pools.clone()));
        assert_pools_inactive(deps, &into_pools(initial_pools.clone()));

        let locked_tokens = Uint128(9999);

        lock_tokens(
            deps,
            mock_env("user", &[]),
            locked_tokens,
            second_pools[0].lp_token.address.clone()
        ).unwrap();

        let third_pools = vec![
            initial_pools[0].clone(),
            initial_pools[1].clone(),
            second_pools[0].clone()
        ];

        handle(deps, mock_env("admin", &[]), HandleMsg::ChangePools {
            pools: third_pools.clone(),
            total_share: Uint128(600)
        }).unwrap();

        let mut third_pools = into_pools(third_pools);
        third_pools[2].size = locked_tokens;

        assert_pools_eq(&get_pools(deps).unwrap(), &third_pools);
        assert_pools_inactive(deps, &into_pools([ second_pools[1].clone() ].to_vec()));
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

    #[test]
    fn test_claim() {
        let runs = 100;

        for _ in 0..runs {
            claim_run();
        }
    }

    #[test]
    fn test_calc_portions() {
        let now = SystemTime::now();
        let time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();
        let claim_interval = 86400; // 1 day

        assert_eq!(0, calc_portions(
            time - claim_interval + 1,
            claim_interval,
            time
        ).unwrap());

        assert_eq!(1, calc_portions(
            time - claim_interval - 1000,
            claim_interval,
            time
        ).unwrap());

        assert_eq!(2, calc_portions(
            time - claim_interval * 2,
            claim_interval,
            time
        ).unwrap());

        assert_eq!(1, calc_portions(
            (time - claim_interval * 2) + 1,
            claim_interval,
            time
        ).unwrap());

        assert_eq!(10, calc_portions(
            time - claim_interval * 10,
            claim_interval,
            time
        ).unwrap());
    }


    fn create_pool(share: u128, size: u128) -> RewardPool<HumanAddr> {
        RewardPool {
            lp_token: ContractInstance {
                address: HumanAddr::from("lp_token"),
                code_hash: "lp_token".into()
            },
            share: Uint128(share),
            size: Uint128(size)
        }
    }

    fn assert_pools_eq(lhs: &Vec<RewardPool<HumanAddr>>, rhs: &Vec<RewardPool<HumanAddr>>) {
        assert_eq!(lhs.len(), rhs.len());

        for (i, pool) in lhs.iter().enumerate() {
            let other = &rhs[i];

            assert_eq!(pool.lp_token, other.lp_token);
            assert_eq!(pool.share, other.share);
            assert_eq!(pool.size, other.size);
        }
    }

    fn execute_claim(
        deps: &mut Extern<MockStorage, MockApi, MockSnip20Querier>,
        time: u64,
        user: HumanAddr,
        lp_token: HumanAddr
    ) -> StdResult<(bool, u128)> {
        let result = claim(
            deps,
            mock_env_with_time(user.clone(), time),
            vec![ lp_token ]
        );

        if result.is_err() { 
            let err = result.unwrap_err();

            match &err {
                StdError::GenericErr { msg, .. } => {
                    if msg == "The reward token pool is currently empty." {
                        return Ok((true, 0));
                    } else if msg.starts_with("Reward amount for") {
                        // It is possible for the user share to be so small that
                        // the reward amount calculated would be zero. So this 
                        // this shouldn't be counted as a program error.
                        return Ok((false, 0));
                    }

                    return Err(err);
                },
                _ => return Err(err)
            }
        }

        let result = result.unwrap();

        let claimed = result.log.iter().find(|e|
            e.key == "claimed_amount"
        ).unwrap();

        let value = claimed.value.parse::<u128>().unwrap();

        // Subtract to simulate SNIP20 transfer message
        let supply = deps.querier.reward_token_supply.u128();
        deps.querier.reward_token_supply = Uint128(supply - value);

        Ok((false, value))
    }

    fn claim_run() {
        const NUM_POOLS: usize = 3;
        
        fn pools_to_call(index: usize) -> usize {
            if index % NUM_POOLS == 0 {
                NUM_POOLS
            } else {
                1
            }
        }

        let mut rng = thread_rng();

        let iterations = rng.gen_range(5..20);
        let claim_interval = 86400; // 1 day
        let num_users = rng.gen_range(5..20);

        let reward_token_decimals = 18;
        let reward_token = ContractInstance {
            address: "reward_token".into(),
            code_hash: "reward_token_hash".into()
        };

        let mut pools = Vec::with_capacity(NUM_POOLS);

        for i in 0..NUM_POOLS {
            pools.push(RewardPoolConfig {
                lp_token: ContractInstance {
                    address: HumanAddr(format!("lp_token_{}", i)),
                    code_hash: format!("lp_token_hash_{}", i)
                },
                share: Uint128(rng.gen_range(100_000_000_000_000_000_000..800_000_000_000_000_000_000))
            });
        }

        let total_share: u128 = pools.iter().map(|p| p.share.u128()).sum();
        let reward_token_supply = total_share * iterations;

        let ref mut deps = mock_dependencies(
            20,
            reward_token.clone(),
            Uint128(reward_token_supply),
            reward_token_decimals
        );

        init(deps, mock_env("admin", &[]), InitMsg {
            reward_token,
            admin: None,
            reward_pools: Some(pools.clone()),
            claim_interval,
            prng_seed: to_binary(&"whatever").unwrap(),
            entropy: to_binary(&"whatever").unwrap()
        }).unwrap();

        let pools = into_pools(pools);
        let mut users = Vec::with_capacity(num_users);

        for i in 0..num_users {
            let user = HumanAddr(format!("User {}", i + 1));
            users.push(user.clone());

            let pools_to_lock = pools_to_call(i);

            for p in 0..pools_to_lock{
                lock_tokens(
                    deps,
                    mock_env(user.clone(), &[]),
                    Uint128(rng.gen_range(10_000_000..100_000_000)),
                    pools[p].lp_token.address.clone()
                ).unwrap();
            }
        }

        let now = SystemTime::now();
        let mut time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

        let mut total_claimed = 0;
        let mut is_done = false;

        while !is_done {
            for _ in 0..iterations {
                for (i, user) in users.clone().into_iter().enumerate() {
                    let rand = rng.gen_range(0..20);

                    // Skip claiming for some users to simulate them
                    // not getting their rewards every time they can
                    if rand % 2 == 0 {
                        let pools_to_claim = pools_to_call(i);

                        for p in 0..pools_to_claim {
                            let (depleted, claimed) = execute_claim(
                                deps,
                                time,
                                user.clone(),
                                pools[p].lp_token.address.clone()
                            ).unwrap();
    
                            total_claimed += claimed;
                            is_done = depleted;
                        }
                    }

                    // Let a tiny amount of time to pass to introduce some entropy
                    time += rand;
                }
                
                // Ensure that the claim interval has passed
                time += claim_interval;
            }
        }

        // Final run to claim any remaining rewards
        while !is_done {
            for user in users.clone() {
                let (depleted, claimed) = execute_claim(
                    deps,
                    time,
                    user,
                    pools[0].lp_token.address.clone()
                ).unwrap();

                total_claimed += claimed;
                is_done = depleted;
            }

            // Ensure that the claim interval has passed
            time += claim_interval;
        }

        assert_eq!(total_claimed, reward_token_supply);
    }
}
