use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, WasmMsg, CosmosMsg, log, HumanAddr
};
use shared::{Callback, ContractInfo, ExchangeInitMsg, IdoInitConfig, IdoInitMsg, TokenPair};

use crate::msg::{InitMsg, HandleMsg, QueryMsg, QueryResponse};
use crate::state::{
    save_config, load_config, Config, pair_exists, store_exchange,
    get_address_for_pair, get_pair_for_address, Exchange, get_idos,
    Pagination, store_ido_address
};

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
    match msg {
        HandleMsg::CreateExchange { pair } => create_exchange(deps, env, pair),
        HandleMsg::CreateIdo { info } => create_ido(deps, env, info),
        HandleMsg::RegisterExchange { pair } => register_exchange(deps, env, pair),
        HandleMsg::RegisterIdo => register_ido(deps, env)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetExchangePair { exchange_addr } => query_exchange_pair(deps, exchange_addr),
        QueryMsg::GetExchangeAddress { pair } => query_exchange_address(deps, pair),
        QueryMsg::ListIdos { pagination } => list_idos(deps, pagination)
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

    // Actually creating the exchange happens when the instantiated contract calls
    // us back via the HandleMsg::RegisterExchange so that we can get its address.
    // This is also more robust as we should register the pair only if the exchange
    // contract has been successfully instantiated.

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
                        env.contract.address.clone(),
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
                                contract_addr: env.contract.address,
                                contract_code_hash: env.contract_code_hash,
                                msg: to_binary(&HandleMsg::RegisterExchange {
                                    pair: pair.clone()
                                })?,
                            },
                            sienna_token: config.sienna_token
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
    pair: TokenPair
) -> StdResult<HandleResponse> {
    let exchange = Exchange {
        pair,
        address: env.message.sender
    };

    store_exchange(deps, &exchange)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_exchange"),
            log("pair", exchange.pair),
        ],
        data: None
    })
}

fn query_exchange_pair<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    exchange_addr: HumanAddr
) -> StdResult<Binary> {
    let pair = get_pair_for_address(deps, &exchange_addr)?;

    to_binary(&QueryResponse::GetExchangePair {
        pair
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

    let config = load_config(deps)?;
    
    // Again, creating the IDO happens when the instantiated contract calls
    // us back via the HandleMsg::RegisterIdo so that we can get its address.
    
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
                    info: info,
                    snip20_contract: config.snip20_contract,
                    callback: Callback {
                        contract_addr: env.contract.address,
                        contract_code_hash: env.contract_code_hash,
                        msg: to_binary(&HandleMsg::RegisterIdo)?
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
    env: Env
) -> StdResult<HandleResponse> {
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
    let result = get_idos(deps, &config, pagination)?;

    Ok(to_binary(&QueryResponse::ListIdos {
        idos: result
    })?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use shared::{ContractInstantiationInfo, TokenType};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{from_binary, StdError};

    fn dependencies() -> Extern<MockStorage, MockApi, MockQuerier> {
        mock_dependencies(10, &[])
    }

    #[test]
    fn proper_initialization() -> StdResult<()> {
        let ref mut deps = dependencies();

        let snip20_contract = ContractInstantiationInfo {
            code_hash: "12355254".into(),
            id: 64
        };

        let lp_token_contract = ContractInstantiationInfo {
            code_hash: "23123123".into(),
            id: 64
        };

        let pair_contract = ContractInstantiationInfo {
            code_hash: "2341586789".into(),
            id: 33
        };

        let ido_contract = ContractInstantiationInfo {
            code_hash: "348534835".into(),
            id: 69
        };

        let sienna_token = ContractInfo {
            code_hash: "3124312312".into(),
            address: HumanAddr("sienna_tkn".into())
        };

        let result = init(deps, mock_env("sender1111", &[]), InitMsg {
            snip20_contract: snip20_contract.clone(),
            lp_token_contract: lp_token_contract.clone(),
            pair_contract: pair_contract.clone(),
            ido_contract: ido_contract.clone(),
            sienna_token: sienna_token.clone()
        });

        assert!(result.is_ok());

        let config = load_config(deps)?;

        assert_eq!(snip20_contract, config.snip20_contract);
        assert_eq!(lp_token_contract, config.lp_token_contract);
        assert_eq!(pair_contract, config.pair_contract);
        assert_eq!(ido_contract, config.ido_contract);
        assert_eq!(sienna_token, config.sienna_token);

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

        let sender_addr = HumanAddr("sender1111".into());

        handle(deps, mock_env(sender_addr.clone(), &[]), HandleMsg::RegisterExchange {
            pair: pair.clone()
        })?;
        
        let result = query(deps, QueryMsg::GetExchangeAddress {
            pair: pair.clone()
        })?;

        let response: QueryResponse = from_binary(&result)?;

        match response {
            QueryResponse::GetExchangeAddress { address } => assert_eq!(sender_addr, address),
            _ => return Err(StdError::generic_err("Wrong response. Expected: QueryResponse::GetExchangeAddress."))
        };
        
        let result = query(deps, QueryMsg::GetExchangePair {
            exchange_addr: sender_addr.clone()
        })?;

        let response: QueryResponse = from_binary(&result)?;
        
        match response {
            QueryResponse::GetExchangePair { pair } => assert_eq!(pair, pair),
            _ => return Err(StdError::generic_err("Wrong response. Expected: QueryResponse::GetExchangePair."))
        };
        
        Ok(())
    }
}
