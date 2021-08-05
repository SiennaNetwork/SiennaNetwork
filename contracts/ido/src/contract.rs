use amm_shared::admin::require_admin;
use amm_shared::{
    admin::admin::{
        admin_handle, admin_query, assert_admin, load_admin,
        save_admin, DefaultHandleImpl, DefaultQueryImpl
    },
    auth::{auth_handle, authenticate, AuthHandleMsg, DefaultHandleImpl as AuthHandle},
    fadroma::scrt::{
        addr::Canonize,
        callback::ContractInstance,
        cosmwasm_std::{
            from_binary, log, to_binary, Api, BankMsg, Binary, CanonicalAddr, Coin, CosmosMsg, Env,
            Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryResult, StdError,
            StdResult, Storage, Uint128, WasmMsg,
        },
        storage::Storable,
        toolkit::snip20,
        utils::{convert::convert_token, viewing_key::ViewingKey},
        BLOCK_SIZE,
    },
    msg::ido::{HandleMsg, InitMsg, QueryMsg, QueryResponse, ReceiverCallbackMsg},
    TokenType,
};

use crate::data::{
    Account, Config, SwapConstants, SaleSchedule,
    save_contract_address, load_contract_address,
    save_viewing_key, load_viewing_key
};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    save_contract_address(deps, &env.contract.address)?;

    let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice());
    save_viewing_key(&mut deps.storage, &viewing_key)?;

    let mut messages = vec![];
    // Set viewing key from IDO contract onto the sold token contract
    // so that we can query the balance of the sold token transfered
    // to the IDO contract
    messages.push(snip20::set_viewing_key_msg(
        viewing_key.to_string(),
        None,
        BLOCK_SIZE,
        msg.info.sold_token.code_hash.clone(),
        msg.info.sold_token.address.clone(),
    )?);

    // Register this contract as a receiver of the callback messages
    // from the buy token so we can trigger back start of the sale after
    // the required amount of funds has been sent to this contract
    messages.push(snip20::register_receive_msg(
        env.contract_code_hash.clone(),
        None,
        BLOCK_SIZE,
        msg.info.sold_token.code_hash.clone(),
        msg.info.sold_token.address.clone(),
    )?);

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
                env.contract_code_hash,
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

    let taken_seats = msg.info.whitelist.len() as u32;

    for address in msg.info.whitelist {
        Account::<CanonicalAddr>::new(&address.canonize(&deps.api)?).save(deps)?;
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
        // Configured when activating
        schedule: None
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
    match msg {
        HandleMsg::Receive {
            from, amount, msg, ..
        } => receive_callback(deps, env, from, amount, msg),
        HandleMsg::Swap { amount, recipient } => {
            // Can be called directly only when the input token is SCRT
            let config = Config::<CanonicalAddr>::load_self(&deps)?;

            if !config.input_token.is_native_token() {
                return Err(StdError::generic_err("Use the SNIP20 receiver interface instead."));
            }

            config.input_token.assert_sent_native_token_balance(&env, amount)?;

            swap(deps, env.block.time, config, amount, env.message.sender, recipient)
        },
        HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultHandleImpl),
        HandleMsg::AdminRefund { address } => refund(deps, env, address),
        HandleMsg::AdminClaim { address } => claim(deps, env, address),
        HandleMsg::AdminAddAddresses { addresses } => add_addresses(deps, env, addresses),
        HandleMsg::CreateViewingKey { entropy, padding } => {
            let msg = AuthHandleMsg::CreateViewingKey { entropy, padding };
            auth_handle(deps, env, msg, AuthHandle)
        },
        HandleMsg::SetViewingKey { key, padding } => {
            let msg = AuthHandleMsg::SetViewingKey { key, padding };
            auth_handle(deps, env, msg, AuthHandle)
        }
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::SaleInfo => get_sale_info(deps),
        QueryMsg::Status => get_status(deps),
        QueryMsg::Balance { address, key } => get_balance(deps, address, key),
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl),
    }
}

/// Universal handler for receive callback from snip20 interface of sold token and possibly custom input token
fn receive_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let config = Config::<CanonicalAddr>::load_self(&deps)?;

    match from_binary(&msg)? {
        ReceiverCallbackMsg::Activate { start_time, end_time } => {
            // If the sender is sold_token, we will treat this like activation
            // handle call that will activate the contract if enough funds is sent
            if env.message.sender == config.sold_token.address {
                return activate(deps, env, from, amount, config, start_time, end_time);
            }
        },
        ReceiverCallbackMsg::Swap { recipient } => {
            if let TokenType::CustomToken { contract_addr, .. } = &config.input_token {
                if env.message.sender == *contract_addr {
                    return swap(deps, env.block.time, config, amount, from, recipient);
                }
            }
        }
    }

    Err(StdError::unauthorized())
}

/// Handle receive callback from the sold token that will activate
/// the start of the IDO sale
///
/// ## Cases
///  - Send full required amount after the contract is instantiated and it will activate the contract
///  - Mint required amount onto IDO contract and then send 0 sell tokens to contract to activate it
fn activate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    mut config: Config<HumanAddr>,
    start: Option<u64>,
    end: u64
) -> StdResult<HandleResponse> {
    if load_admin(deps)? != from {
        return Err(StdError::unauthorized());
    }

    let required_amount = Uint128(config.max_allocation.u128() * config.max_seats as u128);
    let token_balance = get_token_balance(
        &deps,
        env.contract.address,
        config.sold_token.clone(),
        load_viewing_key(&deps.storage)?
    )?;

    if (token_balance + amount) < required_amount {
        return Err(StdError::generic_err(format!(
            "Token balance + amount sent is not enough to cover the sale, contract needs {} tokens, has on balance {}",
            required_amount, token_balance
        )));
    }

    config.schedule = Some(SaleSchedule::new(env.block.time, start, end)?);
    config.save(deps)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "activate")],
        data: None,
    })
}

/// Swap input token for sold token.
/// Checks if the account is whitelisted
/// Checks if the sold token is currently swapable (sale has started and has not yet ended)
/// Checks if the account hasn't gone over the sale limit and is above the sale minimum.
fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    now: u64,
    config: Config<HumanAddr>,
    amount: Uint128,
    from: HumanAddr,
    recipient: Option<HumanAddr>
) -> StdResult<HandleResponse> {
    config.is_swapable(now)?;

    let mut account = Account::<CanonicalAddr>::load_self(&deps, &from)?;

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

    let recipient = recipient.unwrap_or(from);

    Ok(HandleResponse {
        messages: vec![snip20::transfer_msg(
            recipient,
            Uint128(mint_amount),
            None,
            BLOCK_SIZE,
            config.sold_token.code_hash,
            config.sold_token.address,
        )?],
        log: vec![
            log("action", "swap"),
            log("input_amount", amount),
            log("purchased_amount", mint_amount),
            log("account_total_bought", account.total_bought),
        ],
        data: None,
    })
}

/// After the contract has ended, admin can ask for a return of his tokens.
#[require_admin]
fn refund<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = Config::<CanonicalAddr>::load_self(&deps)?;
    config.is_refundable(env.block.time)?;

    let refund_amount = get_token_balance(
        &deps,
        env.contract.address.clone(),
        config.sold_token.clone(),
        load_viewing_key(&deps.storage)?
    )?;

    Ok(HandleResponse {
        messages: vec![snip20::transfer_msg(
            address.unwrap_or_else(|| env.message.sender.clone()),
            refund_amount,
            None,
            BLOCK_SIZE,
            config.sold_token.code_hash,
            config.sold_token.address,
        )?],
        log: vec![log("action", "refund"), log("refund_amount", refund_amount)],
        data: None,
    })
}

/// After the sale has ended, admin will claim any profits that were generated by the sale
#[require_admin]
fn claim<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    let config = Config::<CanonicalAddr>::load_self(&deps)?;
    config.is_refundable(env.block.time)?;
    let output_address = address.unwrap_or_else(|| env.message.sender.clone());
    let balance = config.input_token.query_balance(
        &deps.querier,
        env.contract.address.clone(),
        load_viewing_key(&deps.storage)?.to_string()
    )?;

    let mut messages = vec![];

    match config.input_token {
        TokenType::CustomToken {
            contract_addr,
            token_code_hash,
        } => {
            // Create message for sending the required amount to this contract
            messages.push(snip20::transfer_msg(
                output_address.clone(),
                balance,
                None,
                BLOCK_SIZE,
                token_code_hash,
                contract_addr,
            )?);
        }
        TokenType::NativeToken { denom } => messages.push(
            BankMsg::Send {
                from_address: env.contract.address,
                to_address: output_address.clone(),
                amount: vec![Coin::new(balance.u128(), &denom)],
            }
            .into(),
        ),
    }

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "claim"),
            log("claimed_amount", balance),
            log("output_address", output_address),
        ],
        data: None,
    })
}

fn get_status<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> QueryResult {
    let config = Config::<HumanAddr>::load_self(&deps)?;

    let total_allocation = Uint128(config.max_allocation.u128() * config.max_seats as u128);
    let available_for_sale = get_token_balance(
        &deps,
        load_contract_address(deps)?,
        config.sold_token.clone(),
        load_viewing_key(&deps.storage)?
    )?;

    to_binary(&QueryResponse::Status {
        total_allocation,
        available_for_sale,
        is_active: config.is_active()
    })
}

fn get_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: HumanAddr,
    key: String
) -> QueryResult {
    let canonical = address.canonize(&deps.api)?;
    authenticate(&deps.storage, &ViewingKey(key), canonical.as_slice())?;

    let account = Account::<CanonicalAddr>::load_self(&deps, &address)?;

    to_binary(&QueryResponse::Balance {
        amount: account.total_bought
    })
}

/// Add new address to whitelist
#[require_admin]
fn add_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    let mut config = Config::<CanonicalAddr>::load_self(&deps)?;

    if let Some(schedule) = config.schedule {
        if schedule.has_ended(env.block.time) {
            return Err(StdError::generic_err(
                "Cannot whitelist addresses after the sale has finished."
            ));
        }
    }

    let mut added_count = 0;

    for address in addresses {
        let caonical_address = address.canonize(&deps.api)?;

        config.taken_seats += 1;
        if config.taken_seats > config.max_seats {
            return Err(StdError::generic_err("All seats already taken."));
        }

        let account = Account::<CanonicalAddr>::load_self(&deps, &address);

        if account.is_err() {
            Account::<CanonicalAddr>::new(&caonical_address).save(deps)?;
            added_count += 1;
        }
    }

    config.save(deps)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "add_addresses"),
            log("new_addresses", added_count)
        ],
        data: None,
    })
}

/// Return info about the token sale
fn get_sale_info<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = Config::<HumanAddr>::load_self(&deps)?;

    let (start, end) = if let Some(schedule) = config.schedule {
        (Some(schedule.start), Some(schedule.end))
    } else {
        (None, None)
    };

    to_binary(&QueryResponse::SaleInfo {
        input_token: config.input_token,
        sold_token: config.sold_token,
        rate: config.swap_constants.rate,
        taken_seats: config.taken_seats,
        max_seats: config.max_seats,
        max_allocation: config.max_allocation,
        min_allocation: config.min_allocation,
        end,
        start
    })
}

/// Query the token for number of its decimals
fn get_token_decimals(
    querier: &impl Querier,
    instance: ContractInstance<HumanAddr>,
) -> StdResult<u8> {
    let result = snip20::token_info_query(
        querier,
        BLOCK_SIZE,
        instance.code_hash,
        instance.address
    )?;

    Ok(result.decimals)
}

/// Query the token for number of its decimals
fn get_token_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    this_contract: HumanAddr,
    instance: ContractInstance<HumanAddr>,
    viewing_key: ViewingKey,
) -> StdResult<Uint128> {
    let balance = snip20::balance_query(
        &deps.querier,
        this_contract,
        viewing_key.0,
        BLOCK_SIZE,
        instance.code_hash,
        instance.address,
    )?;

    Ok(balance.amount)
}

#[cfg(test)]
mod tests {
    use super::*;
    use amm_shared::{
        fadroma::scrt::callback::Callback,
        fadroma::scrt::cosmwasm_std::{
            from_binary,
            testing::{mock_env, MockApi, MockStorage},
            Binary, Coin, Env, Extern,
        },
        msg::ido::{HandleMsg, InitMsg, QueryMsg, QueryResponse, TokenSaleConfig},
        TokenType,
    };

    use crate::querier::MockQuerier;

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
            querier: MockQuerier::new(&[(&contract_addr, balance)], Uint128::from(2500 as u128)),
        }
    }

    /// Get init message for initialization of the token.
    fn get_init(
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
                max_seats: 5,
                max_allocation: MAX_ALLOCATION,
                min_allocation: MIN_ALLOCATION
            },
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
                msg: Some(to_binary(&ReceiverCallbackMsg::Activate {
                    start_time: Some(start_time),
                    end_time
                }).unwrap()),
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
                end
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
            },
            _ => panic!("Expected QueryResponse::GetRate")
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
                recipient: None
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
                recipient: None
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
                recipient: None
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
                recipient: None
            },
        )
        .unwrap();

        let bought = resp.log[2].value.parse::<u128>().unwrap();
        let key = String::from("key");

        let resp = query(
            &deps,
            QueryMsg::Balance { address: buyer.into(), key: key.clone() }
        ).unwrap_err();

        assert_eq!(resp, StdError::unauthorized());
        
        handle(
            &mut deps,
            env,
            HandleMsg::SetViewingKey { key: key.clone(), padding: None }
        ).unwrap();

        let resp = query(
            &deps,
            QueryMsg::Balance { address: buyer.into(), key }
        ).unwrap();
        let resp: QueryResponse = from_binary(&resp).unwrap();

        match resp {
            QueryResponse::Balance { amount } => {
                assert_eq!(amount.u128(), bought);
            },
            _ => panic!("Expected QueryResponse::Balance")
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
                recipient: None
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
                recipient: None
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
                recipient: None
            },
        )
        .unwrap();

        deps.querier.sub_balance(Uint128(250 as u128)).unwrap();

        let res = query(&deps, QueryMsg::Status).unwrap();
        let res: QueryResponse = from_binary(&res).unwrap();

        match res {
            QueryResponse::Status { available_for_sale, total_allocation, .. } => {
                assert_eq!(total_allocation, Uint128(2500));
                assert_eq!(available_for_sale, Uint128(2250));
            },
            _ => panic!("Expected QueryResponse::Status")
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
                addresses: vec![ buyer_env.message.sender ],
            }
        ).unwrap();

        assert_eq!(
            String::from("0"),
            res.log[1].value
        );
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
                recipient: None
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
                addresses: vec![ buyer_env.message.sender.clone() ],
            },
        )
        .unwrap();

        handle(
            &mut deps,
            buyer_env,
            HandleMsg::Swap {
                amount: Uint128(250_000_000_u128),
                recipient: None
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
                addresses: vec![ buyer_env.message.sender.clone() ],
            },
        )
        .unwrap();

        let res = handle(
            &mut deps,
            env,
            HandleMsg::AdminAddAddresses {
                addresses: vec![ buyer_env2.message.sender.clone() ],
            },
        );

        assert_eq!(res, Err(StdError::generic_err("All seats already taken.")));
    }
}
