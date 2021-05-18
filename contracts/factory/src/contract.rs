use cosmwasm_std::{
    Api, Binary, CosmosMsg, Env, Extern, HandleResponse, InitResponse,
    Querier, StdError, StdResult, Storage, WasmMsg, log, to_binary
};
use sienna_amm_shared::{Callback, ContractInfo, TokenPair, Pagination};
use sienna_amm_shared::msg::exchange::InitMsg as ExchangeInitMsg;
use sienna_amm_shared::msg::ido::{IdoInitMsg, IdoInitConfig};
use sienna_amm_shared::msg::factory::{InitMsg, HandleMsg, QueryMsg, QueryResponse};
use sienna_amm_shared::msg::sienna_burner::HandleMsg as BurnerHandleMsg;
use sienna_amm_shared::storage::*;

use crate::state::{
    save_config, load_config, Config, pair_exists, store_exchange,
    get_address_for_pair, get_idos, store_ido_address, get_exchanges
};
use fadroma_scrt_migrate::{is_operational, can_set_status, set_status, get_status};

const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let config = Config::from_init_msg(msg);
    save_config(deps, &config)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    with_status!(deps, env, match msg {
        HandleMsg::CreateExchange { pair } => create_exchange(deps, env, pair),
        HandleMsg::CreateIdo { info } => create_ido(deps, env, info),
        HandleMsg::RegisterExchange { pair, signature } => register_exchange(deps, env, pair, signature),
        HandleMsg::RegisterIdo { signature } => register_ido(deps, env, signature)
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::GetExchangeAddress { pair } => query_exchange_address(deps, pair),
        QueryMsg::ListExchanges { pagination } => list_exchanges(deps, pagination),
        QueryMsg::ListIdos { pagination } => list_idos(deps, pagination),
        QueryMsg::GetExchangeSettings => query_exchange_settings(deps)
    }
}

fn create_exchange<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair
) -> StdResult<HandleResponse> {

    if pair.0 == pair.1 {
        return Err(StdError::generic_err("Cannot create an exchange with the same token."));
    }

    if pair_exists(deps, &pair)? {
        return Err(StdError::generic_err("Pair already exists"));
    }

    let config = load_config(deps)?;

    // We take advantage of the serialized execution model to create a signature
    // and remove it at the end of the transaction. This signature is passed to
    // the created pair which it then returns to HandleMsg::RegisterExchange so that
    // it can be compared to the one we stored. This way, we ensure that exchanges 
    // can only be created through this method.
    let signature = create_signature(&env)?;
    save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

    // Actually creating the exchange happens when the instantiated contract calls
    // us back via the HandleMsg::RegisterExchange so that we can get its address.

    Ok(HandleResponse{
        messages: vec![
            CosmosMsg::Wasm(
                WasmMsg::Instantiate {
                    code_id: config.pair_contract.id,
                    callback_code_hash: config.pair_contract.code_hash,
                    send: vec![],
                    label: format!(
                        "{}-{}-pair-{}-{}",
                        pair.0,
                        pair.1,
                        env.contract.address,
                        config.pair_contract.id
                    ),
                    msg: to_binary(
                        &ExchangeInitMsg {
                            pair: pair.clone(),
                            lp_token_contract: config.lp_token_contract.clone(),
                            factory_info: ContractInfo {
                                code_hash: env.contract_code_hash.clone(),
                                address: env.contract.address.clone()
                            },
                            callback: Callback {
                                contract: ContractInfo {
                                    address: env.contract.address,
                                    code_hash: env.contract_code_hash,
                                },
                                msg: to_binary(&HandleMsg::RegisterExchange {
                                    pair: pair.clone(),
                                    signature
                                })?,
                            }
                        }
                    )?
                }
            )
        ],
        log: vec![
            log("action", "create_exchange"),
            log("pair", pair),
        ],
        data: None
    })
}

fn register_exchange<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair,
    signature: Binary
) -> StdResult<HandleResponse> {
    ensure_correct_signature(&mut deps.storage, signature)?;

    store_exchange(deps, &pair, &env.message.sender)?;

    let config = load_config(&deps)?;
    let mut messages = vec![];

    if let Some(info) = config.exchange_settings.sienna_burner {
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: info.address,
            callback_code_hash: info.code_hash,
            msg: to_binary(&BurnerHandleMsg::AddPairs {
                pairs: vec![ env.message.sender.clone() ]
            })?,
            send: vec![]
        }))
    }

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "register_exchange"),
            log("address", env.message.sender),
            log("pair", pair)
        ],
        data: None
    })
}

fn query_exchange_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: TokenPair
) -> StdResult<Binary> {
    let address = get_address_for_pair(deps, &pair)?;
    
    to_binary(&QueryResponse::GetExchangeAddress {
        address
    })
}

fn create_ido<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: IdoInitConfig
) -> StdResult<HandleResponse> {
    let signature = create_signature(&env)?;
    save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;
    
    // Again, creating the IDO happens when the instantiated contract calls
    // us back via the HandleMsg::RegisterIdo so that we can get its address.
    
    let config = load_config(deps)?;

    Ok(HandleResponse {
        messages: vec![
            CosmosMsg::Wasm(WasmMsg::Instantiate {
                code_id: config.ido_contract.id,
                callback_code_hash: config.ido_contract.code_hash.clone(),
                send: vec![],
                label: format!(
                    "IDO for {}({}), id: {}",
                    info.snip20_init_info.name,
                    info.snip20_init_info.symbol,
                    config.ido_contract.id
                ),
                msg: to_binary(&IdoInitMsg {
                    info,
                    snip20_contract: config.snip20_contract,
                    callback: Callback {
                        contract: ContractInfo {
                            address: env.contract.address,
                            code_hash: env.contract_code_hash,
                        },
                        msg: to_binary(&HandleMsg::RegisterIdo {
                            signature
                        })?
                    }
                })?
            })
        ],
        log: vec![
            log("action", "create_exchange")
        ],
        data: None
    })
}

fn register_ido<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    signature: Binary
) -> StdResult<HandleResponse> {
    ensure_correct_signature(&mut deps.storage, signature)?;

    let mut config = load_config(deps)?;

    store_ido_address(deps, &env.message.sender, &mut config)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("created IDO", env.message.sender)
        ],
        data: None
    })
}

fn list_idos<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination
) -> StdResult<Binary> {
    let config = load_config(deps)?;
    let idos = get_idos(deps, &config, pagination)?;

    to_binary(&QueryResponse::ListIdos { idos })
}

fn list_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination
) -> StdResult<Binary> {
    let exchanges = get_exchanges(deps, pagination)?;

    to_binary(&QueryResponse::ListExchanges { exchanges })
}

fn query_exchange_settings<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Binary> {
    let config = load_config(deps)?;

    Ok(to_binary(&QueryResponse::GetExchangeSettings {
        settings: config.exchange_settings
    })?)
}

fn create_signature(env: &Env) -> StdResult<Binary> {
    to_binary(&[
        env.message.sender.0.as_bytes(),
        &env.block.height.to_be_bytes(),
        &env.block.time.to_be_bytes()
    ].concat())
}

fn ensure_correct_signature(storage: &mut impl Storage, signature: Binary) -> StdResult<()> {
    let stored_signature: Binary =
        load(storage, EPHEMERAL_STORAGE_KEY)?.unwrap_or_default();

    if stored_signature != signature {
        return  Err(StdError::unauthorized());
    }

    remove(storage, EPHEMERAL_STORAGE_KEY);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sienna_amm_shared::{ContractInstantiationInfo, Fee, TokenType, ExchangeSettings};
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, MockApi,
        MockQuerier, MockStorage
    };
    use cosmwasm_std::{from_binary, StdError, HumanAddr};

    fn dependencies() -> Extern<MockStorage, MockApi, MockQuerier> {
        mock_dependencies(10, &[])
    }

    fn assert_unauthorized(response: StdResult<HandleResponse>) {
        assert!(response.is_err());

        let err = response.unwrap_err();
        assert_eq!(err, StdError::unauthorized())
    }

    fn config() -> Config {
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
        let ref mut deps = dependencies();

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
        let ref mut deps = dependencies();

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
        let ref mut deps = dependencies();

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
        let ref mut deps = dependencies();

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
        let ref mut deps = dependencies();

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
