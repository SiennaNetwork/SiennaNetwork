use amm_shared::{
    fadroma as fadroma,
    msg::exchange::HandleMsg as ExchangeHandle,
    msg::factory::{HandleMsg, InitMsg, QueryMsg, QueryResponse},
    Pagination, TokenPair, TokenType,
    Exchange, ExchangeSettings, Fee
};
use fadroma::{
    platform::{
        from_binary,
        testing::{mock_dependencies, mock_env},
        to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, Querier,
        StdError, StdResult, Storage, WasmMsg,
        Canonize,
        {ContractInstantiationInfo, ContractLink},
    },
    killswitch,
    storage::{load, save},
};

use crate::{contract::*, state::*};

impl Into<InitMsg> for &Config<HumanAddr> {
    fn into(self) -> InitMsg {
        InitMsg {
            lp_token_contract: self.lp_token_contract.clone(),
            pair_contract: self.pair_contract.clone(),
            exchange_settings: self.exchange_settings.clone(),
            admin: None,
            prng_seed: to_binary(&"prng").unwrap(),
        }
    }
}
impl Into<HandleMsg> for &Config<HumanAddr> {
    fn into(self) -> HandleMsg {
        HandleMsg::SetConfig {
            lp_token_contract: Some(self.lp_token_contract.clone()),
            pair_contract: Some(self.pair_contract.clone()),
            exchange_settings: Some(self.exchange_settings.clone()),
        }
    }
}
impl Into<QueryResponse> for &Config<HumanAddr> {
    fn into(self) -> QueryResponse {
        QueryResponse::Config {
            lp_token_contract: self.lp_token_contract.clone(),
            pair_contract: self.pair_contract.clone(),
            exchange_settings: self.exchange_settings.clone(),
        }
    }
}

fn mkenv(sender: impl Into<HumanAddr>) -> Env {
    mock_env(sender, &[])
}

fn mkdeps() -> Extern<impl Storage, impl Api, impl Querier> {
    mock_dependencies(30, &[])
}

fn mkconfig(id: u64) -> Config<HumanAddr> {
    Config {
        lp_token_contract: ContractInstantiationInfo {
            id,
            code_hash: "lptoken".into(),
        },
        pair_contract: ContractInstantiationInfo {
            id,
            code_hash: "2341586789".into(),
        },
        exchange_settings: ExchangeSettings {
            swap_fee: Fee::new(28, 10000),
            sienna_fee: Fee::new(2, 10000),
            sienna_burner: None,
        },
    }
}

fn assert_unauthorized(response: StdResult<HandleResponse>) {
    let err = response.unwrap_err();
    assert_eq!(err, StdError::unauthorized())
}

fn pagination(start: u64, limit: u8) -> Pagination {
    Pagination { start, limit }
}

mod test_contract {
    use super::*;

    #[test]
    fn ok_init() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let env = mkenv("admin");
        let config = mkconfig(0);
        assert!(init(deps, env, (&config).into()).is_ok());
        assert_eq!(config, load_config(deps)?);
        assert_eq!(
            load_prng_seed(&deps.storage).unwrap(),
            to_binary("prng").unwrap()
        );

        Ok(())
    }

    #[test]
    fn ok_get_set_config() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let config1 = mkconfig(1);
        let env = mkenv("admin");
        // init with some config
        assert!(init(deps, env.clone(), (&config1).into()).is_ok());
        // get current config
        let response: QueryResponse = from_binary(&query(deps, QueryMsg::GetConfig {})?)?;
        assert_eq!(response, (&config1).into());
        // set config to something else
        let config2 = mkconfig(2);
        assert!(handle(deps, env, (&config2).into()).is_ok());
        // updated config is returned
        let response: QueryResponse = from_binary(&query(deps, QueryMsg::GetConfig {})?)?;
        assert_eq!(response, (&config2).into());
        Ok(())
    }

    #[test]
    fn no_unauthorized_set_config() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let config1 = mkconfig(1);
        let env = mkenv("admin");
        // init with some config
        assert!(init(deps, env.clone(), (&config1).into()).is_ok());
        // someone else tries to set config
        let config2 = mkconfig(2);
        let env = mkenv("badman");
        assert!(handle(deps, env, (&config2).into()).is_err());
        // config remains unchanged
        let response: QueryResponse = from_binary(&query(deps, QueryMsg::GetConfig {})?)?;
        assert_eq!(response, (&config1).into());
        Ok(())
    }

    #[test]
    fn create_exchange_for_the_same_tokens_returns_error() -> StdResult<()> {
        fn assert_create_error(pair: TokenPair<HumanAddr>) {
            let ref mut deps = mkdeps();
            let result = create_exchange(deps, mkenv("sender"), pair, to_binary(&"entropy").unwrap());

            let error: StdError = result.unwrap_err();
    
            match error {
                StdError::GenericErr { msg, .. } => {
                    assert_eq!(msg, "Cannot create an exchange with the same token.");
                }
                _ => panic!("Expected StdError::GenericErr"),
            };
        }

        assert_create_error(TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
        ));

        assert_create_error(TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "c1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "C1dc8261059fee1de9f1873cd1359ccd7a6bc5623772661fa3d55332eb652084".into(),
            },
        ));

        assert_create_error(TokenPair(
            TokenType::NativeToken {
                denom: "test1".into(),
            },
            TokenType::NativeToken {
                denom: "test1".into(),
            },
        ));

        Ok(())
    }

    #[test]
    fn test_register_exchange() -> StdResult<()> {
        let ref mut deps = mkdeps();

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
            TokenType::NativeToken {
                denom: "test1".into(),
            },
        );

        let sender_addr = HumanAddr("sender1111".into());

        let result = handle(
            deps,
            mkenv(sender_addr.clone()),
            HandleMsg::RegisterExchange {
                pair: pair.clone(),
                signature: to_binary("whatever")?,
            },
        );

        assert_unauthorized(result);

        let config = mkconfig(0);
        save_config(deps, config)?;

        let env = mkenv(sender_addr.clone());

        let signature = create_signature(&env)?;
        save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

        handle(
            deps,
            env,
            HandleMsg::RegisterExchange {
                pair: pair.clone(),
                signature,
            },
        )?;

        //Ensure that the ephemeral storage is empty after the message
        let result: Option<Binary> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
        match result {
            None => {}
            _ => panic!("Ephemeral storage should be empty!"),
        }

        Ok(())
    }

    #[test]
    fn test_register_exchange_with_empty_signature() -> StdResult<()> {
        let ref mut deps = mkdeps();

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
            TokenType::NativeToken {
                denom: "test1".into(),
            },
        );

        let sender_addr = HumanAddr("sender1111".into());

        let result = handle(
            deps,
            mkenv(sender_addr.clone()),
            HandleMsg::RegisterExchange {
                pair: pair.clone(),
                signature: Binary(vec![]),
            },
        );

        assert_unauthorized(result);

        Ok(())
    }

    #[test]
    fn query_exchange() -> StdResult<()> {
        let ref mut deps = mkdeps();

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
            TokenType::NativeToken {
                denom: "test1".into(),
            },
        );

        let config = mkconfig(0);
        save_config(deps, config)?;

        let sender_addr = HumanAddr("sender1111".into());
        let env = mkenv(sender_addr.clone());

        let signature = create_signature(&env)?;
        save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

        handle(
            deps,
            env,
            HandleMsg::RegisterExchange {
                pair: pair.clone(),
                signature,
            },
        )
        .unwrap();

        let result = query(deps, QueryMsg::GetExchangeAddress { pair: pair.clone() })?;
        let response: QueryResponse = from_binary(&result)?;

        match response {
            QueryResponse::GetExchangeAddress { address } => assert_eq!(sender_addr, address),
            _ => {
                return Err(StdError::generic_err(
                    "Wrong response. Expected: QueryResponse::GetExchangeAddress.",
                ))
            }
        };
        Ok(())
    }

    #[test]
    fn test_transfer_exchanges() {
        let ref mut deps = mkdeps();
        let admin = "admin";

        let config = mkconfig(0);

        init(deps, mkenv(admin), (&config).into()).unwrap();

        let over_limit = 5;
        let mut exchanges = mock_and_store_exchanges(deps, TRANSFER_LIMIT + over_limit);

        let new_instance = ContractLink {
            address: HumanAddr("new_factory".into()),
            code_hash: "new_factory_code_hash".into(),
        };

        let result = handle(
            deps,
            mkenv("rando"),
            HandleMsg::TransferExchanges {
                new_instance: new_instance.clone(),
                skip: None,
            },
        );
        assert_unauthorized(result);

        let mut result = handle(
            deps,
            mkenv(admin),
            HandleMsg::TransferExchanges {
                new_instance: new_instance.clone(),
                skip: None,
            },
        )
        .unwrap();
        assert_eq!(result.messages.len(), TRANSFER_LIMIT + 1); // +1 for the message to the new factory

        let exchanges_left = get_exchanges(deps, pagination(0, 30)).unwrap();
        assert_eq!(exchanges_left.len(), over_limit);
        assert_eq!(exchanges_left, exchanges[..over_limit]);

        let last_msg = result.messages.pop().unwrap();
        assert_eq!(
            last_msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: new_instance.address.clone(),
                callback_code_hash: new_instance.code_hash.clone(),
                send: vec![],
                msg: to_binary(&HandleMsg::ReceiveExchanges {
                    finalize: false,
                    exchanges: exchanges
                        .clone()
                        .into_iter()
                        .rev()
                        .take(TRANSFER_LIMIT)
                        .collect()
                })
                .unwrap()
            })
        );

        for (message, exchange) in result.messages.into_iter().zip(
            exchanges
                .drain(over_limit..)
                .into_iter()
                .rev()
                .take(TRANSFER_LIMIT),
        ) {
            assert_eq!(
                message,
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: exchange.contract.address,
                    callback_code_hash: exchange.contract.code_hash,
                    send: vec![],
                    msg: to_binary(&ExchangeHandle::ChangeFactory {
                        contract: new_instance.clone()
                    })
                    .unwrap()
                })
            )
        }

        let mut result = handle(
            deps,
            mkenv(admin),
            HandleMsg::TransferExchanges {
                new_instance: new_instance.clone(),
                skip: None,
            },
        )
        .unwrap();
        assert_eq!(result.messages.len(), over_limit + 1); // +1 for the message to the new factory

        let exchanges_left = get_exchanges(deps, pagination(0, 30)).unwrap();
        assert_eq!(exchanges_left.len(), 0);

        let last_msg = result.messages.pop().unwrap();
        assert_eq!(
            last_msg,
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: new_instance.address.clone(),
                callback_code_hash: new_instance.code_hash.clone(),
                send: vec![],
                msg: to_binary(&HandleMsg::ReceiveExchanges {
                    finalize: true,
                    exchanges: exchanges.clone().into_iter().rev().collect()
                })
                .unwrap()
            })
        );

        for (message, exchange) in result
            .messages
            .into_iter()
            .zip(exchanges.into_iter().rev().take(TRANSFER_LIMIT))
        {
            assert_eq!(
                message,
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: exchange.contract.address,
                    callback_code_hash: exchange.contract.code_hash,
                    send: vec![],
                    msg: to_binary(&ExchangeHandle::ChangeFactory {
                        contract: new_instance.clone()
                    })
                    .unwrap()
                })
            )
        }

        let status = killswitch::get_status(deps).unwrap();
        assert_eq!(
            status,
            killswitch::ContractStatus {
                level: killswitch::ContractStatusLevel::Migrating,
                reason: "Migrating to new factory.".into(),
                new_address: Some(new_instance.address)
            }
        );
    }

    #[test]
    fn test_transfer_exchanges_with_skip() {
        let ref mut deps = mkdeps();
        let admin = "admin";

        let config = mkconfig(0);

        init(deps, mkenv(admin), (&config).into()).unwrap();

        let mut exchanges = mock_and_store_exchanges(deps, 5);

        let new_instance = ContractLink {
            address: HumanAddr("new_factory".into()),
            code_hash: "new_factory_code_hash".into(),
        };

        let mut result = handle(
            deps,
            mkenv(admin),
            HandleMsg::TransferExchanges {
                new_instance: new_instance.clone(),
                skip: Some(vec![
                    exchanges[0].contract.address.clone(),
                    exchanges[2].contract.address.clone(),
                ]),
            },
        )
        .unwrap();
        assert_eq!(result.messages.len(), 4);

        result.messages.pop();

        let exchanges_left = get_exchanges(deps, pagination(0, 30)).unwrap();
        assert_eq!(exchanges_left.len(), 0);

        exchanges.remove(0);
        exchanges.remove(1);

        for (message, exchange) in result
            .messages
            .into_iter()
            .zip(exchanges.into_iter().rev().take(TRANSFER_LIMIT))
        {
            assert_eq!(
                message,
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: exchange.contract.address,
                    callback_code_hash: exchange.contract.code_hash,
                    send: vec![],
                    msg: to_binary(&ExchangeHandle::ChangeFactory {
                        contract: new_instance.clone()
                    })
                    .unwrap()
                })
            )
        }
    }

    #[test]
    fn test_receive_exchanges() {
        let ref mut deps = mkdeps();
        let admin = "admin";

        let config = mkconfig(0);

        init(deps, mkenv(admin), (&config).into()).unwrap();

        let mut existing_exchanges = mock_and_store_exchanges(deps, 2);

        let address = HumanAddr::from("new_factory");

        let result = handle(
            deps,
            mkenv("rando"),
            HandleMsg::SetMigrationAddress {
                address: address.clone(),
            }
        );
        assert_unauthorized(result);

        let result = handle(
            deps,
            mkenv("rando"),
            HandleMsg::ReceiveExchanges {
                finalize: false,
                exchanges: vec![],
            },
        );
        assert_unauthorized(result);

        handle(
            deps,
            mkenv(admin),
            HandleMsg::SetMigrationAddress {
                address: address.clone(),
            }
        )
        .unwrap();

        let mut new_exchanges = vec![];

        for i in 2..5 {
            let pair = TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr(format!("addr_{}", i)),
                    token_code_hash: format!("code_hash_{}", i),
                },
                TokenType::NativeToken {
                    denom: format!("denom_{}", i),
                },
            );
            let address = HumanAddr(format!("address_{}", i));
            let code_hash = format!("code_hash_{}", i);
            new_exchanges.push(Exchange {
                pair,
                contract: ContractLink { address, code_hash },
            });
        }

        handle(
            deps,
            mkenv(address.clone()),
            HandleMsg::ReceiveExchanges {
                finalize: false,
                exchanges: new_exchanges.clone(),
            },
        )
        .unwrap();

        assert!(load_migration_address(&deps.storage).is_ok());

        let stored_exchanges = get_exchanges(deps, pagination(0, 30)).unwrap();
        existing_exchanges.extend(new_exchanges.into_iter());

        assert_eq!(stored_exchanges, existing_exchanges);

        let new_exchange = Exchange {
            pair: TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr("addr_5".into()),
                    token_code_hash: "code_hash_5".into(),
                },
                TokenType::NativeToken {
                    denom: "denom_5".into(),
                },
            ),
            contract: ContractLink {
                address: HumanAddr("address_5".into()),
                code_hash: "code_hash_5".into(),
            },
        };

        handle(
            deps,
            mkenv(address),
            HandleMsg::ReceiveExchanges {
                finalize: true,
                exchanges: vec![new_exchange.clone()],
            },
        )
        .unwrap();

        assert!(load_migration_address(&deps.storage).is_err());

        let stored_exchanges = get_exchanges(deps, pagination(0, 30)).unwrap();
        existing_exchanges.push(new_exchange);

        assert_eq!(stored_exchanges, existing_exchanges);
    }
}

mod test_state {
    use super::*;

    fn swap_pair<A: Clone>(pair: &TokenPair<A>) -> TokenPair<A> {
        TokenPair(pair.1.clone(), pair.0.clone())
    }

    #[test]
    fn generates_the_same_key_for_uppercase_denom() {
        let ref deps = mkdeps();

        let pair = TokenPair(
            TokenType::NativeToken {
                denom: "test".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("address".into()),
                token_code_hash: "asd21312asd".into(),
            },
        );

        let key_0 = generate_pair_key(pair.canonize(&deps.api).unwrap());

        let pair = TokenPair(
            TokenType::NativeToken {
                denom: "TeST".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("address".into()),
                token_code_hash: "asd21312asd".into(),
            },
        );

        let key_1 = generate_pair_key(pair.canonize(&deps.api).unwrap());

        assert_eq!(key_0, key_1);
    }

    #[test]
    fn generates_the_same_key_for_swapped_pairs() -> StdResult<()> {
        fn cmp_pair<S: Storage, A: Api, Q: Querier>(
            deps: &Extern<S, A, Q>,
            pair: TokenPair<HumanAddr>,
        ) -> StdResult<()> {
            let stored_pair = pair.clone().canonize(&deps.api)?;
            let key = generate_pair_key(stored_pair);

            let pair = swap_pair(&pair);

            let stored_pair = pair.canonize(&deps.api)?;
            let swapped_key = generate_pair_key(stored_pair);

            assert_eq!(key, swapped_key);

            Ok(())
        }

        let ref deps = mkdeps();

        cmp_pair(
            deps,
            TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into(),
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into(),
                },
            ),
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test1".into(),
                },
                TokenType::NativeToken {
                    denom: "test2".into(),
                },
            ),
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test3".into(),
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("third_addr".into()),
                    token_code_hash: "asd21312asd".into(),
                },
            ),
        )?;

        Ok(())
    }

    #[test]
    fn query_correct_exchange_info() -> StdResult<()> {
        let mut deps = mkdeps();

        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("first_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("scnd_addr".into()),
                token_code_hash: "4534qwerqqw".into(),
            },
        );

        let address = HumanAddr("ctrct_addr".into());

        store_exchanges(
            &mut deps,
            vec![Exchange {
                pair: pair.clone(),
                contract: ContractLink {
                    address: address.clone(),
                    code_hash: "code_hash".into(),
                },
            }],
        )?;

        let retrieved_address = get_address_for_pair(&deps, pair.clone())?;

        assert!(pair_exists(&mut deps, pair)?);
        assert_eq!(address, retrieved_address);

        Ok(())
    }

    #[test]
    fn only_one_exchange_per_factory() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr("first_addr".into()),
                token_code_hash: "13123adasd".into(),
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("scnd_addr".into()),
                token_code_hash: "4534qwerqqw".into(),
            },
        );

        store_exchanges(
            deps,
            vec![Exchange {
                pair: pair.clone(),
                contract: ContractLink {
                    address: "first_addr".into(),
                    code_hash: "first_code_hash".into(),
                },
            }],
        )?;

        let swapped = swap_pair(&pair);

        match store_exchanges(
            deps,
            vec![Exchange {
                pair: swapped,
                contract: ContractLink {
                    address: "other_addr".into(),
                    code_hash: "other_code_hash".into(),
                },
            }],
        ) {
            Ok(_) => Err(StdError::generic_err("Exchange already exists")),
            Err(_) => Ok(()),
        }
    }

    #[test]
    fn test_get_exchanges() -> StdResult<()> {
        let ref mut deps = mkdeps();
        let exchanges = mock_and_store_exchanges(deps, 33);

        let result = get_exchanges(deps, pagination(exchanges.len() as u64, 20))?;
        assert_eq!(result.len(), 0);

        let result = get_exchanges(deps, pagination((exchanges.len() - 1) as u64, 20))?;
        assert_eq!(result.len(), 1);

        let result = get_exchanges(deps, pagination(0, 1))?;
        assert_eq!(result.len(), 1);

        let result = get_exchanges(deps, pagination(0, PAGINATION_LIMIT + 10))?;
        assert_eq!(result.len(), PAGINATION_LIMIT as usize);

        let result = get_exchanges(deps, pagination(3, PAGINATION_LIMIT))?;
        assert_eq!(result, exchanges[3..]);

        Ok(())
    }
}

fn mock_and_store_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    count: usize,
) -> Vec<Exchange<HumanAddr>> {
    let mut exchanges = vec![];

    for i in 0..count {
        let pair = TokenPair(
            TokenType::CustomToken {
                contract_addr: HumanAddr(format!("addr_{}", i)),
                token_code_hash: format!("code_hash_{}", i),
            },
            TokenType::NativeToken {
                denom: format!("denom_{}", i),
            },
        );
        let address = HumanAddr(format!("address_{}", i));
        let code_hash = format!("code_hash_{}", i);

        let exchange = Exchange {
            pair,
            contract: ContractLink { address, code_hash },
        };

        store_exchanges(deps, vec![exchange.clone()]).unwrap();
        exchanges.push(exchange);
    }

    exchanges
}
