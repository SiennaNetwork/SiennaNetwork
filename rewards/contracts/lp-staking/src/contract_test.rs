#![cfg(test)]

use crate::{contract::*, msg::*};
use scrt_finance::types::{TokenInfo};
use cosmwasm_std::{
    Coin,
    HumanAddr, Uint128,
    StdResult, StdError,
    Binary, to_binary, from_binary, Env, Extern,
    BlockInfo, MessageInfo, ContractInfo,InitResponse,
    testing::{mock_dependencies, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR}
};
use scrt_finance::ContractInfo as ContractInfoWithHash;
use serde::{Deserialize, Serialize};

// Helper functions

fn init_helper(
    deadline: u64,
) -> (
    StdResult<InitResponse>,
    Extern<MockStorage, MockApi, MockQuerier>,
) {
    let mut deps = mock_dependencies(20, &[]);
    let env = mock_env("admin", &[], 1);

    let init_msg = LPStakingInitMsg {
        deadline,
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

// Tests

//Not implemented yet
/*
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
*/
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
    
    // If there was an error, chack if it was due to the contract being stopped
    if let Err(err) = handle_response {
        match err {
            StdError::GenericErr { msg, .. } => {
                if msg != "this contract is stopped and this action is not allowed" {
                    panic!("Contract should have resumed.")
                }
            },
            _ => ()
        }
    }
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
