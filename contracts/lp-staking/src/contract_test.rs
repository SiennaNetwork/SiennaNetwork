#![cfg(test)]

use crate::{contract::*, msg::*, state::*, constants::*};
use scrt_finance::types::{TokenInfo, RewardPool, UserInfo};
use cosmwasm_std::{
    Coin,
    HumanAddr, Uint128,
    StdResult, StdError,
    Binary, to_binary, from_binary,
    Env, Storage, Api, Querier, Extern,
    BlockInfo, MessageInfo, ContractInfo,
    WasmMsg, CosmosMsg, InitResponse, HandleResponse,
    testing::{mock_dependencies, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR}
};
use scrt_finance::ContractInfo as ContractInfoWithHash;
use rand::Rng;
use serde::{Deserialize, Serialize};
use secret_toolkit::storage::TypedStore;

// Helper functions

fn init_helper(
    _deadline: u64,
) -> (
    StdResult<InitResponse>,
    Extern<MockStorage, MockApi, MockQuerier>,
) {
    let mut deps = mock_dependencies(20, &[]);
    let env = mock_env("admin", &[], 1);

    let init_msg = LPStakingInitMsg {
        deadline: 0u64,
        reward_token: ContractInfoWithHash {
            address: HumanAddr("scrt".to_string()),
            code_hash: "1".to_string(),
        },
        inc_token: ContractInfoWithHash {
            address: HumanAddr("eth".to_string()),
            code_hash: "2".to_string(),
        },
        prng_seed: Binary::from("lolz fun yay".as_bytes()),
        viewing_key: "123".to_string(),
        master: ContractInfoWithHash {
            address: Default::default(),
            code_hash: "".to_string(),
        },
        token_info: TokenInfo {
            name: "".to_string(),
            symbol: "".to_string(),
        },
    };

    (init(&mut deps, env, init_msg), deps)
}

/// Just set sender and sent funds for the message. The rest uses defaults.
/// The sender will be canonicalized internally to allow developers pasing in human readable senders.
/// This is intended for use in test code only.
pub fn mock_env<U: Into<HumanAddr>>(sender: U, sent: &[Coin], height: u64) -> Env {
    Env {
        block: BlockInfo {
            height,
            time: 1_571_797_419,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            sender: sender.into(),
            sent_funds: sent.to_vec(),
        },
        contract: ContractInfo {
            address: HumanAddr::from(MOCK_CONTRACT_ADDR),
        },
        contract_key: Some("".to_string()),
        contract_code_hash: "".to_string(),
    }
}

fn msg_from_action<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    action: &str,
    user: HumanAddr,
) -> (LPStakingHandleMsg, String) {
    let mut rng = rand::thread_rng();
    let chance = rng.gen_range(0, 100000);

    match action {
        "deposit" => {
            let amount: u128 = rng.gen_range(10e12 as u128, 1000e18 as u128);

            let msg = LPStakingHandleMsg::Receive {
                sender: user.clone(),
                from: user,
                amount: Uint128(amount),
                msg: to_binary(&LPStakingReceiveMsg::Deposit {}).unwrap(),
            };

            (msg, "eth".to_string())
        }
        "redeem" => {
            let amount: u128 = rng.gen_range(1e12 as u128, 1000e18 as u128);

            let msg = LPStakingHandleMsg::Redeem {
                amount: Some(Uint128(amount)),
            };

            (msg, user.0)
        }
        "deadline" if chance == 42 => {
            let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();
            let current = config.deadline as f64;

            let new = rng.gen_range(current + 1.0, current * 1.001);

            let msg = LPStakingHandleMsg::SetDeadline { block: new as u64 };

            (msg, "admin".to_string())
        }
        "rewards" if chance == 7 => {
            let amount: u128 = rng.gen_range(10000e6 as u128, 100000e6 as u128);

            let msg = LPStakingHandleMsg::Receive {
                sender: user.clone(),
                from: user,
                amount: Uint128(amount),
                msg: to_binary(&LPStakingReceiveMsg::DepositRewards {}).unwrap(),
            };

            (msg, "scrt".to_string())
        }
        _ => (
            LPStakingHandleMsg::Redeem {
                amount: Some(Uint128(u128::MAX)), // This will never work but will keep the tests going
            },
            "".to_string(),
        ),
    }
}

fn print_status(
    deps: &Extern<MockStorage, MockApi, MockQuerier>,
    users: Vec<HumanAddr>,
    new_mint: u128,
) {
    let reward_pool = TypedStore::<RewardPool, MockStorage>::attach(&deps.storage)
        .load(REWARD_POOL_KEY)
        .unwrap();
    let config: Config = TypedStore::attach(&deps.storage).load(CONFIG_KEY).unwrap();

    //println!("####### Statistics for block: {} #######", block);
    println!("Deadline: {}", config.deadline);
    println!("Locked ETH: {}", reward_pool.inc_token_supply);
    println!("Pending rewards: {}", reward_pool.pending_rewards);
    println!(
        "Accumulated rewards per share: {}",
        reward_pool.acc_reward_per_share
    );
    println!("Last reward block: {}", reward_pool.last_reward_block);

    for user in users {
        println!("## {}:", user.0);
        let user_info = TypedStore::<UserInfo, MockStorage>::attach(&deps.storage)
            .load(user.0.as_bytes())
            .unwrap_or(UserInfo { locked: 0, debt: 0 });
        let rewards = query_rewards(deps, user.clone(), Uint128(new_mint));

        println!("Locked: {}", user_info.locked);
        println!("Debt: {}", user_info.debt);
        println!("Reward: {}", rewards);
    }

    println!();
}

fn query_rewards(
    deps: &Extern<MockStorage, MockApi, MockQuerier>,
    user: HumanAddr,
    new_mint: Uint128,
) -> u128 {
    let query_msg = LPStakingQueryMsg::Rewards {
        address: user,
        new_rewards: new_mint,
        key: "42".to_string(),
        height: 0
    };

    let result: LPStakingQueryAnswer = from_binary(&query(&deps, query_msg).unwrap()).unwrap();
    match result {
        LPStakingQueryAnswer::Rewards { rewards } => rewards.u128(),
        _ => panic!("NOPE"),
    }
}

fn set_vks(deps: &mut Extern<MockStorage, MockApi, MockQuerier>, users: Vec<HumanAddr>) {
    for user in users {
        let vk_msg = LPStakingHandleMsg::SetViewingKey {
            key: "42".to_string(),
            padding: None,
        };
        handle(deps, mock_env(user.0, &[], 2001), vk_msg).unwrap();
    }
}

fn extract_rewards(result: StdResult<HandleResponse>) -> u128 {
    match result {
        Ok(resp) => {
            for message in resp.messages {
                match message {
                    CosmosMsg::Wasm(w) => match w {
                        WasmMsg::Execute {
                            contract_addr, msg, ..
                        } => {
                            if contract_addr == HumanAddr("scrt".to_string()) {
                                let transfer_msg: Snip20HandleMsg = from_binary(&msg).unwrap();

                                match transfer_msg {
                                    Snip20HandleMsg::Transfer { amount, .. } => {
                                        return amount.u128();
                                    }
                                }
                            }
                        }
                        _ => panic!(),
                    },
                    _ => panic!(),
                }
            }
        }
        Err(e) => match e {
            StdError::NotFound { .. } => {}
            StdError::GenericErr { msg, .. } => {
                if !msg.contains("insufficient") {
                    panic!("Wrong error message")
                }
            }
            _ => panic!("Unexpected error"),
        },
    }

    0
}

fn extract_reward_deposit(msg: LPStakingHandleMsg) -> u128 {
    match msg {
        LPStakingHandleMsg::Receive { amount, msg, .. } => {
            let transfer_msg: LPStakingReceiveMsg = from_binary(&msg).unwrap();

            match transfer_msg {
                LPStakingReceiveMsg::DepositRewards {} => amount.u128(),
                _ => 0,
            }
        }
        _ => 0,
    }
}

fn sanity_run(mut rewards: u128, mut deadline: u64) {
    let mut rng = rand::thread_rng();

    let (_init_result, mut deps) = init_helper(deadline);

    // TODO: deposit_rewards(&mut deps, mock_env("scrt", &[], 1), rewards).unwrap();

    let actions = vec!["deposit", "redeem", "deadline", "rewards"];
    let users = vec![
        HumanAddr("Lebron James".to_string()),
        HumanAddr("Kobe Bryant".to_string()),
        HumanAddr("Giannis".to_string()),
        HumanAddr("Steph Curry".to_string()),
        HumanAddr("Deni Avdija".to_string()),
    ];

    let mut total_rewards_output = 0;

    set_vks(&mut deps, users.clone());
    let mut block: u64 = 2;
    while block < (deadline + 10_000) {
        let num_of_actions = rng.gen_range(0, 5);

        for _ in 0..num_of_actions {
            let action_idx = rng.gen_range(0, actions.len());

            let user_idx = rng.gen_range(0, users.len());
            let user = users[user_idx].clone();

            let (msg, sender) = msg_from_action(&deps, actions[action_idx], user.clone());
            rewards += extract_reward_deposit(msg.clone());
            let result = handle(&mut deps, mock_env(sender, &[], block), msg);
            total_rewards_output += extract_rewards(result);
        }

        if block % 10000 == 0 {
            print_status(&deps, users.clone(), block as u128);
        }

        deadline = TypedStore::<Config, MockStorage>::attach(&deps.storage)
            .load(CONFIG_KEY)
            .unwrap()
            .deadline;
        block += 1;
    }

    // Make sure all users are fully redeemed
    for user in users.clone() {
        let redeem_msg = LPStakingHandleMsg::Redeem { amount: None };
        let result = handle(&mut deps, mock_env(user.0, &[], 1_700_000), redeem_msg);
        total_rewards_output += extract_rewards(result);
    }

    let error = 1.0 - (total_rewards_output as f64 / rewards as f64);
    println!("Error is: {}", error);
    assert!(error >= 0f64 && error < 0.01);

    // Do another run after first iteration is ended
    continue_after_ended(&mut deps, deadline, actions, users);
}

fn continue_after_ended(
    deps: &mut Extern<MockStorage, MockApi, MockQuerier>,
    deadline: u64,
    actions: Vec<&str>,
    users: Vec<HumanAddr>,
) {
    let mut rng = rand::thread_rng();

    let start_block = deadline + 10_001;
    let mut new_deadline = start_block + 500_000;
    let mut rewards = 500_000_000000;
    let mut total_rewards_output = 0;

    let msg = LPStakingHandleMsg::SetDeadline {
        block: new_deadline,
    };
    let _result = handle(deps, mock_env("admin".to_string(), &[], start_block), msg);
    let msg = LPStakingHandleMsg::Receive {
        sender: HumanAddr("admin".to_string()),
        from: HumanAddr("admin".to_string()),
        amount: Uint128(rewards),
        msg: to_binary(&LPStakingReceiveMsg::DepositRewards {}).unwrap(),
    };
    let _result = handle(
        deps,
        mock_env("scrt".to_string(), &[], start_block + 1),
        msg,
    );

    let mut block = start_block + 2;
    while block < (new_deadline + 10_000) {
        let num_of_actions = rng.gen_range(0, 5);

        for _ in 0..num_of_actions {
            let action_idx = rng.gen_range(0, actions.len());

            let user_idx = rng.gen_range(0, users.len());
            let user = users[user_idx].clone();

            let (msg, sender) = msg_from_action(&deps, actions[action_idx], user.clone());
            rewards += extract_reward_deposit(msg.clone());
            let result = handle(deps, mock_env(sender, &[], block), msg);
            total_rewards_output += extract_rewards(result);
        }

        if block % 10000 == 0 {
            print_status(&deps, users.clone(), block.into());
        }

        new_deadline = TypedStore::<Config, MockStorage>::attach(&deps.storage)
            .load(CONFIG_KEY)
            .unwrap()
            .deadline;
        block += 1;
    }

    // Make sure all users are fully redeemed
    for user in users {
        let redeem_msg = LPStakingHandleMsg::Redeem { amount: None };
        let result = handle(deps, mock_env(user.0, &[], 1_700_000), redeem_msg);
        total_rewards_output += extract_rewards(result);
    }

    let error = 1.0 - (total_rewards_output as f64 / rewards as f64);
    println!("Error is: {}", error);
    assert!(error >= 0f64 && error < 0.01);
}

// Tests

#[test]
fn test_claim_pool() {
    let (_init_result, mut deps) = init_helper(10000000); // Claim height is deadline + 1

    let claim_msg = LPStakingHandleMsg::ClaimRewardPool { to: None };
    let handle_response = handle(&mut deps, mock_env("not_admin", &[], 10), claim_msg.clone());
    assert_eq!(
        handle_response.unwrap_err(),
        StdError::GenericErr {
            msg: "not an admin: not_admin".to_string(),
            backtrace: None
        }
    );

    let handle_response = handle(&mut deps, mock_env("admin", &[], 10), claim_msg.clone());
    assert_eq!(
        handle_response.unwrap_err(),
        StdError::GenericErr {
            msg: format!("minimum claim height hasn't passed yet: {}", 10000001),
            backtrace: None
        }
    );

    let handle_response = handle(
        &mut deps,
        mock_env("admin", &[], 10000001),
        claim_msg.clone(),
    );
    assert_eq!(
        handle_response.unwrap_err(),
        StdError::GenericErr {
            msg: "Error performing Balance query: Generic error: Querier system error: No such contract: scrt".to_string(), // No way to test external queries yet
            backtrace: None
        }
    );
}

#[test]
fn test_stop_contract() {
    let (_init_result, mut deps) = init_helper(10000000);

    let stop_msg = LPStakingHandleMsg::StopContract {};
    let handle_response = handle(&mut deps, mock_env("not_admin", &[], 10), stop_msg.clone());
    assert_eq!(
        handle_response.unwrap_err(),
        StdError::GenericErr {
            msg: "not an admin: not_admin".to_string(),
            backtrace: None
        }
    );

    let handle_response = handle(&mut deps, mock_env("admin", &[], 10), stop_msg);
    let unwrapped_result: LPStakingHandleAnswer =
        from_binary(&handle_response.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&LPStakingHandleAnswer::StopContract { status: LPStakingResponseStatus::Success }).unwrap()
    );

    let redeem_msg = LPStakingHandleMsg::Redeem { amount: None };
    let handle_response = handle(&mut deps, mock_env("user", &[], 20), redeem_msg);
    assert_eq!(
        handle_response.unwrap_err(),
        StdError::GenericErr {
            msg: "this contract is stopped and this action is not allowed".to_string(),
            backtrace: None
        }
    );

    let resume_msg = LPStakingHandleMsg::ResumeContract {};
    let handle_response = handle(&mut deps, mock_env("admin", &[], 21), resume_msg);
    let unwrapped_result: LPStakingHandleAnswer =
        from_binary(&handle_response.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&LPStakingHandleAnswer::ResumeContract { status: LPStakingResponseStatus::Success }).unwrap()
    );

    let redeem_msg = LPStakingHandleMsg::Redeem { amount: None };
    let handle_response = handle(&mut deps, mock_env("user", &[], 20), redeem_msg);
    let unwrapped_result: LPStakingHandleAnswer =
        from_binary(&handle_response.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&LPStakingHandleAnswer::Redeem { status: LPStakingResponseStatus::Success }).unwrap()
    );
}

#[test]
fn test_admin() {
    let (_init_result, mut deps) = init_helper(10000000);

    let admin_action_msg = LPStakingHandleMsg::ChangeAdmin {
        address: HumanAddr("not_admin".to_string()),
    };
    let handle_response = handle(&mut deps, mock_env("not_admin", &[], 1), admin_action_msg);
    assert_eq!(
        handle_response.unwrap_err(),
        StdError::GenericErr {
            msg: "not an admin: not_admin".to_string(),
            backtrace: None
        }
    );

    let admin_action_msg = LPStakingHandleMsg::ChangeAdmin {
        address: HumanAddr("new_admin".to_string()),
    };
    let handle_response = handle(&mut deps, mock_env("admin", &[], 1), admin_action_msg);
    let unwrapped_result: LPStakingHandleAnswer =
        from_binary(&handle_response.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&LPStakingHandleAnswer::ChangeAdmin { status: LPStakingResponseStatus::Success }).unwrap()
    );

    let admin_action_msg = LPStakingHandleMsg::ChangeAdmin {
        address: HumanAddr("not_admin".to_string()),
    };
    let handle_response = handle(&mut deps, mock_env("admin", &[], 1), admin_action_msg);
    assert_eq!(
        handle_response.unwrap_err(),
        StdError::GenericErr {
            msg: "not an admin: admin".to_string(),
            backtrace: None
        }
    );

    let admin_action_msg = LPStakingHandleMsg::ChangeAdmin {
        address: HumanAddr("not_admin".to_string()),
    };
    let handle_response = handle(&mut deps, mock_env("new_admin", &[], 1), admin_action_msg);
    let unwrapped_result: LPStakingHandleAnswer =
        from_binary(&handle_response.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&LPStakingHandleAnswer::ChangeAdmin { status: LPStakingResponseStatus::Success }).unwrap()
    );
}

#[test]
fn test_single_run() {
    let mut rng = rand::thread_rng();

    let deadline: u64 = rng.gen_range(100_000, 5_000_000);
    let rewards: u128 = rng.gen_range(1_000_000000, 10_000_000_000000); // 1k-10mn SCRT

    sanity_run(rewards, deadline);
}

#[test]
#[ignore]
fn test_simulations() {
    let mut rng = rand::thread_rng();

    for run in 0..100 {
        let deadline: u64 = rng.gen_range(100_000, 5_000_000);
        let rewards: u128 = rng.gen_range(1_000_000000, 10_000_000_000000); // 1k-10mn SCRT

        println!("$$$$$$$$$$$$$$$$$$ Run Parameters $$$$$$$$$$$$$$$$$$");
        println!("Run number: {}", run + 1);
        println!("Rewards: {}", rewards);
        println!("Deadline: {}", deadline);
        println!();

        sanity_run(rewards, deadline);
    }
}

/// SNIP20 token handle messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum Snip20HandleMsg {
    // Basic SNIP20 functions
    Transfer {
        recipient: HumanAddr,
        amount: Uint128,
        padding: Option<String>,
    },
}
