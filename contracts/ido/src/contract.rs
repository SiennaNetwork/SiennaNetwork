#![allow(dead_code)]
#![allow(unused_imports)]
use amm_shared::admin::admin::{
    admin_handle, admin_query, assert_admin, save_admin, DefaultHandleImpl, DefaultQueryImpl,
};
use amm_shared::msg::ido::{HandleMsg, InitMsg, QueryMsg, QueryResponse};
use amm_shared::TokenType;
use fadroma::scrt::addr::Canonize;
use fadroma::scrt::callback::ContractInstance;
use fadroma::scrt::cosmwasm_std::{
    log, to_binary, Api, CanonicalAddr, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, LogAttribute, Querier, QueryResult, StdError, StdResult, Storage, Uint128,
    WasmMsg,
};
use fadroma::scrt::toolkit::snip20;
use fadroma::scrt::utils::convert::convert_token;
use fadroma::scrt::BLOCK_SIZE;

use crate::data::{Account, Config, SwapConstants};
use crate::storable::Storable;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let input_token_decimals = match &msg.info.input_token {
        TokenType::NativeToken { .. } => 6,
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => get_token_decimals(
            &deps.querier,
            ContractInstance {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone(),
            },
        )?,
    };

    save_admin(deps, &msg.admin)?;

    let start_time = msg.info.start_time.unwrap_or(env.block.time);

    let mut whitelist: Vec<CanonicalAddr> = vec![];

    for address in msg.info.whitelist {
        whitelist.push(address.canonize(&deps.api)?);
    }

    let config = Config {
        input_token: msg.info.input_token,
        sold_token: msg.info.sold_token.clone(),
        swap_constants: SwapConstants {
            sold_token_decimals: get_token_decimals(&deps.querier, msg.info.sold_token)?,
            rate: msg.info.rate,
            input_token_decimals,
        },
        whitelist,
        max_seats: msg.info.max_seats,
        max_allocation: msg.info.max_allocation,
        min_allocation: msg.info.min_allocation,
        start_time,
        end_time: msg.info.end_time,
    };
    config.save(deps)?;

    Ok(InitResponse {
        messages: vec![
            // Execute the HandleMsg::RegisterIdo method of
            // the factory contract in order to register this address
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: msg.callback.contract.address,
                callback_code_hash: msg.callback.contract.code_hash,
                msg: msg.callback.msg,
                send: vec![],
            }),
        ],
        log: vec![],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Swap { amount } => swap(deps, env, amount),
        HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultHandleImpl),
        HandleMsg::Refund => refund(deps, env),
        HandleMsg::Status => get_status(deps, env),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetRate => get_rate(deps),
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl),
    }
}

/// Swap input token for sold token.
/// Checks if the account is whitelisted
/// Checks if the sold token is currently swapable (sale has started and has not yet ended)
/// Checks if the account hasn't gone over the sale limit and is above the sale minimum.
fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = Config::<CanonicalAddr>::load_self(&deps)?;
    let mut account = config.load_account(&deps, &env.message.sender)?;
    config.is_swapable(env.block.time)?;

    let mint_amount = convert_token(
        amount.u128(),
        config.swap_constants.rate.u128(),
        config.swap_constants.input_token_decimals,
        config.swap_constants.sold_token_decimals,
    )?;

    if mint_amount < config.min_allocation.u128() {
        return Err(StdError::generic_err(format!(
            "Insufficient amount provided: the resulting amount fell short of the minimum purchase expected: {}",
            config.min_allocation
        )));
    }

    account.total_bought = account
        .total_bought
        .u128()
        .checked_add(mint_amount)
        .ok_or(StdError::generic_err("Upper bound overflow detected."))?
        .into();

    if account.total_bought > config.max_allocation {
        return Err(StdError::generic_err(format!(
            "This purchase exceeds the total maximum allowed amount for a single address: {}",
            config.max_allocation
        )));
    }

    account.save(deps)?;

    swap_internal(
        env,
        Some(amount),
        Uint128(mint_amount),
        config,
        vec![
            log("action", "swap"),
            log("input_amount", amount),
            log("purchased_amount", mint_amount),
        ],
    )
}

/// After the contract has ended, admin can ask for a return of his tokens.
fn refund<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    assert_admin(&deps, &env)?;
    let config = Config::<CanonicalAddr>::load_self(&deps)?;
    config.is_refundable(env.block.time)?;

    let mut refund_amount = Uint128::zero();

    let accounts = config.load_accounts(&deps)?;

    for account in accounts {
        refund_amount += (config.max_allocation - account.total_bought)?;
    }

    swap_internal(
        env,
        None,
        refund_amount,
        config,
        vec![
            log("action", "refund"),
            log("refunded_amount", refund_amount),
        ],
    )
}

/// Performs internal swap of the amount
fn swap_internal(
    env: Env,
    input_amount: Option<Uint128>,
    output_amount: Uint128,
    config: Config<HumanAddr>,
    log: Vec<LogAttribute>,
) -> StdResult<HandleResponse> {
    let mut messages = vec![];

    // If the input amount was given, add the message to take the input tokens
    if let Some(amount) = input_amount {
        // Retrieve the input amount from the sender's balance
        match config.input_token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                messages.push(snip20::transfer_from_msg(
                    env.message.sender.clone(),
                    env.contract.address,
                    amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash,
                    contract_addr,
                )?);
            }
            TokenType::NativeToken { .. } => {
                config
                    .input_token
                    .assert_sent_native_token_balance(&env, amount)?;
            }
        }
    }

    // Transfer the resulting amount to the sender
    messages.push(snip20::transfer_msg(
        env.message.sender,
        output_amount,
        None,
        BLOCK_SIZE,
        config.sold_token.code_hash,
        config.sold_token.address,
    )?);

    Ok(HandleResponse {
        messages,
        log,
        data: None,
    })
}

/// Handle method that will return status
fn get_status<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    assert_admin(&deps, &env)?;
    let config = Config::<HumanAddr>::load_self(&deps)?;

    let mut sold_allocation = Uint128::zero();
    let mut available_to_allocate = Uint128::zero();
    let mut total_allocation = Uint128::zero();

    let accounts = config.load_accounts(&deps)?;

    for account in accounts {
        sold_allocation += account.total_bought;
        available_to_allocate += (config.max_allocation - account.total_bought)?;
        total_allocation += config.max_allocation;
    }

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "status"),
            log("sold_allocation", sold_allocation),
            log("available_to_allocate", available_to_allocate),
            log("total_allocation", total_allocation),
        ],
        data: None,
    })
}

/// Return exchange rate for swap
fn get_rate<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::<HumanAddr>::load_self(&deps)?;

    Ok(to_binary(&QueryResponse::GetRate {
        rate: config.swap_constants.rate,
    })?)
}

/// Query the token for number of its decimals
fn get_token_decimals(
    querier: &impl Querier,
    instance: ContractInstance<HumanAddr>,
) -> StdResult<u8> {
    let result =
        snip20::token_info_query(querier, BLOCK_SIZE, instance.code_hash, instance.address)?;

    Ok(result.decimals)
}

#[cfg(test)]
mod tests {
    use super::*;
    use amm_shared::msg::ido::{HandleMsg, InitMsg, QueryMsg, QueryResponse, TokenSaleConfig};
    use amm_shared::TokenType;
    use fadroma::scrt::callback::Callback;
    use fadroma::scrt::cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockStorage};
    use fadroma::scrt::cosmwasm_std::Binary;
    use fadroma::scrt::cosmwasm_std::{from_binary, to_binary};
    use fadroma::scrt::cosmwasm_std::{Coin, Env, Extern};

    use crate::querier::MockQuerier;

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
            querier: MockQuerier::new(&[(&contract_addr, balance)]),
        }
    }

    /// Get init message for initialization of the token.
    fn get_init(
        start_time: Option<u64>,
        end_time: Option<u64>,
        sold_token: Option<ContractInstance<HumanAddr>>,
        admin: &HumanAddr,
    ) -> InitMsg {
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
                max_seats: 4,
                max_allocation: MAX_ALLOCATION,
                min_allocation: MIN_ALLOCATION,
                start_time,
                end_time,
            },
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
        let msg = get_init(start_time, end_time, None, &env.message.sender);
        init(&mut deps, env.clone(), msg).unwrap();

        (deps, env)
    }

    #[test]
    fn fails_with_init_if_invalid_token() {
        let mut deps = internal_mock_deps(123, &[]);
        let env = mock_env("admin", &[]);
        let msg = get_init(
            None,
            None,
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
    fn query_get_rate_matches_init() {
        let (deps, _) = init_contract(None, None);
        let res = query(&deps, QueryMsg::GetRate).unwrap();
        let res: QueryResponse = from_binary(&res).unwrap();

        match res {
            QueryResponse::GetRate { rate } => assert_eq!(rate, RATE),
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
    fn buyer_swaps_success_gets_ok() {
        let (mut deps, _) = init_contract(None, None);
        let env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
            },
        )
        .unwrap();
    }

    #[test]
    fn buyer_attempt_swap_before_sale_start_gets_error() {
        let block_time = 1_571_797_419;
        let start_time = 1_571_797_500;
        let (mut deps, _) = init_contract(Some(start_time), None);
        let env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
            },
        );

        assert_eq!(
            res,
            Err(StdError::generic_err(format!(
                "Sale hasn\'t started yet, come back in {} seconds",
                start_time - block_time
            )))
        )
    }

    #[test]
    fn buyer_attempt_swap_after_sale_end_gets_error() {
        let _block_time = 1_571_797_419;
        let start_time = 1_571_797_300;
        let end_time = 1_571_797_400;
        let (mut deps, _) = init_contract(Some(start_time), Some(end_time));
        let env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        let res = handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
            },
        );

        assert_eq!(res, Err(StdError::generic_err("Sale has ended")))
    }

    #[test]
    fn admin_attempt_refund_before_sale_end_gets_error() {
        let block_time = 1_571_797_419;
        let start_time = 1_571_797_300;
        let end_time = 1_571_797_500;
        let (mut deps, env) = init_contract(Some(start_time), Some(end_time));

        let res = handle(&mut deps, env, HandleMsg::Refund);

        assert_eq!(
            res,
            Err(StdError::generic_err(format!(
                "Sale hasn\'t finished yet, come back in {} seconds",
                end_time - block_time
            )))
        );
    }

    #[test]
    fn admin_attempt_refund_on_sale_with_no_end_gets_error() {
        let _block_time = 1_571_797_419;
        let start_time = 1_571_797_300;
        let (mut deps, env) = init_contract(Some(start_time), None);

        let res = handle(&mut deps, env, HandleMsg::Refund);

        assert_eq!(
            res,
            Err(StdError::generic_err(
                "Cannot refund, sale is still active and will last until all the funds are swapped"
            ))
        );
    }

    #[test]
    fn admin_performs_refund_after_sale_end() {
        let _block_time = 1_571_797_419;
        let start_time = 1_571_797_300;
        let end_time = 1_571_797_400;
        let (mut deps, env) = init_contract(Some(start_time), Some(end_time));

        let res = handle(&mut deps, env.clone(), HandleMsg::Refund).unwrap();
        let refunded_amount = &res.log[1].value;

        assert_eq!(refunded_amount, "2000");
    }

    #[test]
    fn admin_get_status_of_sale() {
        let (mut deps, env) = init_contract(None, None);
        let buyer_env = mock_env("buyer-1", &[Coin::new(250_000_000_u128, "uscrt")]);

        handle(
            &mut deps,
            buyer_env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
            },
        )
        .unwrap();

        let res = handle(&mut deps, env, HandleMsg::Status).unwrap();
        let sold_allocation = &res.log[1].value;
        let available_to_allocate = &res.log[2].value;
        let total_allocation = &res.log[3].value;

        assert_eq!(sold_allocation, "250");
        assert_eq!(available_to_allocate, "1750");
        assert_eq!(total_allocation, "2000");
    }
}
