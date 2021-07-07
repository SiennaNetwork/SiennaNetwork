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

use crate::data::{Account, Config, SwapConstants};
use crate::storable::Storable;
use fadroma::scrt::BLOCK_SIZE;

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

    let config = Config {
        input_token: msg.info.input_token,
        sold_token: msg.info.sold_token.clone(),
        swap_constants: SwapConstants {
            sold_token_decimals: get_token_decimals(&deps.querier, msg.info.sold_token)?,
            rate: msg.info.rate,
            input_token_decimals,
        },
        max_seats: msg.info.max_seats,
        max_allocation: msg.info.max_allocation,
        min_allocation: msg.info.min_allocation,
        start_time,
        end_time: msg.info.end_time,
    };
    config.save(deps)?;

    for address in msg.info.whitelist {
        let canonical_address = address.canonize(&deps.api)?;
        let account = Account::new(&canonical_address);
        account.save(deps)?;
    }

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
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::GetRate => get_rate(deps),
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl),
    }
}

/// Swap input token for sold token.
/// Checks if the account exists
/// Checks if the sold token is currently swapable (sale has started and has not yet ended)
/// Checks if the account hasn't gone over the sale limit and is above the sale minimum.
fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let config = Config::<HumanAddr>::load_self(&deps)?;
    let mut account = Account::<CanonicalAddr>::load_self(deps, &env.message.sender)?;
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
    let config = Config::<HumanAddr>::load_self(&deps)?;
    config.is_refundable(env.block.time)?;

    let mut refund_amount = Uint128::zero();

    let accounts = Account::<CanonicalAddr>::load_all(&deps)?;

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

fn get_rate<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::<HumanAddr>::load_self(&deps)?;

    Ok(to_binary(&QueryResponse::GetRate {
        rate: config.swap_constants.rate,
    })?)
}

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

    fn init_contract() -> (Extern<MockStorage, MockApi, MockQuerier>, Env) {
        let mut deps = internal_mock_deps(123, &[]);
        let env = mock_env("admin", &[]);
        let msg = get_init(None, None, None, &env.message.sender);
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
        init_contract();
    }

    #[test]
    fn get_rate_matches() {
        let (deps, _) = init_contract();
        let res = query(&deps, QueryMsg::GetRate).unwrap();
        let res: QueryResponse = from_binary(&res).unwrap();

        match res {
            QueryResponse::GetRate { rate } => assert_eq!(rate, RATE),
        };
    }

    #[test]
    fn attemt_swap_not_whitelisted() {
        let (mut deps, _) = init_contract();
        let env = mock_env(
            "buyer-X",
            &[Coin::new(500_000_000_000_000_000_u128, "uscrt")],
        );

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
    fn attempt_swap_below_minimum() {
        let (mut deps, _) = init_contract();
        let env = mock_env(
            "buyer-1",
            &[Coin::new(500_000_000_000_000_000_u128, "uscrt")],
        );

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
    fn attempt_swap_above_maximum() {
        let (mut deps, _) = init_contract();
        let env = mock_env("buyer-1", &[Coin::new(100_000_000_u128, "uscrt")]);

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
    fn attempt_swap_success() {
        let (mut deps, _) = init_contract();
        let env = mock_env("buyer-1", &[Coin::new(100_000_000_u128, "uscrt")]);

        handle(
            &mut deps,
            env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
            },
        )
        .unwrap();
    }
}
