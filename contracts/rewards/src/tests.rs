
use std::time::{SystemTime, UNIX_EPOCH};
use cosmwasm_std::{
    Api, Extern, HumanAddr, Querier, StdResult,
    Storage, Uint128, StdError, to_binary, from_binary
};
use cosmwasm_std::testing::{mock_env, MockStorage, MockApi};
use rand::{Rng, thread_rng};
use fadroma_scrt_callback::ContractInstance;
use cosmwasm_utils::convert::get_whole_token_representation;

use crate::contract::*;
use crate::msg::{
    HandleMsg, InitMsg, QueryMsg, RewardPoolConfig, QueryMsgResponse
};
use crate::state::{load_config, get_pools, get_inactive_pool};
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

#[test]
fn test_claim_with_lock_unlock() {
    let reward_token = ContractInstance {
        address: "reward_token".into(),
        code_hash: "reward_token_hash".into()
    };

    let lp_token_decimals = 6;
    let pool_addr = HumanAddr::from("pool");

    let share = get_whole_token_representation(lp_token_decimals) * 600;
    let pool = RewardPoolConfig {
        share: Uint128(share),
        lp_token: ContractInstance {
            address: pool_addr.clone(),
            code_hash: "pool_hash".into()
        }
    };

    let ref mut deps = mock_dependencies(20, reward_token.clone(), pool.share, 18);

    let claim_interval = 100;
    let msg = InitMsg {
        admin: None,
        reward_token,
        claim_interval,
        reward_pools: Some(vec![pool]),
        prng_seed: to_binary(&"whatever").unwrap(),
        entropy: to_binary(&"whatever").unwrap()
    };

    init(deps, mock_env("admin", &[]), msg).unwrap();

    let mut time = 0;

    let sender1 = HumanAddr::from("sender1");
    let sender2 = HumanAddr::from("sender2");
    let sender3 = HumanAddr::from("sender3");
    let sender4 = HumanAddr::from("sender4");

    let deposit_amount = share / 4;

    lock_tokens(
        deps,
        mock_env(sender1.clone(), &[]),
        Uint128(deposit_amount),
        pool_addr.clone()
    ).unwrap();

    lock_tokens(
        deps,
        mock_env(sender2.clone(), &[]),
        Uint128(deposit_amount),
        pool_addr.clone()
    ).unwrap();

    let (empty, amount) = execute_claim(deps, time, sender1.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 2);
    
    let (empty, amount) = execute_claim(deps, time, sender2.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 2);

    // User locks tokens and claims while the pool is still empty
    lock_tokens(
        deps,
        mock_env(sender3.clone(), &[]),
        Uint128(deposit_amount),
        pool_addr.clone()
    ).unwrap();

    let (empty, amount) = execute_claim(deps, time, sender3.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, true);
    assert_eq!(amount, 0);

    deps.querier.reward_token_supply += Uint128(share);
    time += claim_interval;

    lock_tokens(
        deps,
        mock_env(sender4.clone(), &[]),
        Uint128(deposit_amount),
        pool_addr.clone()
    ).unwrap();

    let (empty, amount) = execute_claim(deps, time, sender1.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    let (empty, amount) = execute_claim(deps, time, sender2.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    let (empty, amount) = execute_claim(deps, time, sender3.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    let (empty, amount) = execute_claim(deps, time, sender4.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, share / 4);

    assert_eq!(deps.querier.reward_token_supply.u128(), 0);

    deps.querier.reward_token_supply += Uint128(share);
    time += claim_interval;

    retrieve_tokens(
        deps,
        mock_env(sender1.clone(), &[]),
        Uint128(deposit_amount),
        pool_addr.clone()
    ).unwrap();

    let expected_amount = 198000000u128;

    let (empty, amount) = execute_claim(deps, time, sender2.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, expected_amount);

    let (empty, amount) = execute_claim(deps, time, sender3.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, expected_amount);

    let (empty, amount) = execute_claim(deps, time, sender4.clone(), pool_addr.clone()).unwrap();
    assert_eq!(empty, false);
    assert_eq!(amount, expected_amount);
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
