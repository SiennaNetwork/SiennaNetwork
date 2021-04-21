#[cfg(test)]

use crate::contract::*;
use crate::msg::ResponseStatus;
use crate::msg::{InitConfig, InitialBalance};
use cosmwasm_std::testing::*;
use cosmwasm_std::{from_binary, BlockInfo, ContractInfo, MessageInfo, QueryResponse, WasmMsg};
use std::any::Any;

// Helper functions

fn init_helper(
    initial_balances: Vec<InitialBalance>,
) -> (
    StdResult<InitResponse>,
    Extern<MockStorage, MockApi, MockQuerier>,
) {
    let mut deps = mock_dependencies(20, &[]);
    let env = mock_env("instantiator", &[]);

    let init_msg = InitMsg {
        name: "sec-sec".to_string(),
        admin: Some(HumanAddr("admin".to_string())),
        symbol: "SECSEC".to_string(),
        decimals: 8,
        initial_balances: Some(initial_balances),
        prng_seed: Binary::from("lolz fun yay".as_bytes()),
        config: None,
    };

    (init(&mut deps, env, init_msg), deps)
}

/// Will return a ViewingKey only for the first account in `initial_balances`
fn auth_query_helper(
    initial_balances: Vec<InitialBalance>,
) -> (ViewingKey, Extern<MockStorage, MockApi, MockQuerier>) {
    let (init_result, mut deps) = init_helper(initial_balances.clone());
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let account = initial_balances[0].address.clone();
    let create_vk_msg = HandleMsg::CreateViewingKey {
        entropy: "42".to_string(),
        padding: None,
    };
    let handle_response = handle(&mut deps, mock_env(account.0, &[]), create_vk_msg).unwrap();
    let vk = match from_binary(&handle_response.data.unwrap()).unwrap() {
        HandleAnswer::CreateViewingKey { key } => key,
        _ => panic!("Unexpected result from handle"),
    };

    (vk, deps)
}

fn extract_error_msg<T: Any>(error: StdResult<T>) -> String {
    match error {
        Ok(response) => {
            let bin_err = (&response as &dyn Any)
                .downcast_ref::<QueryResponse>()
                .expect("An error was expected, but no error could be extracted");
            match from_binary(bin_err).unwrap() {
                QueryAnswer::ViewingKeyError { msg } => msg,
                _ => panic!("Unexpected query answer"),
            }
        }
        Err(err) => match err {
            StdError::GenericErr { msg, .. } => msg,
            _ => panic!("Unexpected result from init"),
        },
    }
}

fn ensure_success(handle_result: HandleResponse) -> bool {
    let handle_result: HandleAnswer = from_binary(&handle_result.data.unwrap()).unwrap();

    match handle_result {
        HandleAnswer::Deposit { status }
        | HandleAnswer::Redeem { status }
        | HandleAnswer::Transfer { status }
        | HandleAnswer::Send { status }
        | HandleAnswer::Burn { status }
        | HandleAnswer::RegisterReceive { status }
        | HandleAnswer::SetViewingKey { status }
        | HandleAnswer::TransferFrom { status }
        | HandleAnswer::SendFrom { status }
        | HandleAnswer::BurnFrom { status }
        | HandleAnswer::Mint { status }
        | HandleAnswer::ChangeAdmin { status }
        | HandleAnswer::SetContractStatus { status }
        | HandleAnswer::SetMinters { status }
        | HandleAnswer::AddMinters { status }
        | HandleAnswer::RemoveMinters { status } => {
            matches!(status, ResponseStatus::Success {..})
        }
        _ => panic!("HandleAnswer not supported for success extraction"),
    }
}

// Init tests

#[test]
fn test_init_sanity() {
    let (init_result, deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(5000),
    }]);
    assert_eq!(init_result.unwrap(), InitResponse::default());

    let config = ReadonlyConfig::from_storage(&deps.storage);
    let constants = config.constants().unwrap();
    assert_eq!(config.total_supply(), 5000);
    assert_eq!(config.contract_status(), ContractStatusLevel::NormalRun);
    assert_eq!(constants.name, "sec-sec".to_string());
    assert_eq!(constants.admin, HumanAddr("admin".to_string()));
    assert_eq!(constants.symbol, "SECSEC".to_string());
    assert_eq!(constants.decimals, 8);
    assert_eq!(
        constants.prng_seed,
        sha_256("lolz fun yay".to_owned().as_bytes())
    );
    assert_eq!(constants.total_supply_is_public, false);
}

#[test]
fn test_total_supply_overflow() {
    let (init_result, _deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(u128::max_value()),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let (init_result, _deps) = init_helper(vec![
        InitialBalance {
            address: HumanAddr("lebron".to_string()),
            amount: Uint128(u128::max_value()),
        },
        InitialBalance {
            address: HumanAddr("giannis".to_string()),
            amount: Uint128(1),
        },
    ]);
    let error = extract_error_msg(init_result);
    assert_eq!(
        error,
        "The sum of all initial balances exceeds the maximum possible total supply"
    );
}

// Handle tests

#[test]
fn test_handle_transfer() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::Transfer {
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(1000),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let result = handle_result.unwrap();
    assert!(ensure_success(result));
    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let alice_canonical = deps
        .api
        .canonical_address(&HumanAddr("alice".to_string()))
        .unwrap();
    let balances = ReadonlyBalances::from_storage(&deps.storage);
    assert_eq!(5000 - 1000, balances.account_amount(&bob_canonical));
    assert_eq!(1000, balances.account_amount(&alice_canonical));

    let handle_msg = HandleMsg::Transfer {
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(10000),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient funds"));
}

#[test]
fn test_handle_send() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::RegisterReceive {
        code_hash: "this_is_a_hash_of_a_code".to_string(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("contract", &[]), handle_msg);
    let result = handle_result.unwrap();
    assert!(ensure_success(result));

    let handle_msg = HandleMsg::Send {
        recipient: HumanAddr("contract".to_string()),
        amount: Uint128(100),
        padding: None,
        msg: Some(to_binary("hey hey you you").unwrap()),
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let result = handle_result.unwrap();
    assert!(ensure_success(result.clone()));
    assert!(result.messages.contains(&CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: HumanAddr("contract".to_string()),
        callback_code_hash: "this_is_a_hash_of_a_code".to_string(),
        msg: Snip20ReceiveMsg::new(
            HumanAddr("bob".to_string()),
            HumanAddr("bob".to_string()),
            Uint128(100),
            Some(to_binary("hey hey you you").unwrap())
        )
        .into_binary()
        .unwrap(),
        send: vec![]
    })));
}

#[test]
fn test_handle_register_receive() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::RegisterReceive {
        code_hash: "this_is_a_hash_of_a_code".to_string(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("contract", &[]), handle_msg);
    let result = handle_result.unwrap();
    assert!(ensure_success(result));

    let hash = get_receiver_hash(&deps.storage, &HumanAddr("contract".to_string()))
        .unwrap()
        .unwrap();
    assert_eq!(hash, "this_is_a_hash_of_a_code".to_string());
}

#[test]
fn test_handle_create_viewing_key() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::CreateViewingKey {
        entropy: "".to_string(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    let answer: HandleAnswer = from_binary(&handle_result.unwrap().data.unwrap()).unwrap();

    let key = match answer {
        HandleAnswer::CreateViewingKey { key } => key,
        _ => panic!("NOPE"),
    };
    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let saved_vk = read_viewing_key(&deps.storage, &bob_canonical).unwrap();
    assert!(key.check_viewing_key(saved_vk.as_slice()));
}

#[test]
fn test_handle_set_viewing_key() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    // Set VK
    let handle_msg = HandleMsg::SetViewingKey {
        key: "hi lol".to_string(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let unwrapped_result: HandleAnswer =
        from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&HandleAnswer::SetViewingKey {
            status: ResponseStatus::Success
        })
        .unwrap(),
    );

    // Set valid VK
    let actual_vk = ViewingKey("x".to_string().repeat(VIEWING_KEY_SIZE));
    let handle_msg = HandleMsg::SetViewingKey {
        key: actual_vk.0.clone(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let unwrapped_result: HandleAnswer =
        from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&HandleAnswer::SetViewingKey { status: Success }).unwrap(),
    );
    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let saved_vk = read_viewing_key(&deps.storage, &bob_canonical).unwrap();
    assert!(actual_vk.check_viewing_key(&saved_vk));
}

#[test]
fn test_handle_transfer_from() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    // Transfer before allowance
    let handle_msg = HandleMsg::TransferFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(2500),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));

    // Transfer more than allowance
    let handle_msg = HandleMsg::IncreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: Some(1_571_797_420),
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    let handle_msg = HandleMsg::TransferFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(2500),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));

    // Transfer after allowance expired
    let handle_msg = HandleMsg::TransferFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
    };
    let handle_result = handle(
        &mut deps,
        Env {
            block: BlockInfo {
                height: 12_345,
                time: 1_571_797_420,
                chain_id: "cosmos-testnet-14002".to_string(),
            },
            message: MessageInfo {
                sender: HumanAddr("bob".to_string()),
                sent_funds: vec![],
            },
            contract: ContractInfo {
                address: HumanAddr::from(MOCK_CONTRACT_ADDR),
            },
            contract_key: Some("".to_string()),
            contract_code_hash: "".to_string(),
        },
        handle_msg,
    );
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));

    // Sanity check
    let handle_msg = HandleMsg::TransferFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let alice_canonical = deps
        .api
        .canonical_address(&HumanAddr("alice".to_string()))
        .unwrap();
    let bob_balance = crate::state::ReadonlyBalances::from_storage(&deps.storage)
        .account_amount(&bob_canonical);
    let alice_balance = crate::state::ReadonlyBalances::from_storage(&deps.storage)
        .account_amount(&alice_canonical);
    assert_eq!(bob_balance, 5000 - 2000);
    assert_eq!(alice_balance, 2000);
    let total_supply = ReadonlyConfig::from_storage(&deps.storage).total_supply();
    assert_eq!(total_supply, 5000);

    // Second send more than allowance
    let handle_msg = HandleMsg::TransferFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(1),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));
}

#[test]
fn test_handle_send_from() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    // Send before allowance
    let handle_msg = HandleMsg::SendFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(2500),
        msg: None,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));

    // Send more than allowance
    let handle_msg = HandleMsg::IncreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    let handle_msg = HandleMsg::SendFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(2500),
        msg: None,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));

    // Sanity check
    let handle_msg = HandleMsg::RegisterReceive {
        code_hash: "lolz".to_string(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("contract", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    let send_msg = Binary::from(r#"{ "some_msg": { "some_key": "some_val" } }"#.as_bytes());
    let snip20_msg = Snip20ReceiveMsg::new(
        HumanAddr("alice".to_string()),
        HumanAddr("bob".to_string()),
        Uint128(2000),
        Some(send_msg.clone()),
    );
    let handle_msg = HandleMsg::SendFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("contract".to_string()),
        amount: Uint128(2000),
        msg: Some(send_msg),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    assert!(handle_result.unwrap().messages.contains(
        &snip20_msg
            .into_cosmos_msg("lolz".to_string(), HumanAddr("contract".to_string()))
            .unwrap()
    ));
    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let contract_canonical = deps
        .api
        .canonical_address(&HumanAddr("contract".to_string()))
        .unwrap();
    let bob_balance = crate::state::ReadonlyBalances::from_storage(&deps.storage)
        .account_amount(&bob_canonical);
    let contract_balance = crate::state::ReadonlyBalances::from_storage(&deps.storage)
        .account_amount(&contract_canonical);
    assert_eq!(bob_balance, 5000 - 2000);
    assert_eq!(contract_balance, 2000);
    let total_supply = ReadonlyConfig::from_storage(&deps.storage).total_supply();
    assert_eq!(total_supply, 5000);

    // Second send more than allowance
    let handle_msg = HandleMsg::SendFrom {
        owner: HumanAddr("bob".to_string()),
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(1),
        msg: None,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));
}

#[test]
fn test_handle_burn_from() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    // Burn before allowance
    let handle_msg = HandleMsg::BurnFrom {
        owner: HumanAddr("bob".to_string()),
        amount: Uint128(2500),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));

    // Burn more than allowance
    let handle_msg = HandleMsg::IncreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    let handle_msg = HandleMsg::BurnFrom {
        owner: HumanAddr("bob".to_string()),
        amount: Uint128(2500),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));

    // Sanity check
    let handle_msg = HandleMsg::BurnFrom {
        owner: HumanAddr("bob".to_string()),
        amount: Uint128(2000),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );
    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let bob_balance = crate::state::ReadonlyBalances::from_storage(&deps.storage)
        .account_amount(&bob_canonical);
    assert_eq!(bob_balance, 5000 - 2000);
    let total_supply = ReadonlyConfig::from_storage(&deps.storage).total_supply();
    assert_eq!(total_supply, 5000 - 2000);

    // Second burn more than allowance
    let handle_msg = HandleMsg::BurnFrom {
        owner: HumanAddr("bob".to_string()),
        amount: Uint128(1),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("alice", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("insufficient allowance"));
}

#[test]
fn test_handle_decrease_allowance() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::DecreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let alice_canonical = deps
        .api
        .canonical_address(&HumanAddr("alice".to_string()))
        .unwrap();

    let allowance = read_allowance(&deps.storage, &bob_canonical, &alice_canonical).unwrap();
    assert_eq!(
        allowance,
        crate::state::Allowance {
            amount: 0,
            expiration: None
        }
    );

    let handle_msg = HandleMsg::IncreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let handle_msg = HandleMsg::DecreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(50),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let allowance = read_allowance(&deps.storage, &bob_canonical, &alice_canonical).unwrap();
    assert_eq!(
        allowance,
        crate::state::Allowance {
            amount: 1950,
            expiration: None
        }
    );
}

#[test]
fn test_handle_increase_allowance() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::IncreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let bob_canonical = deps
        .api
        .canonical_address(&HumanAddr("bob".to_string()))
        .unwrap();
    let alice_canonical = deps
        .api
        .canonical_address(&HumanAddr("alice".to_string()))
        .unwrap();

    let allowance = read_allowance(&deps.storage, &bob_canonical, &alice_canonical).unwrap();
    assert_eq!(
        allowance,
        crate::state::Allowance {
            amount: 2000,
            expiration: None
        }
    );

    let handle_msg = HandleMsg::IncreaseAllowance {
        spender: HumanAddr("alice".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let allowance = read_allowance(&deps.storage, &bob_canonical, &alice_canonical).unwrap();
    assert_eq!(
        allowance,
        crate::state::Allowance {
            amount: 4000,
            expiration: None
        }
    );
}

#[test]
fn test_handle_change_admin() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::ChangeAdmin {
        address: HumanAddr("bob".to_string()),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let admin = ReadonlyConfig::from_storage(&deps.storage)
        .constants()
        .unwrap()
        .admin;
    assert_eq!(admin, HumanAddr("bob".to_string()));
}

#[test]
fn test_handle_set_contract_status() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("admin".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let contract_status = ReadonlyConfig::from_storage(&deps.storage).contract_status();
    assert!(matches!(contract_status, ContractStatusLevel::StopAll{..}));
}

#[test]
fn test_handle_redeem() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("butler".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::Redeem {
        amount: Uint128(1000),
        denom: None,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("butler", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let balances = ReadonlyBalances::from_storage(&deps.storage);
    let canonical = deps
        .api
        .canonical_address(&HumanAddr("butler".to_string()))
        .unwrap();
    assert_eq!(balances.account_amount(&canonical), 4000)
}

#[test]
fn test_handle_deposit() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::Deposit { padding: None };
    let handle_result = handle(
        &mut deps,
        mock_env(
            "lebron",
            &[Coin {
                denom: "uscrt".to_string(),
                amount: Uint128(1000),
            }],
        ),
        handle_msg,
    );
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let balances = ReadonlyBalances::from_storage(&deps.storage);
    let canonical = deps
        .api
        .canonical_address(&HumanAddr("lebron".to_string()))
        .unwrap();
    assert_eq!(balances.account_amount(&canonical), 6000)
}

#[test]
fn test_handle_burn() {
    let initial_amount: u128 = 5000;
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(initial_amount),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let supply = ReadonlyConfig::from_storage(&deps.storage).total_supply();
    let burn_amount: u128 = 100;
    let handle_msg = HandleMsg::Burn {
        amount: Uint128(burn_amount),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("lebron", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "Pause handle failed: {}",
        handle_result.err().unwrap()
    );

    let new_supply = ReadonlyConfig::from_storage(&deps.storage).total_supply();
    assert_eq!(new_supply, supply - burn_amount);
}

#[test]
fn test_handle_mint() {
    let initial_amount: u128 = 5000;
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(initial_amount),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let supply = ReadonlyConfig::from_storage(&deps.storage).total_supply();
    let mint_amount: u128 = 100;
    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("lebron".to_string()),
        amount: Uint128(mint_amount),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "Pause handle failed: {}",
        handle_result.err().unwrap()
    );

    let new_supply = ReadonlyConfig::from_storage(&deps.storage).total_supply();
    assert_eq!(new_supply, supply + mint_amount);
}

#[test]
fn test_handle_admin_commands() {
    let admin_err = "Admin commands can only be run from admin address".to_string();

    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let pause_msg = HandleMsg::SetContractStatus {
        level: ContractStatusLevel::StopAllButRedeems,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("not_admin", &[]), pause_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains(&admin_err.clone()));

    let mint_msg = HandleMsg::AddMinters {
        minters: vec![HumanAddr("not_admin".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("not_admin", &[]), mint_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains(&admin_err.clone()));

    let mint_msg = HandleMsg::RemoveMinters {
        minters: vec![HumanAddr("admin".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("not_admin", &[]), mint_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains(&admin_err.clone()));

    let mint_msg = HandleMsg::SetMinters {
        minters: vec![HumanAddr("not_admin".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("not_admin", &[]), mint_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains(&admin_err.clone()));

    let change_admin_msg = HandleMsg::ChangeAdmin {
        address: HumanAddr("not_admin".to_string()),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("not_admin", &[]), change_admin_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains(&admin_err.clone()));
}

#[test]
fn test_handle_pause_with_withdrawals() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let pause_msg = HandleMsg::SetContractStatus {
        level: ContractStatusLevel::StopAllButRedeems,
        padding: None,
    };

    let handle_result = handle(&mut deps, mock_env("admin", &[]), pause_msg);
    assert!(
        handle_result.is_ok(),
        "Pause handle failed: {}",
        handle_result.err().unwrap()
    );

    let send_msg = HandleMsg::Transfer {
        recipient: HumanAddr("account".to_string()),
        amount: Uint128(123),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), send_msg);
    let error = extract_error_msg(handle_result);
    assert_eq!(
        error,
        "This contract is stopped and this action is not allowed".to_string()
    );

    let withdraw_msg = HandleMsg::Redeem {
        amount: Uint128(5000),
        denom: None,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("lebron", &[]), withdraw_msg);
    assert!(
        handle_result.is_ok(),
        "Withdraw failed: {}",
        handle_result.err().unwrap()
    );
}

#[test]
fn test_handle_pause_all() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("lebron".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let pause_msg = HandleMsg::SetContractStatus {
        level: ContractStatusLevel::StopAll,
        padding: None,
    };

    let handle_result = handle(&mut deps, mock_env("admin", &[]), pause_msg);
    assert!(
        handle_result.is_ok(),
        "Pause handle failed: {}",
        handle_result.err().unwrap()
    );

    let send_msg = HandleMsg::Transfer {
        recipient: HumanAddr("account".to_string()),
        amount: Uint128(123),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), send_msg);
    let error = extract_error_msg(handle_result);
    assert_eq!(
        error,
        "This contract is stopped and this action is not allowed".to_string()
    );

    let withdraw_msg = HandleMsg::Redeem {
        amount: Uint128(5000),
        denom: None,
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("lebron", &[]), withdraw_msg);
    let error = extract_error_msg(handle_result);
    assert_eq!(
        error,
        "This contract is stopped and this action is not allowed".to_string()
    );
}

#[test]
fn test_handle_set_minters() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::SetMinters {
        minters: vec![HumanAddr("bob".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("Admin commands can only be run from admin address"));

    let handle_msg = HandleMsg::SetMinters {
        minters: vec![HumanAddr("bob".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("allowed to minter accounts only"));
}

#[test]
fn test_handle_add_minters() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::AddMinters {
        minters: vec![HumanAddr("bob".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("Admin commands can only be run from admin address"));

    let handle_msg = HandleMsg::AddMinters {
        minters: vec![HumanAddr("bob".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));
}

#[test]
fn test_handle_remove_minters() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::RemoveMinters {
        minters: vec![HumanAddr("admin".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("Admin commands can only be run from admin address"));

    let handle_msg = HandleMsg::RemoveMinters {
        minters: vec![HumanAddr("admin".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("allowed to minter accounts only"));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("allowed to minter accounts only"));

    // Removing another extra time to ensure nothing funky happens
    let handle_msg = HandleMsg::RemoveMinters {
        minters: vec![HumanAddr("admin".to_string())],
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("allowed to minter accounts only"));

    let handle_msg = HandleMsg::Mint {
        recipient: HumanAddr("bob".to_string()),
        amount: Uint128(100),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("admin", &[]), handle_msg);
    let error = extract_error_msg(handle_result);
    assert!(error.contains("allowed to minter accounts only"));
}

// Query tests

#[test]
fn test_authenticated_queries() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("giannis".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let no_vk_yet_query_msg = QueryMsg::Balance {
        address: HumanAddr("giannis".to_string()),
        key: "no_vk_yet".to_string(),
    };
    let query_result = query(&deps, no_vk_yet_query_msg);
    let error = extract_error_msg(query_result);
    assert_eq!(
        error,
        "Wrong viewing key for this address or viewing key not set".to_string()
    );

    let create_vk_msg = HandleMsg::CreateViewingKey {
        entropy: "34".to_string(),
        padding: None,
    };
    let handle_response = handle(&mut deps, mock_env("giannis", &[]), create_vk_msg).unwrap();
    let vk = match from_binary(&handle_response.data.unwrap()).unwrap() {
        HandleAnswer::CreateViewingKey { key } => key,
        _ => panic!("Unexpected result from handle"),
    };

    let query_balance_msg = QueryMsg::Balance {
        address: HumanAddr("giannis".to_string()),
        key: vk.0,
    };

    let query_response = query(&deps, query_balance_msg).unwrap();
    let balance = match from_binary(&query_response).unwrap() {
        QueryAnswer::Balance { amount } => amount,
        _ => panic!("Unexpected result from query"),
    };
    assert_eq!(balance, Uint128(5000));

    let wrong_vk_query_msg = QueryMsg::Balance {
        address: HumanAddr("giannis".to_string()),
        key: "wrong_vk".to_string(),
    };
    let query_result = query(&deps, wrong_vk_query_msg);
    let error = extract_error_msg(query_result);
    assert_eq!(
        error,
        "Wrong viewing key for this address or viewing key not set".to_string()
    );
}

#[test]
fn test_query_token_info() {
    let init_name = "sec-sec".to_string();
    let init_admin = HumanAddr("admin".to_string());
    let init_symbol = "SECSEC".to_string();
    let init_decimals = 8;
    let init_config: InitConfig = from_binary(&Binary::from(
        r#"{ "public_total_supply": true }"#.as_bytes(),
    ))
    .unwrap();
    let init_supply = Uint128(5000);

    let mut deps = mock_dependencies(20, &[]);
    let env = mock_env("instantiator", &[]);
    let init_msg = InitMsg {
        name: init_name.clone(),
        admin: Some(init_admin.clone()),
        symbol: init_symbol.clone(),
        decimals: init_decimals.clone(),
        initial_balances: Some(vec![InitialBalance {
            address: HumanAddr("giannis".to_string()),
            amount: init_supply,
        }]),
        prng_seed: Binary::from("lolz fun yay".as_bytes()),
        config: Some(init_config),
    };
    let init_result = init(&mut deps, env, init_msg);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let query_msg = QueryMsg::TokenInfo {};
    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Init failed: {}",
        query_result.err().unwrap()
    );
    let query_answer: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    match query_answer {
        QueryAnswer::TokenInfo {
            name,
            symbol,
            decimals,
            total_supply,
        } => {
            assert_eq!(name, init_name);
            assert_eq!(symbol, init_symbol);
            assert_eq!(decimals, init_decimals);
            assert_eq!(total_supply, Some(Uint128(5000)));
        }
        _ => panic!("unexpected"),
    }
}

#[test]
fn test_query_allowance() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("giannis".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::IncreaseAllowance {
        spender: HumanAddr("lebron".to_string()),
        amount: Uint128(2000),
        padding: None,
        expiration: None,
    };
    let handle_result = handle(&mut deps, mock_env("giannis", &[]), handle_msg);
    assert!(
        handle_result.is_ok(),
        "handle() failed: {}",
        handle_result.err().unwrap()
    );

    let vk1 = ViewingKey("key1".to_string());
    let vk2 = ViewingKey("key2".to_string());

    let query_msg = QueryMsg::Allowance {
        owner: HumanAddr("giannis".to_string()),
        spender: HumanAddr("lebron".to_string()),
        key: vk1.0.clone(),
    };
    let query_result = query(&deps, query_msg);
    assert!(
        query_result.is_ok(),
        "Query failed: {}",
        query_result.err().unwrap()
    );
    let error = extract_error_msg(query_result);
    assert!(error.contains("Wrong viewing key"));

    let handle_msg = HandleMsg::SetViewingKey {
        key: vk1.0.clone(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("lebron", &[]), handle_msg);
    let unwrapped_result: HandleAnswer =
        from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&HandleAnswer::SetViewingKey {
            status: ResponseStatus::Success
        })
        .unwrap(),
    );

    let handle_msg = HandleMsg::SetViewingKey {
        key: vk2.0.clone(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("giannis", &[]), handle_msg);
    let unwrapped_result: HandleAnswer =
        from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&HandleAnswer::SetViewingKey {
            status: ResponseStatus::Success
        })
        .unwrap(),
    );

    let query_msg = QueryMsg::Allowance {
        owner: HumanAddr("giannis".to_string()),
        spender: HumanAddr("lebron".to_string()),
        key: vk1.0.clone(),
    };
    let query_result = query(&deps, query_msg);
    let allowance = match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::Allowance { allowance, .. } => allowance,
        _ => panic!("Unexpected"),
    };
    assert_eq!(allowance, Uint128(2000));

    let query_msg = QueryMsg::Allowance {
        owner: HumanAddr("giannis".to_string()),
        spender: HumanAddr("lebron".to_string()),
        key: vk2.0.clone(),
    };
    let query_result = query(&deps, query_msg);
    let allowance = match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::Allowance { allowance, .. } => allowance,
        _ => panic!("Unexpected"),
    };
    assert_eq!(allowance, Uint128(2000));

    let query_msg = QueryMsg::Allowance {
        owner: HumanAddr("lebron".to_string()),
        spender: HumanAddr("giannis".to_string()),
        key: vk2.0.clone(),
    };
    let query_result = query(&deps, query_msg);
    let allowance = match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::Allowance { allowance, .. } => allowance,
        _ => panic!("Unexpected"),
    };
    assert_eq!(allowance, Uint128(0));
}

#[test]
fn test_query_balance() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let unwrapped_result: HandleAnswer =
        from_binary(&handle_result.unwrap().data.unwrap()).unwrap();
    assert_eq!(
        to_binary(&unwrapped_result).unwrap(),
        to_binary(&HandleAnswer::SetViewingKey {
            status: ResponseStatus::Success
        })
        .unwrap(),
    );

    let query_msg = QueryMsg::Balance {
        address: HumanAddr("bob".to_string()),
        key: "wrong_key".to_string(),
    };
    let query_result = query(&deps, query_msg);
    let error = extract_error_msg(query_result);
    assert!(error.contains("Wrong viewing key"));

    let query_msg = QueryMsg::Balance {
        address: HumanAddr("bob".to_string()),
        key: "key".to_string(),
    };
    let query_result = query(&deps, query_msg);
    let balance = match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::Balance { amount } => amount,
        _ => panic!("Unexpected"),
    };
    assert_eq!(balance, Uint128(5000));
}

#[test]
fn test_query_transfer_history() {
    let (init_result, mut deps) = init_helper(vec![InitialBalance {
        address: HumanAddr("bob".to_string()),
        amount: Uint128(5000),
    }]);
    assert!(
        init_result.is_ok(),
        "Init failed: {}",
        init_result.err().unwrap()
    );

    let handle_msg = HandleMsg::SetViewingKey {
        key: "key".to_string(),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    assert!(ensure_success(handle_result.unwrap()));

    let handle_msg = HandleMsg::Transfer {
        recipient: HumanAddr("alice".to_string()),
        amount: Uint128(1000),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let result = handle_result.unwrap();
    assert!(ensure_success(result));
    let handle_msg = HandleMsg::Transfer {
        recipient: HumanAddr("banana".to_string()),
        amount: Uint128(500),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let result = handle_result.unwrap();
    assert!(ensure_success(result));
    let handle_msg = HandleMsg::Transfer {
        recipient: HumanAddr("mango".to_string()),
        amount: Uint128(2500),
        padding: None,
    };
    let handle_result = handle(&mut deps, mock_env("bob", &[]), handle_msg);
    let result = handle_result.unwrap();
    assert!(ensure_success(result));

    let query_msg = QueryMsg::TransferHistory {
        address: HumanAddr("bob".to_string()),
        key: "key".to_string(),
        page: None,
        page_size: 0,
    };
    let query_result = query(&deps, query_msg);
    // let a: QueryAnswer = from_binary(&query_result.unwrap()).unwrap();
    // println!("{:?}", a);
    let transfers = match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::TransferHistory { txs } => txs,
        _ => panic!("Unexpected"),
    };
    assert!(transfers.is_empty());

    let query_msg = QueryMsg::TransferHistory {
        address: HumanAddr("bob".to_string()),
        key: "key".to_string(),
        page: None,
        page_size: 10,
    };
    let query_result = query(&deps, query_msg);
    let transfers = match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::TransferHistory { txs } => txs,
        _ => panic!("Unexpected"),
    };
    assert_eq!(transfers.len(), 3);

    let query_msg = QueryMsg::TransferHistory {
        address: HumanAddr("bob".to_string()),
        key: "key".to_string(),
        page: None,
        page_size: 2,
    };
    let query_result = query(&deps, query_msg);
    let transfers = match from_binary(&query_result.unwrap()).unwrap() {
        QueryAnswer::TransferHistory { txs } => txs,
        _ => panic!("Unexpected"),
    };
    assert_eq!(transfers.len(), 2);
}
