pub use cosmwasm_std::{
    StdResult, StdError, Extern, Storage, Api, Querier, Env, Binary, to_binary,
    HandleResponse, from_binary, HumanAddr,
    testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage}
};
pub use sienna_amm_shared::{
    Exchange, ExchangeSettings, Fee,
    TokenPair, TokenType,
    Pagination,
    msg::factory::{InitMsg, HandleMsg, QueryMsg, QueryResponse},
};
pub use fadroma_scrt_callback::ContractInstantiationInfo;
pub use fadroma_scrt_storage::{load, save, remove};
pub use crate::{contract::*, state::*};

pub fn create_deps() -> Extern<impl Storage, impl Api, impl Querier> {
    mock_dependencies(10, &[])
}

mod test_contract {
    use super::*;

    fn assert_unauthorized(response: StdResult<HandleResponse>) {
        assert!(response.is_err());

        let err = response.unwrap_err();
        assert_eq!(err, StdError::unauthorized())
    }

    fn config() -> Config<HumanAddr> {
        Config::from_init_msg(InitMsg {
            snip20_contract: ContractInstantiationInfo {
                code_hash: "12355254".into(),
                id: 64
            },
            lp_token_contract: ContractInstantiationInfo {
                code_hash: "23123123".into(),
                id: 64
            },
            pair_contract: ContractInstantiationInfo {
                code_hash: "2341586789".into(),
                id: 33
            },
            ido_contract: ContractInstantiationInfo {
                code_hash: "348534835".into(),
                id: 69
            },
            exchange_settings: ExchangeSettings {
                swap_fee: Fee::new(28, 10000),
                sienna_fee: Fee::new(2, 10000),
                sienna_burner: None
            }
        })
    }

    #[test]
    fn proper_initialization() -> StdResult<()> {
        let ref mut deps = create_deps();

        let config = config();

        let result = init(deps, mock_env("sender1111", &[]), InitMsg {
            snip20_contract: config.snip20_contract.clone(),
            lp_token_contract: config.lp_token_contract.clone(),
            pair_contract: config.pair_contract.clone(),
            ido_contract: config.ido_contract.clone(),
            exchange_settings: config.exchange_settings.clone()
        });

        assert!(result.is_ok());

        let loaded_config = load_config(deps)?;

        assert_eq!(config.snip20_contract, loaded_config.snip20_contract);
        assert_eq!(config.lp_token_contract, loaded_config.lp_token_contract);
        assert_eq!(config.pair_contract, loaded_config.pair_contract);
        assert_eq!(config.ido_contract, loaded_config.ido_contract);
        assert_eq!(config.exchange_settings, loaded_config.exchange_settings);
        
        Ok(())
    }

    #[test]
    fn create_exchange_for_the_same_tokens_returns_error() -> StdResult<()> {
        let ref mut deps = create_deps();

        let pair = TokenPair (
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into()
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into()
            },
        );

        let result = create_exchange(deps, mock_env("sender", &[]), pair);

        let error: StdError = result.unwrap_err();

        let result = match error {
            StdError::GenericErr { msg, .. } => {
                if msg.as_str() == "Cannot create an exchange with the same token." {
                    true
                } else {
                    false
                }
            }
            _ => false
        };

        assert!(result);

        let pair = TokenPair (
            TokenType::NativeToken {
                denom: "test1".into()
            },
            TokenType::NativeToken {
                denom: "test1".into()
            },
        );

        let result = create_exchange(deps, mock_env("sender", &[]), pair);

        let error: StdError = result.unwrap_err();

        let result = match error {
            StdError::GenericErr { msg, .. } => {
                if msg.as_str() == "Cannot create an exchange with the same token." {
                    true
                } else {
                    false
                }
            }
            _ => false
        };

        assert!(result);

        Ok(())
    }

    #[test]
    fn test_register_exchange() -> StdResult<()> {
        let ref mut deps = create_deps();

        let pair = TokenPair (
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into()
            },
            TokenType::NativeToken {
                denom: "test1".into()
            },
        );

        let sender_addr = HumanAddr("sender1111".into());

        let result = handle(
            deps,
            mock_env(sender_addr.clone(), &[]),
            HandleMsg::RegisterExchange {
                pair: pair.clone(),
                signature: to_binary("whatever")?
            }
        );

        assert_unauthorized(result);

        let config = config();
        save_config(deps, &config)?;

        let env = mock_env(sender_addr.clone(), &[]);

        let signature = create_signature(&env)?;
        save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

        handle(
            deps,
            env,
            HandleMsg::RegisterExchange {
                pair: pair.clone(),
                signature
            }
        )?;

        //Ensure that the ephemeral storage is empty after the message
        let result: Option<Binary> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
        
        match result {
            None => { },
            _ => panic!("Ephemeral storage should be empty!")
        }

        Ok(())
    }

    #[test]
    fn test_register_ido() -> StdResult<()> {
        let ref mut deps = create_deps();

        let sender_addr = HumanAddr("sender1111".into());

        let result = handle(
            deps,
            mock_env(sender_addr.clone(), &[]),
            HandleMsg::RegisterIdo {
                signature: to_binary("whatever")?
            }
        );

        assert_unauthorized(result);

        let config = config();
        save_config(deps, &config)?;

        let env = mock_env(sender_addr.clone(), &[]);

        let signature = create_signature(&env)?;
        save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

        handle(
            deps,
            env,
            HandleMsg::RegisterIdo {
                signature
            }
        )?;
        //Ensure that the ephemeral storage is empty after the message
        let result: Option<Binary> = load(&deps.storage, EPHEMERAL_STORAGE_KEY)?;
        
        match result {
            None => { },
            _ => panic!("Ephemeral storage should be empty!")
        }

        Ok(())
    }

    #[test]
    fn query_exchange() -> StdResult<()> {
        let ref mut deps = create_deps();

        let pair = TokenPair (
            TokenType::CustomToken {
                contract_addr: HumanAddr("token_addr".into()),
                token_code_hash: "13123adasd".into()
            },
            TokenType::NativeToken {
                denom: "test1".into()
            },
        );

        let config = config();
        save_config(deps, &config)?;

        let sender_addr = HumanAddr("sender1111".into());
        let env = mock_env(sender_addr.clone(), &[]);

        let signature = create_signature(&env)?;
        save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

        handle(
            deps,
            env,
            HandleMsg::RegisterExchange {
                pair: pair.clone(),
                signature
            }
        ).unwrap();
        
        let result = query(
            deps,
            QueryMsg::GetExchangeAddress {
                pair: pair.clone()
            }
        )?;

        let response: QueryResponse = from_binary(&result)?;

        match response {
            QueryResponse::GetExchangeAddress { address } => assert_eq!(sender_addr, address),
            _ => return Err(StdError::generic_err("Wrong response. Expected: QueryResponse::GetExchangeAddress."))
        };
        
        Ok(())
    }
}

mod test_state {
    use super::*;

    fn swap_pair(pair: &TokenPair) -> TokenPair {
        TokenPair( pair.1.clone(), pair.0.clone() )
    }

    fn pagination(start: u64, limit: u8) -> Pagination {
        Pagination { start, limit }
    }

    fn mock_config() -> Config<HumanAddr> {
        Config::from_init_msg(InitMsg {
            snip20_contract: ContractInstantiationInfo {
                id: 1,
                code_hash: "snip20_contract".into()
            },
            lp_token_contract: ContractInstantiationInfo {
                id: 2,
                code_hash: "lp_token_contract".into()
            },
            ido_contract: ContractInstantiationInfo {
                id: 3,
                code_hash: "ido_contract".into()
            },
            pair_contract: ContractInstantiationInfo {
                id: 4,
                code_hash: "pair_contract".into()
            },
            exchange_settings: ExchangeSettings {
                swap_fee: Fee::new(28, 10000),
                sienna_fee: Fee::new(2, 10000),
                sienna_burner: None
            }
        })
    }

    #[test]
    fn generates_the_same_key_for_swapped_pairs() -> StdResult<()> {
        fn cmp_pair<S: Storage, A: Api, Q: Querier>(
            deps: &Extern<S, A, Q>,
            pair: TokenPair
        ) -> StdResult<()> {
            let stored_pair = pair.to_stored(&deps.api)?;
            let key = generate_pair_key(&stored_pair);

            let pair = swap_pair(&pair);

            let stored_pair = pair.to_stored(&deps.api)?;
            let swapped_key = generate_pair_key(&stored_pair);

            assert_eq!(key, swapped_key);

            Ok(())
        }

        let ref deps = create_deps();

        cmp_pair(
            deps,
            TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into()
                }
            )
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test1".into()
                },
                TokenType::NativeToken {
                    denom: "test2".into()
                },
            )
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test3".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("third_addr".into()),
                    token_code_hash: "asd21312asd".into()
                }
            )
        )?;

        Ok(())
    }

    #[test]
    fn query_correct_exchange_info() -> StdResult<()> {
        let mut deps = create_deps();

        let pair = TokenPair (
            TokenType::CustomToken {
                contract_addr: HumanAddr("first_addr".into()),
                token_code_hash: "13123adasd".into()
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("scnd_addr".into()),
                token_code_hash: "4534qwerqqw".into()
            }
        );

        let address = HumanAddr("ctrct_addr".into());

        store_exchange(&mut deps, &pair, &address)?;

        let retrieved_address = get_address_for_pair(&deps, &pair)?;

        assert!(pair_exists(&mut deps, &pair)?);
        assert_eq!(address, retrieved_address);

        Ok(())
    }

    #[test]
    fn only_one_exchange_per_factory() -> StdResult<()> {
        let ref mut deps = create_deps();
        let pair = TokenPair (
            TokenType::CustomToken {
                contract_addr: HumanAddr("first_addr".into()),
                token_code_hash: "13123adasd".into()
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("scnd_addr".into()),
                token_code_hash: "4534qwerqqw".into()
            }
        );

        store_exchange(deps, &pair, &"first_addr".into())?;

        let swapped = swap_pair(&pair);

        match store_exchange(deps, &swapped, &"other_addr".into()) {
            Ok(_) => Err(StdError::generic_err("Exchange already exists")),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn test_get_idos() -> StdResult<()> {
        let ref mut deps = create_deps();
        let mut config = mock_config();

        save_config(deps, &config)?;

        let mut addresses = vec![];

        for i in 0..33 {
            let addr = HumanAddr::from(format!("addr_{}", i));

            store_ido_address(deps, &addr, &mut config)?;
            addresses.push(addr);
        }

        let mut config = load_config(deps)?;

        let result = get_idos(deps, &mut config, pagination(addresses.len() as u64, 20))?;
        assert_eq!(result.len(), 0);

        let result = get_idos(deps, &mut config, pagination((addresses.len() - 1) as u64, 20))?;
        assert_eq!(result.len(), 1);

        let result = get_idos(deps, &mut config, pagination(0, PAGINATION_LIMIT + 10))?;
        assert_eq!(result.len(), PAGINATION_LIMIT as usize);

        let result = get_idos(deps, &mut config, pagination(3, PAGINATION_LIMIT))?;
        assert_eq!(result, addresses[3..]);

        Ok(())
    }

    #[test]
    fn test_get_exchanges() -> StdResult<()> {
        let ref mut deps = create_deps();

        let mut exchanges = vec![];

        for i in 0..33 {
            let pair = TokenPair (
                TokenType::CustomToken {
                    contract_addr: HumanAddr(format!("addr_{}", i)),
                    token_code_hash: format!("code_hash_{}", i)
                },
                TokenType::NativeToken {
                    denom: format!("denom_{}", i)
                },
            );
            let address = HumanAddr(format!("address_{}", i));

            store_exchange(deps, &pair, &address)?;

            exchanges.push(Exchange { pair, address });
        }

        let result = get_exchanges(deps, pagination(exchanges.len() as u64, 20))?;
        assert_eq!(result.len(), 0);

        let result = get_exchanges(deps, pagination((exchanges.len() - 1) as u64, 20))?;
        assert_eq!(result.len(), 1);

        let result = get_exchanges(deps, pagination(0, PAGINATION_LIMIT + 10))?;
        assert_eq!(result.len(), PAGINATION_LIMIT as usize);

        let result = get_exchanges(deps, pagination(3, PAGINATION_LIMIT))?;
        assert_eq!(result, exchanges[3..]);

        Ok(())
    }
}
