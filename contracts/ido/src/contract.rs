use amm_shared::{
    admin::admin::{admin_handle, admin_query, save_admin, DefaultHandleImpl, DefaultQueryImpl},
    auth::{auth_handle, AuthHandleMsg, DefaultHandleImpl as AuthHandle},
    fadroma::scrt::{
        addr::Canonize,
        callback::ContractInstance,
        cosmwasm_std::{
            to_binary, Api, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse,
            InitResponse, Querier, QueryRequest, QueryResult, StdError, StdResult,
            Storage, WasmMsg, WasmQuery,
        },
        migrate as fadroma_scrt_migrate,
        storage::Storable,
        toolkit::snip20,
        utils::viewing_key::ViewingKey,
        BLOCK_SIZE,
    },
    msg::ido::{HandleMsg, InitMsg, QueryMsg},
    msg::launchpad::QueryMsg as LaunchpadQueryMsg,
    msg::launchpad::QueryResponse as LaunchpadQueryResponse,
    TokenType,
};
use fadroma_scrt_migrate::{get_status, with_status};

use crate::data::{save_contract_address, save_viewing_key, Account, Config, SwapConstants};

use crate::helpers::*;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg
        .info
        .min_allocation
        .u128()
        .checked_mul(msg.info.max_seats as u128)
        .is_none()
    {
        return Err(StdError::generic_err(
            "The total amount required for the sale is too big.",
        ));
    }

    save_contract_address(deps, &env.contract.address)?;

    let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice());
    save_viewing_key(&mut deps.storage, &viewing_key)?;

    let mut messages = vec![
        // Set viewing key from IDO contract onto the sold token contract
        // so that we can query the balance of the sold token transfered
        // to the IDO contract
        snip20::set_viewing_key_msg(
            viewing_key.to_string(),
            None,
            BLOCK_SIZE,
            msg.info.sold_token.code_hash.clone(),
            msg.info.sold_token.address.clone(),
        )?,
        // Register this contract as a receiver of the callback messages
        // from the buy token so we can trigger back start of the sale after
        // the required amount of funds has been sent to this contract
        snip20::register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            BLOCK_SIZE,
            msg.info.sold_token.code_hash.clone(),
            msg.info.sold_token.address.clone(),
        )?,
    ];

    let input_token_decimals = match &msg.info.input_token {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            // Set viewing key from IDO contract onto the custom input token
            // so we can query the balance later
            messages.push(snip20::set_viewing_key_msg(
                viewing_key.to_string(),
                None,
                BLOCK_SIZE,
                token_code_hash.clone(),
                contract_addr.clone(),
            )?);

            // Register this contract as a receiver of the callback messages
            // from the custom input token. This will allow us to receive
            // message after the tokens have been sent and will make the swap
            // happen in a single transaction
            messages.push(snip20::register_receive_msg(
                env.contract_code_hash.clone(),
                None,
                BLOCK_SIZE,
                token_code_hash.clone(),
                contract_addr.clone(),
            )?);

            // Update the token decimals based on the custom token number of decimals
            get_token_decimals(
                &deps.querier,
                ContractInstance {
                    address: contract_addr.clone(),
                    code_hash: token_code_hash.clone(),
                },
            )?
        }
        TokenType::NativeToken { .. } => 6,
    };

    // Execute the HandleMsg::RegisterIdo method of
    // the factory contract in order to register this address
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.callback.contract.address,
        callback_code_hash: msg.callback.contract.code_hash,
        msg: msg.callback.msg,
        send: vec![],
    }));

    let mut taken_seats = msg.info.whitelist.len() as u32;

    for address in msg.info.whitelist {
        Account::<CanonicalAddr>::new(&address.canonize(&deps.api)?).save(deps)?;
    }

    // Call the launchpad contract and request whitelist addresses
    if taken_seats < msg.info.max_seats {
        if let Some(request) = &msg.launchpad {
            let response: LaunchpadQueryResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: request.launchpad.address.clone(),
                    callback_code_hash: request.launchpad.code_hash.clone(),
                    msg: to_binary(&LaunchpadQueryMsg::Draw {
                        tokens: request.tokens.clone(),
                        number: (msg.info.max_seats - taken_seats) as u32,
                        timestamp: env.block.time,
                    })?,
                }))?;

            match response {
                LaunchpadQueryResponse::DrawnAddresses(addresses) => {
                    for address in addresses {
                        Account::<CanonicalAddr>::new(&address.canonize(&deps.api)?).save(deps)?;
                        taken_seats = taken_seats + 1;
                    }
                }
                _ => {
                    return Err(StdError::generic_err(
                        "QueryResponse from Launchpad return unexpected result",
                    ));
                }
            };
        }
    }

    let config = Config {
        input_token: msg.info.input_token,
        sold_token: msg.info.sold_token.clone(),
        swap_constants: SwapConstants {
            sold_token_decimals: get_token_decimals(&deps.querier, msg.info.sold_token)?,
            rate: msg.info.rate,
            input_token_decimals,
        },
        taken_seats,
        max_seats: msg.info.max_seats,
        max_allocation: msg.info.max_allocation,
        min_allocation: msg.info.min_allocation,
        sale_type: msg.info.sale_type.unwrap_or_default(),
        launchpad: msg.launchpad.map(|l| l.launchpad),
        // Configured when activating
        schedule: None,
    };

    save_admin(deps, &msg.admin)?;
    config.save(deps)?;

    Ok(InitResponse {
        messages,
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    with_status!(
        deps,
        env,
        match msg {
            HandleMsg::Receive {
                from, amount, msg, ..
            } => crate::handle::receive_callback(deps, env, from, amount, msg),
            HandleMsg::PreLock { amount } => {
                // Can be called directly only when the input token is SCRT
                let config = Config::<CanonicalAddr>::load_self(&deps)?;

                if !config.input_token.is_native_token() {
                    return Err(StdError::generic_err(
                        "Use the SNIP20 receiver interface instead.",
                    ));
                }

                config
                    .input_token
                    .assert_sent_native_token_balance(&env, amount)?;

                crate::handle::pre_lock(deps, env.block.time, config, amount, env.message.sender)
            }
            HandleMsg::Swap { amount, recipient } => {
                // Can be called directly only when the input token is SCRT
                let config = Config::<CanonicalAddr>::load_self(&deps)?;

                if !config.input_token.is_native_token() {
                    return Err(StdError::generic_err(
                        "Use the SNIP20 receiver interface instead.",
                    ));
                }

                config
                    .input_token
                    .assert_sent_native_token_balance(&env, amount)?;

                crate::handle::swap(
                    deps,
                    env.block.time,
                    config,
                    amount,
                    env.message.sender,
                    recipient,
                )
            }
            HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultHandleImpl),
            HandleMsg::AdminRefund { address } => crate::handle::refund(deps, env, address),
            HandleMsg::AdminClaim { address } => crate::handle::claim(deps, env, address),
            HandleMsg::AdminAddAddresses { addresses } =>
                crate::handle::add_addresses(deps, env, addresses),
            HandleMsg::CreateViewingKey { entropy, padding } => {
                let msg = AuthHandleMsg::CreateViewingKey { entropy, padding };
                auth_handle(deps, env, msg, AuthHandle)
            }
            HandleMsg::SetViewingKey { key, padding } => {
                let msg = AuthHandleMsg::SetViewingKey { key, padding };
                auth_handle(deps, env, msg, AuthHandle)
            }
        }
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::EligibilityInfo { address } => crate::query::get_eligibility_info(deps, address),
        QueryMsg::SaleInfo => crate::query::get_sale_info(deps),
        QueryMsg::SaleStatus => crate::query::get_sale_status(deps),
        QueryMsg::Balance { address, key } => crate::query::get_balance(deps, address, key),
        QueryMsg::TokenInfo {} => crate::query::get_token_info(deps),
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amm_shared::{
        fadroma::scrt::callback::Callback,
        fadroma::scrt::cosmwasm_std::{
            from_binary,
            testing::{mock_env, MockApi, MockStorage},
            Binary, Coin, Env, Extern, HumanAddr, Uint128,
        },
        msg::ido::{
            HandleMsg, InitMsg, QueryMsg, QueryResponse, ReceiverCallbackMsg, TokenSaleConfig,
        },
        querier::{MockContractInstance, MockQuerier},
        TokenType,
    };

    const BLOCK_TIME: u64 = 1_571_797_419;
    const RATE: Uint128 = Uint128(1_u128);
    const MIN_ALLOCATION: Uint128 = Uint128(100_u128);
    const MAX_ALLOCATION: Uint128 = Uint128(500_u128);

    fn internal_mock_deps(
        len: usize,
        balance: &[Coin],
    ) -> Extern<MockStorage, MockApi, MockQuerier> {
        let contract_addr = HumanAddr::from("mock-address");
        Extern {
            storage: MockStorage::default(),
            api: MockApi::new(len),
            querier: MockQuerier::new(
                &[(&contract_addr, balance)],
                vec![MockContractInstance {
                    instance: ContractInstance {
                        address: HumanAddr::from("sold-token"),
                        code_hash: "".to_string(),
                    },
                    token_decimals: 18,
                    token_supply: Uint128::from(2500_u128),
                }],
            ),
        }
    }

    /// Get init message for initialization of the token.
    fn get_init(sold_token: Option<ContractInstance<HumanAddr>>, admin: &HumanAddr) -> InitMsg {
        let sold_token = sold_token.unwrap_or_else(|| ContractInstance::<HumanAddr> {
            address: HumanAddr::from("sold-token"),
            code_hash: "".to_string(),
        });

        InitMsg {
            info: TokenSaleConfig {
                input_token: TokenType::NativeToken {
                    denom: "uscrt".to_string(),
                },
                rate: RATE,
                sold_token,
                whitelist: vec![
                    HumanAddr::from("buyer-1"),
                    HumanAddr::from("buyer-2"),
                    HumanAddr::from("buyer-3"),
                    HumanAddr::from("buyer-4"),
                ],
                max_seats: 5,
                max_allocation: MAX_ALLOCATION,
                min_allocation: MIN_ALLOCATION,
                sale_type: None,
            },
            launchpad: None,
            prng_seed: to_binary(&"whatever").unwrap(),
            entropy: to_binary(&"whatever").unwrap(),
            admin: admin.clone(),
            callback: Callback {
                msg: Binary::from(&[]),
                contract: ContractInstance {
                    address: HumanAddr::from("callback-address"),
                    code_hash: "code-hash-of-callback-contract".to_string(),
                },
            },
        }
    }

    fn init_contract(
        start_time: Option<u64>,
        end_time: Option<u64>,
    ) -> (Extern<MockStorage, MockApi, MockQuerier>, Env) {
        let mut deps = internal_mock_deps(123, &[]);
        let env = mock_env("admin", &[]);
        let msg = get_init(None, &env.message.sender);
        init(&mut deps, env.clone(), msg).unwrap();

        let sold_env = mock_env("sold-token", &[]);

        let start_time = start_time.unwrap_or(BLOCK_TIME);
        let end_time = end_time.unwrap_or(start_time + 60);

        handle(
            &mut deps,
            sold_env,
            HandleMsg::Receive {
                from: env.message.sender.clone(),
                amount: Uint128(5 as u128 * MAX_ALLOCATION.u128()),
                msg: Some(
                    to_binary(&ReceiverCallbackMsg::Activate {
                        start_time: Some(start_time),
                        end_time,
                    })
                    .unwrap(),
                ),
            },
        )
        .unwrap();

        (deps, env)
    }

    #[test]
    fn fails_with_init_if_invalid_token() {
        let mut deps = internal_mock_deps(123, &[]);
        let env = mock_env("admin", &[]);
        let msg = get_init(
            Some(ContractInstance::<HumanAddr> {
                address: HumanAddr::from("random-token"),
                code_hash: "".to_string(),
            }),
            &env.message.sender,
        );

        let res = init(&mut deps, env, msg);

        assert_eq!(
            res,
            Err(StdError::generic_err("Error performing TokenInfo query: Generic error: Querier system error: No such contract: random-token"))
        );
    }

    #[test]
    fn test_init_contract() {
        init_contract(None, None);
    }

    #[test]
    #[should_panic]
    fn test_init_contract_same_start_and_end() {
        let start_time = BLOCK_TIME + 1;
        let end_time = start_time;

        init_contract(Some(start_time), Some(end_time));
    }

    #[test]
    #[should_panic]
    fn test_init_contract_end_before_block_time() {
        let start_time = BLOCK_TIME - 200;
        let end_time = BLOCK_TIME - 100;

        init_contract(Some(start_time), Some(end_time));
    }

    #[test]
    fn query_get_info_matches_init() {
        let (deps, _) = init_contract(None, None);
        let res = query(&deps, QueryMsg::SaleInfo).unwrap();
        let res: QueryResponse = from_binary(&res).unwrap();

        match res {
            QueryResponse::SaleInfo {
                input_token,
                sold_token,
                rate,
                taken_seats,
                max_seats,
                min_allocation,
                max_allocation,
                start,
                end,
            } => {
                let config = Config::<CanonicalAddr>::load_self(&deps).unwrap();

                assert_eq!(input_token, config.input_token);
                assert_eq!(sold_token, config.sold_token);
                assert_eq!(rate, config.swap_constants.rate);
                assert_eq!(taken_seats, config.taken_seats);
                assert_eq!(max_seats, config.max_seats);
                assert_eq!(min_allocation, config.min_allocation);
                assert_eq!(max_allocation, config.max_allocation);
                assert_eq!(start.unwrap(), BLOCK_TIME);
                assert_eq!(end.unwrap(), BLOCK_TIME + 60)
            }
            _ => panic!("Expected QueryResponse::GetRate"),
        };
    }

    #[test]
    fn random_address_attempt_swap_gets_error() {
        let (mut deps, _) = init_contract(None, None);
        let env = mock_env("buyer-X", &[Coin::new(10000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(10000_u128),
                recipient: None,
            },
        );

        assert_eq!(
            res,
            Err(StdError::generic_err("This address is not whitelisted."))
        )
    }

    #[test]
    fn buyer_attempt_swap_below_minimum_gets_error() {
        let (mut deps, _) = init_contract(None, None);
        let env = mock_env("buyer-1", &[Coin::new(99_000_000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(99_000_000_u128),
                recipient: None,
            },
        );

        assert_eq!(
            res,
            Err(StdError::generic_err(
                format!(
                    "Insufficient amount provided: the resulting amount fell short of the minimum purchase expected: {}", 
                    MIN_ALLOCATION
                )
            ))
        )
    }

    #[test]
    fn buyer_attempt_swap_above_maximum_gets_error() {
        let (mut deps, _) = init_contract(None, None);
        let env = mock_env("buyer-1", &[Coin::new(501_000_000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(501_000_000_u128),
                recipient: None,
            },
        );

        assert_eq!(
            res,
            Err(StdError::generic_err(format!(
                "This purchase exceeds the total maximum allowed amount for a single address: {}",
                MAX_ALLOCATION
            )))
        )
    }

    #[test]
    fn buyer_swaps_and_views_balance() {
        let (mut deps, _) = init_contract(None, None);
        let buyer = "buyer-1";
        let env = mock_env(buyer, &[Coin::new(250_000_000_u128, "uscrt")]);

        let resp = handle(
            &mut deps,
            env.clone(),
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
                recipient: None,
            },
        )
        .unwrap();

        let bought = resp.log[2].value.parse::<u128>().unwrap();
        let key = String::from("key");

        let resp = query(
            &deps,
            QueryMsg::Balance {
                address: buyer.into(),
                key: key.clone(),
            },
        )
        .unwrap_err();

        assert_eq!(resp, StdError::unauthorized());
        handle(
            &mut deps,
            env,
            HandleMsg::SetViewingKey {
                key: key.clone(),
                padding: None,
            },
        )
        .unwrap();

        let resp = query(
            &deps,
            QueryMsg::Balance {
                address: buyer.into(),
                key,
            },
        )
        .unwrap();
        let resp: QueryResponse = from_binary(&resp).unwrap();

        match resp {
            QueryResponse::Balance {
                pre_lock_amount: _,
                total_bought,
            } => {
                assert_eq!(total_bought.u128(), bought);
            }
            _ => panic!("Expected QueryResponse::Balance"),
        }
    }

    #[test]
    fn buyer_attempt_swap_before_sale_start_gets_error() {
        let start_time = 1_571_797_500;
        let (mut deps, _) = init_contract(Some(start_time), None);
        let env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
                recipient: None,
            },
        );

        assert_eq!(
            res,
            Err(StdError::generic_err(format!(
                "Sale hasn\'t started yet, come back in {} seconds",
                start_time - BLOCK_TIME
            )))
        )
    }

    #[test]
    fn buyer_attempt_swap_after_sale_end_gets_error() {
        let start_time = BLOCK_TIME;
        let end_time = BLOCK_TIME + 100;
        let (mut deps, _) = init_contract(Some(start_time), Some(end_time));
        let mut env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        env.block.time = env.block.time + 200;

        let res = handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
                recipient: None,
            },
        );

        assert_eq!(res, Err(StdError::generic_err("Sale has ended")))
    }

    #[test]
    fn admin_attempt_refund_before_sale_end_gets_error() {
        let start_time = 1_571_797_300;
        let end_time = 1_571_797_500;
        let (mut deps, env) = init_contract(Some(start_time), Some(end_time));

        let res = handle(&mut deps, env, HandleMsg::AdminRefund { address: None });

        assert_eq!(
            res,
            Err(StdError::generic_err(format!(
                "Sale hasn\'t finished yet, come back in {} seconds",
                end_time - BLOCK_TIME
            )))
        );
    }

    #[test]
    fn admin_performs_refund_after_sale_end() {
        let start_time = BLOCK_TIME;
        let end_time = BLOCK_TIME + 1;
        let (mut deps, mut env) = init_contract(Some(start_time), Some(end_time));

        env.block.time = env.block.time + 60;

        let res = handle(
            &mut deps,
            env.clone(),
            HandleMsg::AdminRefund { address: None },
        )
        .unwrap();
        let refunded_amount = &res.log[1].value;

        assert_eq!(refunded_amount, "2500");
    }

    #[test]
    fn admin_attempt_claim_before_sale_end_gets_error() {
        let start_time = 1_571_797_300;
        let end_time = 1_571_797_500;
        let (mut deps, env) = init_contract(Some(start_time), Some(end_time));

        let res = handle(&mut deps, env, HandleMsg::AdminClaim { address: None });

        assert_eq!(
            res,
            Err(StdError::generic_err(format!(
                "Sale hasn\'t finished yet, come back in {} seconds",
                end_time - BLOCK_TIME
            )))
        );
    }

    #[test]
    fn admin_performs_claim_after_sale_end() {
        let start_time = BLOCK_TIME;
        let end_time = BLOCK_TIME + 1;
        let (mut deps, mut env) = init_contract(Some(start_time), Some(end_time));

        env.block.time = env.block.time + 60;

        let res = handle(
            &mut deps,
            env.clone(),
            HandleMsg::AdminClaim { address: None },
        )
        .unwrap();
        let claimed_amount = &res.log[1].value;

        assert_eq!(claimed_amount, "0");
    }

    #[test]
    fn admin_get_status_of_sale() {
        let (mut deps, _) = init_contract(None, None);
        let buyer_env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        handle(
            &mut deps,
            buyer_env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
                recipient: None,
            },
        )
        .unwrap();

        deps.querier
            .sub_balance(Uint128(250 as u128), &HumanAddr::from("sold-token"))
            .unwrap();

        let res = query(&deps, QueryMsg::SaleStatus).unwrap();
        let res: QueryResponse = from_binary(&res).unwrap();

        match res {
            QueryResponse::Status {
                available_for_sale,
                total_allocation,
                ..
            } => {
                assert_eq!(total_allocation, Uint128(2500));
                assert_eq!(available_for_sale, Uint128(2250));
            }
            _ => panic!("Expected QueryResponse::Status"),
        }
    }

    #[test]
    fn admin_attempts_to_add_existing_address_to_whitelist() {
        let (mut deps, env) = init_contract(None, None);
        let buyer_env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            env,
            HandleMsg::AdminAddAddresses {
                addresses: vec![buyer_env.message.sender],
            },
        )
        .unwrap();

        assert_eq!(String::from("0"), res.log[1].value);
    }

    #[test]
    fn admin_adds_new_address_to_whitelist() {
        let (mut deps, env) = init_contract(None, None);
        let buyer_env = mock_env("buyer-5", &[Coin::new(250_000_000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            buyer_env.clone(),
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
                recipient: None,
            },
        );

        assert_eq!(
            res,
            Err(StdError::generic_err("This address is not whitelisted."))
        );

        handle(
            &mut deps,
            env,
            HandleMsg::AdminAddAddresses {
                addresses: vec![buyer_env.message.sender.clone()],
            },
        )
        .unwrap();

        handle(
            &mut deps,
            buyer_env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
                recipient: None,
            },
        )
        .unwrap();
    }

    #[test]
    fn admin_attempts_to_add_more_addresses_that_are_expected_to_be_on_whitelist_gets_error() {
        let (mut deps, env) = init_contract(None, None);
        let buyer_env = mock_env("buyer-5", &[Coin::new(250_000_000_u128, "uscrt")]);
        let buyer_env2 = mock_env("buyer-6", &[Coin::new(250_000_000_u128, "uscrt")]);

        handle(
            &mut deps,
            env.clone(),
            HandleMsg::AdminAddAddresses {
                addresses: vec![buyer_env.message.sender.clone()],
            },
        )
        .unwrap();

        let res = handle(
            &mut deps,
            env,
            HandleMsg::AdminAddAddresses {
                addresses: vec![buyer_env2.message.sender.clone()],
            },
        );

        assert_eq!(
            res,
            Err(StdError::generic_err(
                "Cannot fill more seats then left (0)"
            ))
        );
    }
}
