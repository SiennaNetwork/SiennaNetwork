
use std::time::{SystemTime, UNIX_EPOCH};
use cosmwasm_std::{
    Extern, HumanAddr, StdResult, Uint128,
    StdError, to_binary, from_binary
};
use cosmwasm_std::testing::{mock_env, MockStorage, MockApi};
use rand::{Rng, thread_rng};
use fadroma_scrt_callback::{ContractInstance, Callback};
use cosmwasm_utils::convert::get_whole_token_representation;

use crate::contract::*;
use crate::msg::{
    HandleMsg, InitMsg, QueryMsg, QueryMsgResponse, RewardPoolConfig
};
use crate::state::{load_config, load_pool};
use crate::data::RewardPool;
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

    let pool = RewardPoolConfig {
        share: Uint128(100),
        lp_token: ContractInstance {
            address: "pool".into(),
            code_hash: "pool_hash".into()
        }
    };

    let ref mut deps = mock_dependencies(20, reward_token.clone(), Uint128(1), decimals);

    let callback = create_callback();

    let msg = InitMsg {
        admin: None,
        reward_token: reward_token.clone(),
        claim_interval,
        pool: pool.clone(),
        prng_seed: prng_seed.clone(),
        entropy: to_binary(&"whatever").unwrap(),
        callback: callback.clone()
    };

    init(deps, mock_env("admin", &[]), msg).unwrap();

    let config = load_config(deps).unwrap();
    assert_eq!(config.reward_token, reward_token);
    assert_eq!(config.token_decimals, decimals);
    assert_eq!(config.claim_interval, claim_interval);
    assert_eq!(config.prng_seed, prng_seed);
    assert_eq!(config.factory_address, callback.contract.address);

    let stored_pool = load_pool(deps).unwrap();

    assert_pools_eq(&pool.into(), &stored_pool);

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
fn test_calc_reward_share() {
    let pool = create_pool(
        500_000_000_000_000_000_000,
        1000_000_000
    );

    // If owning 15% of pool share, then receive 15% of 500 = 75
    let result = calc_reward_share(150_000_000, pool.size.u128(), pool.share.u128(), 18).unwrap();
    assert_eq!(result, 75_000_000_000_000_000_000);

    // Absorb the entire pool if owning 100% of pool share.
    let result = calc_reward_share(1000_000_000, pool.size.u128(), pool.share.u128(), 18).unwrap();
    assert_eq!(result, 500_000_000_000_000_000_000);

    let result = calc_reward_share(0, pool.size.u128(), pool.share.u128(), 18).unwrap();
    assert_eq!(result, 0);
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

#[test]
fn test_claim_with_lock_unlock() {
    let reward_token = ContractInstance {
        address: "reward_token".into(),
        code_hash: "reward_token_hash".into()
    };

    let lp_token_decimals = 6;

    let share = get_whole_token_representation(lp_token_decimals) * 600;

    let pool = RewardPoolConfig {
        share: Uint128(share),
        lp_token: ContractInstance {
            address: "pool".into(),
            code_hash: "pool_hash".into()
        }
    };

    let ref mut deps = mock_dependencies(20, reward_token.clone(), pool.share, 18);

    let claim_interval = 100;
    let msg = InitMsg {
        admin: None,
        reward_token,
        claim_interval,
        pool,
        prng_seed: to_binary(&"whatever").unwrap(),
        entropy: to_binary(&"whatever").unwrap(),
        callback: create_callback()
    };

    init(deps, mock_env("admin", &[]), msg).unwrap();

    let mut time = claim_interval;

    let sender1 = HumanAddr::from("sender1");
    let sender2 = HumanAddr::from("sender2");
    let sender3 = HumanAddr::from("sender3");
    let sender4 = HumanAddr::from("sender4");

    let deposit_amount = share / 4;

    lock_tokens(
        deps,
        mock_env_with_time(sender1.clone(), time),
        Uint128(deposit_amount)
    ).unwrap();

    lock_tokens(
        deps,
        mock_env_with_time(sender2.clone(), time),
        Uint128(deposit_amount),
    ).unwrap();

    time += claim_interval;

    let (empty, amount) = execute_claim(deps, time, sender1.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 2);
    
    let (empty, amount) = execute_claim(deps, time, sender2.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 2);

    // User locks tokens and claims while the pool is still empty
    lock_tokens(
        deps,
        mock_env_with_time(sender3.clone(), time),
        Uint128(deposit_amount)
    ).unwrap();

    let (empty, amount) = execute_claim(deps, time, sender3.clone()).unwrap();
    assert_eq!(empty, true);
    assert_eq!(amount, 0);

    lock_tokens(
        deps,
        mock_env_with_time(sender4.clone(), time),
        Uint128(deposit_amount)
    ).unwrap();

    deps.querier.reward_token_supply += Uint128(share);
    time += claim_interval;

    let (empty, amount) = execute_claim(deps, time, sender1.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    let (empty, amount) = execute_claim(deps, time, sender2.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    let (empty, amount) = execute_claim(deps, time, sender3.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    let (empty, amount) = execute_claim(deps, time, sender4.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    assert_eq!(deps.querier.reward_token_supply.u128(), 0);

    deps.querier.reward_token_supply += Uint128(share);
    time += claim_interval;

    retrieve_tokens(
        deps,
        mock_env(sender1.clone(), &[]),
        Uint128(deposit_amount)
    ).unwrap();

    let expected_amount = 198000000u128;

    let (empty, amount) = execute_claim(deps, time, sender2.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, expected_amount);

    let (empty, amount) = execute_claim(deps, time, sender3.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, expected_amount);

    let (empty, amount) = execute_claim(deps, time, sender4.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, expected_amount);
}

#[test]
fn test_cant_claim_twice_by_retrieving_tokens() {
    let reward_token = ContractInstance {
        address: "reward_token".into(),
        code_hash: "reward_token_hash".into()
    };

    let lp_token_addr = HumanAddr("lp_token_hash".into());
    let share = 500u128;

    let ref mut deps = mock_dependencies(20, reward_token.clone(), Uint128(share * 2), 18);

    let claim_interval = 100;

    init(deps, mock_env("admin", &[]), InitMsg {
        claim_interval,
        admin: None,
        reward_token,
        pool: RewardPoolConfig {
            share: Uint128(share),
            lp_token: ContractInstance {
                address: lp_token_addr.clone(),
                code_hash: "lp_token_hash".into()
            }
        },
        prng_seed: to_binary(&"whatever").unwrap(),
        entropy: to_binary(&"whatever").unwrap(),
        callback: create_callback()
    }).unwrap();

    let user = HumanAddr("user".into());
    let lp_amount = Uint128(100);

    let mut time = claim_interval;

    handle(deps, mock_env_with_time(user.clone(), time), HandleMsg::LockTokens {
        amount: lp_amount,
    }).unwrap();

    time += claim_interval;

    let (_, amount) = execute_claim(deps, time, user.clone()).unwrap();
    assert_eq!(amount, share);
    assert_eq!(deps.querier.reward_token_supply, Uint128(share));

    handle(deps, mock_env(user.clone(), &[]), HandleMsg::RetrieveTokens {
        amount: lp_amount,
    }).unwrap();

    handle(deps, mock_env_with_time(user.clone(), time), HandleMsg::LockTokens {
        amount: lp_amount,
    }).unwrap();

    time += claim_interval / 2;

    let err = claim(
        deps,
        mock_env_with_time(user.clone(), time)
    ).unwrap_err();

    match err {
        StdError::GenericErr { msg, .. } => {
            if !(msg == "Reward amount is currently zero.") {
                panic!("Expecting reward amount to be 0.")
            }
        },
        _ => panic!("Expecting StdError::GenericErr")
    }
}

#[test]
fn test_cant_claim_instantly() {
    let reward_token = ContractInstance {
        address: "reward_token".into(),
        code_hash: "reward_token_hash".into()
    };

    let lp_token_addr = HumanAddr("lp_token_hash".into());
    let share = 500u128;

    let ref mut deps = mock_dependencies(20, reward_token.clone(), Uint128(share * 2), 18);

    init(deps, mock_env("admin", &[]), InitMsg {
        claim_interval: 100,
        admin: None,
        reward_token,
        pool: RewardPoolConfig {
            share: Uint128(share),
            lp_token: ContractInstance {
                address: lp_token_addr.clone(),
                code_hash: "lp_token_hash".into()
            }
        },
        prng_seed: to_binary(&"whatever").unwrap(),
        entropy: to_binary(&"whatever").unwrap(),
        callback: create_callback()
    }).unwrap();

    let user = HumanAddr("user".into());
    let lp_amount = Uint128(100);

    let mut time = 100;

    handle(deps, mock_env_with_time(user.clone(), time), HandleMsg::LockTokens {
        amount: lp_amount
    }).unwrap();

    time += 50;

    let err = claim(
        deps,
        mock_env_with_time(user.clone(), time)
    ).unwrap_err();

    match err {
        StdError::GenericErr { msg, .. } => {
            if !(msg == "Reward amount is currently zero.") {
                panic!("Expecting reward amount to be 0.")
            }
        },
        _ => panic!("Expecting StdError::GenericErr")
    }
}

#[test]
fn test_cant_retrieve_soon_after_claim() {
    let reward_token = ContractInstance {
        address: "reward_token".into(),
        code_hash: "reward_token_hash".into()
    };

    let lp_token_addr = HumanAddr("lp_token_hash".into());
    let share = 600u128;

    let ref mut deps = mock_dependencies(20, reward_token.clone(), Uint128(share * 2), 18);
    let claim_interval = 100;

    init(deps, mock_env("admin", &[]), InitMsg {
        claim_interval,
        admin: None,
        reward_token,
        pool: RewardPoolConfig {
            share: Uint128(share),
            lp_token: ContractInstance {
                address: lp_token_addr.clone(),
                code_hash: "lp_token_hash".into()
            }
        },
        prng_seed: to_binary(&"whatever").unwrap(),
        entropy: to_binary(&"whatever").unwrap(),
        callback: create_callback()
    }).unwrap();

    let lp_amount = Uint128(300);

    let num_users = 3;
    let mut users = Vec::with_capacity(num_users);

    let mut time = claim_interval;

    for i in 0..num_users {
        let user = HumanAddr::from(format!("user_{}", i));
        users.push(user.clone());

        handle(deps, mock_env_with_time(user, time), HandleMsg::LockTokens {
            amount: lp_amount
        }).unwrap();
    }

    time += claim_interval;

    let (empty, amount) = execute_claim(deps, time, users[0].clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, 198);

    let err = retrieve_tokens(
        deps,
        mock_env_with_time(users[0].clone(), time),
        lp_amount
    ).unwrap_err();

    match err {
        StdError::GenericErr { msg, .. } => {
            if !(msg == format!("Can only retrieve tokens if hasn't claimed in the past {} seconds.", claim_interval)) {
                panic!("Expecting reward amount to be 0.")
            }
        },
        _ => panic!("Expecting StdError::GenericErr")
    }

    for i in 1..users.len() {
        let (empty, amount) = execute_claim(deps, time, users[i].clone()).unwrap();
        assert_eq!(empty, false);
        assert_eq!(amount, 198);
    }

    time += claim_interval;

    retrieve_tokens(
        deps,
        mock_env_with_time(users[0].clone(), time),
        lp_amount
    ).unwrap();

    let err = claim(
        deps,
        mock_env_with_time(users[0].clone(), time)
    ).unwrap_err();

    match err {
        StdError::GenericErr { msg, .. } => {
            if !(msg == "Reward amount is currently zero.") {
                panic!("Expecting reward amount to be 0.")
            }
        },
        _ => panic!("Expecting StdError::GenericErr")
    }

    for i in 1..users.len() {
        let (empty, amount) = execute_claim(deps, time, users[i].clone()).unwrap();
        assert_eq!(empty, false);
        assert_eq!(amount, 300);
    }

    assert_eq!(deps.querier.reward_token_supply, Uint128(6));
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

fn assert_pools_eq(lhs: &RewardPool<HumanAddr>, rhs: &RewardPool<HumanAddr>) {
    assert_eq!(lhs.lp_token, rhs.lp_token);
    assert_eq!(lhs.share, rhs.share);
    assert_eq!(lhs.size, rhs.size);
}

fn execute_claim(
    deps: &mut Extern<MockStorage, MockApi, MockSnip20Querier>,
    time: u64,
    user: HumanAddr
) -> StdResult<(bool, u128)> {
    let result = claim(
        deps,
        mock_env_with_time(user.clone(), time)
    );

    if result.is_err() { 
        let err = result.unwrap_err();

        match &err {
            StdError::GenericErr { msg, .. } => {
                if msg == "The reward token pool is currently empty." {
                    return Ok((true, 0));
                } else if msg == "Reward amount is currently zero." {
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
    let mut rng = thread_rng();

    let iterations = rng.gen_range(5..20);
    let claim_interval = 86400; // 1 day
    let num_users = rng.gen_range(5..20);

    let reward_token_decimals = 18;
    let reward_token = ContractInstance {
        address: "reward_token".into(),
        code_hash: "reward_token_hash".into()
    };

    let pool = RewardPoolConfig {
        lp_token: ContractInstance {
            address: "lp_token".into(),
            code_hash: "lp_token_hash".into()
        },
        share: Uint128(rng.gen_range(100_000_000_000_000_000_000..800_000_000_000_000_000_000))
    };

    let reward_token_supply = pool.share.u128() * iterations;

    let ref mut deps = mock_dependencies(
        20,
        reward_token.clone(),
        Uint128(reward_token_supply),
        reward_token_decimals
    );

    init(deps, mock_env("admin", &[]), InitMsg {
        reward_token,
        admin: None,
        pool,
        claim_interval,
        prng_seed: to_binary(&"whatever").unwrap(),
        entropy: to_binary(&"whatever").unwrap(),
        callback: create_callback()
    }).unwrap();

    let mut users = Vec::with_capacity(num_users);

    for i in 0..num_users {
        let user = HumanAddr(format!("User {}", i + 1));
        users.push(user.clone());

        lock_tokens(
            deps,
            mock_env(user.clone(), &[]),
            Uint128(rng.gen_range(10_000_000..100_000_000))
        ).unwrap();
    }

    let now = SystemTime::now();
    let mut time = now.duration_since(UNIX_EPOCH).unwrap().as_secs();

    let mut total_claimed = 0;
    let mut is_done = false;

    while !is_done {
        for _ in 0..iterations {
            for user in users.clone().into_iter() {
                let rand = rng.gen_range(0..20);

                // Skip claiming for some users to simulate them
                // not getting their rewards every time they can
                if rand % 2 == 0 {
                    let (depleted, claimed) = execute_claim(
                        deps,
                        time,
                        user.clone()
                    ).unwrap();

                    total_claimed += claimed;
                    is_done = depleted;
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
                user
            ).unwrap();

            total_claimed += claimed;
            is_done = depleted;
        }

        // Ensure that the claim interval has passed
        time += claim_interval;
    }

    assert_eq!(total_claimed, reward_token_supply);
}

fn create_callback() -> Callback<HumanAddr> {
    Callback {
        contract: ContractInstance {
            address: "dummy_addr".into(),
            code_hash: "dummy_code_hash".into(),
        },
        msg: to_binary(&"dummy_msg").unwrap()
    }
}
