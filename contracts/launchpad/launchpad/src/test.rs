use crate::contract::*;

use amm_shared::{
    fadroma,
    TokenType,
    querier::{MockContractInstance, MockQuerier},
    msg::launchpad::{
        HandleMsg, InitMsg, QueryMsg, QueryResponse,
        ReceiverCallbackMsg, TokenSettings,
    }
};

use fadroma::platform::{
    from_binary,
    testing::{mock_env, MockApi, MockStorage},
    to_binary, Binary, Coin, Extern, HumanAddr, StdError, Uint128,
    Callback, ContractLink
};

fn mock_deps() -> Extern<MockStorage, MockApi, MockQuerier> {
    Extern {
        storage: MockStorage::default(),
        api: MockApi::new(123),
        querier: MockQuerier::new(
            &[],
            vec![
                MockContractInstance {
                    instance: ContractLink {
                        address: HumanAddr::from("snip20-token-1"),
                        code_hash: "".to_string(),
                    },
                    token_decimals: 18,
                    token_supply: Uint128::from(2500_u128),
                },
                MockContractInstance {
                    instance: ContractLink {
                        address: HumanAddr::from("snip20-token-2"),
                        code_hash: "".to_string(),
                    },
                    token_decimals: 18,
                    token_supply: Uint128::from(2500_u128),
                },
            ],
        ),
    }
}

fn 
get_deps_after_init(tokens: Vec<TokenSettings>) -> Extern<MockStorage, MockApi, MockQuerier> {
    let mut deps = mock_deps();
    let env = mock_env("admin", &[]);

    init(
        &mut deps,
        env.clone(),
        InitMsg {
            tokens,
            admin: env.message.sender,
            prng_seed: to_binary(&"whatever").unwrap(),
            entropy: to_binary(&"whatever").unwrap(),
            callback: None
        },
    )
    .unwrap();

    deps
}

#[test]
fn test_init() {
    get_deps_after_init(vec![TokenSettings {
        token_type: TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
        segment: Uint128(25_000_000_000_u128),
        bounding_period: 0,
    }]);
}

#[test]
fn lock_success() {
    let mut deps = get_deps_after_init(vec![
        TokenSettings {
            token_type: TokenType::NativeToken {
                denom: "uscrt".to_string(),
            },
            segment: Uint128(25_000_000_000_u128),
            bounding_period: 0,
        },
        TokenSettings {
            token_type: TokenType::CustomToken {
                contract_addr: HumanAddr::from("snip20-token-1"),
                token_code_hash: "hash".to_string(),
            },
            segment: Uint128(10_000_000_000_u128),
            bounding_period: 0,
        },
    ]);

    let env = mock_env("account-1", &[Coin::new(25_000_000_000_u128, "uscrt")]);

    let response = handle(
        &mut deps,
        env,
        HandleMsg::Lock {
            amount: Uint128(25_000_000_000_u128),
        },
    )
    .unwrap();

    assert_eq!(response.messages.len(), 0);
    // Locked amount
    assert_eq!(
        response.log.get(1).unwrap().value,
        "25000000000".to_string()
    );
    // Change amount
    assert_eq!(response.log.get(2).unwrap().value, "0".to_string());
    // Number of entry
    assert_eq!(response.log.get(3).unwrap().value, "1".to_string());
}

#[test]
fn lock_below_segment_fail() {
    let mut deps = get_deps_after_init(vec![
        TokenSettings {
            token_type: TokenType::NativeToken {
                denom: "uscrt".to_string(),
            },
            segment: Uint128(25_000_000_000_u128),
            bounding_period: 0,
        },
        TokenSettings {
            token_type: TokenType::CustomToken {
                contract_addr: HumanAddr::from("snip20-token-1"),
                token_code_hash: "hash".to_string(),
            },
            segment: Uint128(10_000_000_000_u128),
            bounding_period: 0,
        },
    ]);

    let env = mock_env("account-1", &[Coin::new(20_000_000_000_u128, "uscrt")]);

    let response = handle(
        &mut deps,
        env,
        HandleMsg::Lock {
            amount: Uint128(20_000_000_000_u128),
        },
    );

    assert_eq!(
        response,
        Err(StdError::generic_err(
            "Amount is lower then the minimum segment amount of 25000000000"
        ))
    );
}

#[test]
fn lock_with_change_success() {
    let mut deps = get_deps_after_init(vec![
        TokenSettings {
            token_type: TokenType::NativeToken {
                denom: "uscrt".to_string(),
            },
            segment: Uint128(25_000_000_000_u128),
            bounding_period: 0,
        },
        TokenSettings {
            token_type: TokenType::CustomToken {
                contract_addr: HumanAddr::from("snip20-token-1"),
                token_code_hash: "hash".to_string(),
            },
            segment: Uint128(10_000_000_000_u128),
            bounding_period: 0,
        },
    ]);

    let env = mock_env("account-1", &[Coin::new(26_000_000_000_u128, "uscrt")]);

    let response = handle(
        &mut deps,
        env,
        HandleMsg::Lock {
            amount: Uint128(26_000_000_000_u128),
        },
    )
    .unwrap();

    // Has one msg that sends the change amount back
    assert_eq!(response.messages.len(), 1);
    // Locked amount
    assert_eq!(
        response.log.get(1).unwrap().value,
        "25000000000".to_string()
    );
    // Change amount
    assert_eq!(response.log.get(2).unwrap().value, "1000000000".to_string());
    // Number of entry
    assert_eq!(response.log.get(3).unwrap().value, "1".to_string());
}

#[test]
fn lock_non_native_token_success() {
    let mut deps = get_deps_after_init(vec![
        TokenSettings {
            token_type: TokenType::NativeToken {
                denom: "uscrt".to_string(),
            },
            segment: Uint128(25_000_000_000_u128),
            bounding_period: 0,
        },
        TokenSettings {
            token_type: TokenType::CustomToken {
                contract_addr: HumanAddr::from("snip20-token-1"),
                token_code_hash: "hash".to_string(),
            },
            segment: Uint128(10_000_000_000_u128),
            bounding_period: 0,
        },
    ]);

    let env = mock_env("account-1", &[Coin::new(25_000_000_000_u128, "uscrt")]);
    let env_token = mock_env("snip20-token-1", &[]);

    let response = handle(
        &mut deps,
        env_token,
        HandleMsg::Receive {
            amount: Uint128(25_000_000_000_u128),
            msg: Some(to_binary(&ReceiverCallbackMsg::Lock {}).unwrap()),
            from: env.message.sender.clone(),
        },
    )
    .unwrap();

    // Has one msg that sends the change amount back
    assert_eq!(response.messages.len(), 1);
    // Locked amount
    assert_eq!(
        response.log.get(1).unwrap().value,
        "20000000000".to_string()
    );
    // Change amount
    assert_eq!(response.log.get(2).unwrap().value, "5000000000".to_string());
    // Number of entry
    assert_eq!(response.log.get(3).unwrap().value, "2".to_string());

    let response = query(&deps, QueryMsg::LaunchpadInfo);

    println!("{:?}", response);
}

#[test]
fn lock_token_with_bounding_period_and_check_if_draw_works() {
    let mut deps = get_deps_after_init(vec![TokenSettings {
        token_type: TokenType::NativeToken {
            denom: "uscrt".to_string(),
        },
        segment: Uint128(25_000_000_000_u128),
        bounding_period: 24 * 60 * 60, // One day bounding period
    }]);

    for n in 0..4 {
        let env = mock_env(
            format!("account-{}", n),
            &[Coin::new((n + 1) * 25_000_000_000 as u128, "uscrt")],
        );

        let response = handle(
            &mut deps,
            env,
            HandleMsg::Lock {
                amount: Uint128((n + 1) * 25_000_000_000 as u128),
            },
        )
        .unwrap();

        // No change left
        assert_eq!(response.messages.len(), 0);
        // Locked amount
        assert_eq!(
            response.log.get(1).unwrap().value,
            ((n + 1) * 25_000_000_000).to_string()
        );
        // Change amount
        assert_eq!(response.log.get(2).unwrap().value, "0".to_string());
        // Number of entry
        assert_eq!(response.log.get(3).unwrap().value, (n + 1).to_string());
    }

    let env = mock_env("dummy-ido-contract", &[]);
    let response = query(
        &deps,
        QueryMsg::Draw {
            tokens: vec![None],
            number: 4,
            timestamp: env.block.time,
        },
    )
    .unwrap();
    let response: QueryResponse = from_binary(&response).unwrap();

    match response {
        QueryResponse::DrawnAddresses(addresses) => {
            assert_eq!(addresses.len(), 0_usize);
        }
        _ => {
            panic!("Wrong type of response returned when trying to draw addresses")
        }
    };

    let mut env = mock_env("dummy-ido-contract", &[]);
    env.block.time = env.block.time + (24 * 60 * 60) + 1;

    let response = query(
        &deps,
        QueryMsg::Draw {
            tokens: vec![None],
            number: 4,
            timestamp: env.block.time,
        },
    )
    .unwrap();
    let response: QueryResponse = from_binary(&response).unwrap();

    let addresses = match response {
        QueryResponse::DrawnAddresses(addresses) => addresses,
        _ => panic!("Wrong type of response returned when trying to draw addresses"),
    };

    assert!(addresses
        .iter()
        .position(|a| a == &HumanAddr::from("account-0"))
        .is_some());
    assert!(addresses
        .iter()
        .position(|a| a == &HumanAddr::from("account-1"))
        .is_some());
    assert!(addresses
        .iter()
        .position(|a| a == &HumanAddr::from("account-2"))
        .is_some());
    assert!(addresses
        .iter()
        .position(|a| a == &HumanAddr::from("account-3"))
        .is_some());
}
