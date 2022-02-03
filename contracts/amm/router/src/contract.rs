use amm_shared::msg::router::{
    Asset, AssetInfo, HandleMsg, Hop, InitMsg, NativeSwap, QueryMsg, Route, Snip20Swap,
};
use amm_shared::{
    fadroma::platform::{
        from_binary, secret_toolkit::snip20, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Env,
        Extern, HandleResponse, HumanAddr, InitResponse, Querier, StdError, StdResult, Storage,
        Uint128, WasmMsg,
    },
    TokenType,
};

use crate::state::{
    delete_route_state, read_owner, read_route_state, read_tokens, store_owner, store_route_state,
    store_tokens, RouteState,
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if let Some(owner) = msg.owner {
        store_owner(&mut deps.storage, &owner)?;
    } else {
        store_owner(&mut deps.storage, &env.message.sender)?;
    }

    let mut messages: Vec<CosmosMsg> = vec![];

    store_tokens(&mut deps.storage, &vec![])?;
    if let Some(tokens) = msg.register_tokens {
        messages.extend(register_tokens(deps, &env, tokens)?);
    }

    if let Some(callback) = msg.callback {
        // Execute the HandleMsg::RegisterRouter method of
        // the factory contract in order to register this address
        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: callback.contract.address,
            callback_code_hash: callback.contract.code_hash,
            msg: callback.msg,
            send: vec![],
        }));
    }

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
    match msg {
        HandleMsg::Receive {
            from: _,
            msg: Some(msg),
            amount,
        } => handle_first_hop(deps, &env, msg, amount),
        HandleMsg::Receive {
            from,
            msg: None,
            amount,
        } => handle_hop(deps, &env, from, amount),
        HandleMsg::FinalizeRoute {} => finalize_route(deps, &env),
        HandleMsg::RegisterTokens { tokens } => {
            check_owner(deps, &env)?;

            let output_msgs = register_tokens(deps, &env, tokens)?;

            Ok(HandleResponse {
                messages: output_msgs,
                log: vec![],
                data: None,
            })
        }
        HandleMsg::RecoverFunds {
            token,
            amount,
            to,
            snip20_send_msg,
        } => {
            check_owner(deps, &env)?;

            let send_msg = match token {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => vec![snip20::send_msg(
                    to,
                    amount,
                    snip20_send_msg,
                    None,
                    None,
                    256,
                    token_code_hash,
                    contract_addr,
                )?],
                TokenType::NativeToken { denom } => vec![CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address,
                    to_address: to,
                    amount: vec![Coin::new(amount.u128(), &denom)],
                })],
            };

            Ok(HandleResponse {
                messages: send_msg,
                log: vec![],
                data: None,
            })
        }
        HandleMsg::UpdateSettings { new_owner } => {
            check_owner(deps, &env)?;

            if let Some(new_owner) = new_owner {
                store_owner(&mut deps.storage, &new_owner)?;
            }

            Ok(HandleResponse::default())
        }
    }
}

fn handle_first_hop<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    msg: Binary,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    // This is the first msg from the user, with the entire route details
    // 1. save the remaining route to state (e.g. if the route is X/Y -> Y/Z -> Z/W then save Y/Z -> Z/W to state)
    // 2. send `amount` X to pair X/Y
    // 3. call FinalizeRoute to make sure everything went ok, otherwise revert the tx

    let Route {
        mut hops,
        to,
        expected_return,
    } = from_binary(&msg)?;

    if hops.len() < 2 {
        return Err(StdError::generic_err("route must be at least 2 hops"));
    }

    // uscrt can only be the input or output token
    // check that uscrt is not the input token for any hop that is not the first hop
    // (we don't need to check if it's the output token because it's handled in the swap_pair contract)
    for i in 1..(hops.len() - 1) {
        match hops[i].from_token {
            TokenType::NativeToken { denom: _ } => {
                return Err(StdError::generic_err(
                    "cannot route via uscrt. uscrt can only be route input token or output token.",
                ))
            }
            _ => continue,
        }
    }

    let first_hop: Hop = hops.pop_front().unwrap(); // unwrap is cool because `hops.len() >= 2`

    let received_first_hop: bool = match first_hop.from_token {
        TokenType::CustomToken {
            ref contract_addr,
            token_code_hash: _,
        } => env.message.sender == *contract_addr,
        TokenType::NativeToken { ref denom } => {
            env.message.sent_funds.len() == 1
                && env.message.sent_funds[0].amount == amount
                && &env.message.sent_funds[0].denom == denom
        }
    };

    if !received_first_hop {
        return Err(StdError::generic_err(
            "route can only be initiated by sending here the token of the first hop",
        ));
    }

    store_route_state(
        &mut deps.storage,
        &RouteState {
            is_done: false,
            current_hop: Some(first_hop.clone()),
            remaining_route: Route {
                hops, // hops was mutated earlier when we did `hops.pop_front()`
                expected_return,
                to,
            },
        },
    )?;

    let mut msgs = vec![];

    match first_hop.from_token {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            // first hop is a snip20
            msgs.push(snip20::send_msg(
                first_hop.pair_address,
                amount,
                // build swap msg for the next hop
                Some(to_binary(&Snip20Swap::Swap {
                    // set expected_return to None because we don't care about slippage mid-route
                    expected_return: None,
                    // set the recepient of the swap to be this contract (the router)
                    to: Some(env.contract.address.clone()),
                })?),
                None,
                None,
                256,
                token_code_hash,
                contract_addr,
            )?);
        }
        TokenType::NativeToken { ref denom } => {
            // first hop is SCRT
            msgs.push(
                // build swap msg for the next hop
                CosmosMsg::Wasm(WasmMsg::Execute {
                    contract_addr: first_hop.pair_address,
                    callback_code_hash: first_hop.pair_code_hash,
                    msg: to_binary(&NativeSwap::Swap {
                        offer_asset: Asset {
                            amount,
                            info: AssetInfo::NativeToken {
                                denom: denom.clone(),
                            },
                        },
                        // set expected_return to None because we don't care about slippage mid-route
                        expected_return: None,
                        // set the recepient of the swap to be this contract (the router)
                        to: Some(env.contract.address.clone()),
                    })?,
                    send: vec![Coin::new(amount.u128(), &denom)],
                }),
            );
        }
    }

    msgs.push(
        // finalize the route at the end, to make sure the route was completed successfully
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: env.contract.address.clone(),
            callback_code_hash: env.contract_code_hash.clone(),
            msg: to_binary(&HandleMsg::FinalizeRoute {})?,
            send: vec![],
        }),
    );

    Ok(HandleResponse {
        messages: msgs,
        log: vec![],
        data: None,
    })
}

#[allow(unused_variables)]
fn handle_hop<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    from: HumanAddr,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    // This is a receive msg somewhere along the route
    // 1. load route from state (Y/Z -> Z/W)
    // 2. save the remaining route to state (Z/W)
    // 3. send `amount` Y to pair Y/Z

    // 1'. load route from state (Z/W)
    // 2'. this is the last hop so delete the entire route state
    // 3'. send `amount` Z to pair Z/W with recepient `to`
    match read_route_state(&deps.storage)? {
        Some(RouteState {
            is_done,
            current_hop,
            remaining_route:
                Route {
                    mut hops,
                    expected_return,
                    to,
                },
        }) => {
            let next_hop: Hop = match hops.pop_front() {
                Some(next_hop) => next_hop,
                None => return Err(StdError::generic_err("route must be at least 1 hop")),
            };

            let (from_token_address, from_token_code_hash) = match next_hop.clone().from_token {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => (contract_addr, token_code_hash),
                TokenType::NativeToken { .. } => {
                    return Err(StdError::generic_err(
                        "weird. cannot route via uscrt. uscrt can only be route input token or output token.",
                        ));
                }
            };

            let from_pair_of_current_hop = match current_hop {
                Some(Hop {
                    from_token: _,
                    pair_code_hash: _,
                    ref pair_address,
                }) => *pair_address == from,
                None => false,
            };

            if env.message.sender != from_token_address || !from_pair_of_current_hop {
                return Err(StdError::generic_err(
                    "route can only be called by receiving the token of the next hop from the previous pair",
                ));
            }

            let mut is_done = false;
            let mut msgs = vec![];
            let mut current_hop = Some(next_hop.clone());
            if hops.len() == 0 {
                // last hop
                // 1. set is_done to true for FinalizeRoute
                // 2. set expected_return for the final swap
                // 3. set the recipient of the final swap to be the user
                is_done = true;
                current_hop = None;
                msgs.push(snip20::send_msg(
                    next_hop.clone().pair_address,
                    amount,
                    Some(to_binary(&Snip20Swap::Swap {
                        expected_return,
                        to: Some(to.clone()),
                    })?),
                    None,
                    None,
                    256,
                    from_token_code_hash,
                    from_token_address,
                )?);
            } else {
                // not last hop
                // 1. set expected_return to None because we don't care about slippage mid-route
                // 2. set the recipient of the swap to be this contract (the router)
                msgs.push(snip20::send_msg(
                    next_hop.clone().pair_address,
                    amount,
                    Some(to_binary(&Snip20Swap::Swap {
                        expected_return: None,
                        to: Some(env.contract.address.clone()),
                    })?),
                    None,
                    None,
                    256,
                    from_token_code_hash,
                    from_token_address,
                )?);
            }

            store_route_state(
                &mut deps.storage,
                &RouteState {
                    is_done,
                    current_hop,
                    remaining_route: Route {
                        hops, // hops was mutated earlier when we did `hops.pop_front()`
                        expected_return,
                        to,
                    },
                },
            )?;

            Ok(HandleResponse {
                messages: msgs,
                log: vec![],
                data: None,
            })
        }
        None => Err(StdError::generic_err("cannot find route")),
    }
}

fn finalize_route<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<HandleResponse> {
    match read_route_state(&deps.storage)? {
        Some(RouteState {
            is_done,
            current_hop,
            remaining_route,
        }) => {
            // this function is called only by the route creation function
            // it is intended to always make sure that the route was completed successfully
            // otherwise we revert the transaction

            if env.contract.address != env.message.sender {
                return Err(StdError::unauthorized());
            }
            if !is_done {
                return Err(StdError::generic_err(format!(
                    "cannot finalize: route is not done: {:?}",
                    remaining_route
                )));
            }
            if remaining_route.hops.len() != 0 {
                return Err(StdError::generic_err(format!(
                    "cannot finalize: route still contains hops: {:?}",
                    remaining_route
                )));
            }
            if current_hop != None {
                return Err(StdError::generic_err(format!(
                    "cannot finalize: route still processing hops: {:?}",
                    remaining_route
                )));
            }

            delete_route_state(&mut deps.storage);

            Ok(HandleResponse::default())
        }
        None => Err(StdError::generic_err("no route to finalize")),
    }
}

fn check_owner<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
) -> StdResult<()> {
    let owner = read_owner(&deps.storage)?;
    if owner != env.message.sender {
        Err(StdError::unauthorized())
    } else {
        Ok(())
    }
}

fn register_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: &Env,
    tokens: Vec<TokenType<HumanAddr>>,
) -> StdResult<Vec<CosmosMsg>> {
    let mut registered_tokens = read_tokens(&deps.storage)?;
    let mut output_msgs = vec![];

    for token in tokens {
        match token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                if registered_tokens.contains(&contract_addr) {
                    continue;
                }
                registered_tokens.push(contract_addr.clone());

                output_msgs.push(snip20::register_receive_msg(
                    env.contract_code_hash.clone(),
                    None,
                    256,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?);
                output_msgs.push(snip20::set_viewing_key_msg(
                    "SecretSwap Router".into(),
                    None,
                    256,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?);
            }
            _ => (),
        }
    }

    store_tokens(&mut deps.storage, &registered_tokens)?;

    return Ok(output_msgs);
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::SupportedTokens {} => {
            let tokens = read_tokens(&deps.storage)?;
            Ok(to_binary(&tokens)?)
        }
    }
}
