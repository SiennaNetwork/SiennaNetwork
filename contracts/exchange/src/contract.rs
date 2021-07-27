use amm_shared::{
    exchange::{ExchangeSettings, Fee},
    fadroma::scrt::{
        callback::{Callback, ContractInstance},
        cosmwasm_std::{
            from_binary, log, to_binary, Api, BankMsg, Binary, Coin, CosmosMsg, Decimal, Env,
            Extern, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest, QueryResult,
            StdError, StdResult, Storage, Uint128, WasmMsg, WasmQuery,
        },
        migrate as fadroma_scrt_migrate,
        toolkit::snip20,
        utils::{crypto::Prng, viewing_key::ViewingKey, Uint256},
    },
    msg::{
        exchange::{
            HandleMsg, InitMsg, QueryMsg, QueryMsgResponse, ReceiverCallbackMsg,
            SwapSimulationResponse,
        },
        factory::{QueryMsg as FactoryQueryMsg, QueryResponse as FactoryResponse},
        snip20::{InitConfig as Snip20InitConfig, InitMsg as Snip20InitMsg},
    },
    TokenPairAmount, TokenType, TokenTypeAmount,
};
use fadroma_scrt_migrate::{get_status, with_status};

use crate::{
    decimal_math,
    state::{load_config, store_config, Config},
};

// This should be incremented every time there is a change to the interface of the contract.
const CONTRACT_VERSION: u32 = 1;

struct SwapInfo {
    total_commission: Uint128,
    sienna_commission: Uint128,
    swap_commission: Uint128,
    result: SwapResult,
}

struct SwapResult {
    return_amount: Uint128,
    spread_amount: Uint128,
}

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg.pair.0 == msg.pair.1 {
        return Err(StdError::generic_err(
            "Trying to create an exchange with the same token.",
        ));
    }

    let mut messages = vec![];

    let mut rng = Prng::new(
        &env.message.sender.0.as_bytes(),
        &env.block.time.to_be_bytes(),
    );
    let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), &rng.rand_bytes());

    register_custom_token(&env, &mut messages, &msg.pair.0, &viewing_key)?;
    register_custom_token(&env, &mut messages, &msg.pair.1, &viewing_key)?;

    // Create LP token
    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: msg.lp_token_contract.id,
        msg: to_binary(&Snip20InitMsg {
            name: format!(
                "SiennaSwap Liquidity Provider (LP) token for {}-{}",
                &msg.pair.0, &msg.pair.1
            ),
            admin: Some(env.contract.address.clone()),
            symbol: "SWAP-LP".to_string(),
            decimals: 6,
            callback: Some(Callback {
                msg: to_binary(&HandleMsg::OnLpTokenInit)?,
                contract: ContractInstance {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash,
                },
            }),
            initial_balances: None,
            initial_allowances: None,
            prng_seed: Binary::from(rng.rand_bytes()),
            config: Some(
                Snip20InitConfig::builder()
                    .public_total_supply()
                    .enable_mint()
                    .enable_burn()
                    .build(),
            ),
        })?,
        send: vec![],
        label: format!(
            "{}-{}-SiennaSwap-LP-Token-{}",
            &msg.pair.0, &msg.pair.1, &env.contract.address
        ),
        callback_code_hash: msg.lp_token_contract.code_hash.clone(),
    }));

    // Execute the HandleMsg::RegisterExchange method of
    // the factory contract in order to register this address
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.callback.contract.address,
        callback_code_hash: msg.callback.contract.code_hash,
        msg: msg.callback.msg,
        send: vec![],
    }));

    let config = Config {
        factory_info: msg.factory_info,
        lp_token_info: ContractInstance {
            code_hash: msg.lp_token_contract.code_hash,
            // We get the address when the instantiated LP token calls OnLpTokenInit
            address: HumanAddr::default(),
        },
        pair: msg.pair,
        contract_addr: env.contract.address.clone(),
        viewing_key,
    };

    store_config(deps, &config)?;

    Ok(InitResponse {
        messages,
        log: vec![log("created_exchange_address", env.contract.address)],
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
            } => receiver_callback(deps, env, from, amount, msg),
            HandleMsg::AddLiquidity {
                deposit,
                slippage_tolerance,
            } => add_liquidity(deps, env, deposit, slippage_tolerance),
            HandleMsg::OnLpTokenInit => register_lp_token(deps, env),
            HandleMsg::Swap {
                offer,
                expected_return,
                recipient,
            } => {
                // Can only be called directly when the offer token is SCRT, otherwise
                // has to be called through the SNIP20 receiver interface by sending
                // the amount to the pair's account in the SNIP20 token

                if !offer.token.is_native_token() {
                    return Err(StdError::unauthorized());
                }

                offer.assert_sent_native_token_balance(&env)?;

                let config = load_config(deps)?;
                let sender = env.message.sender.clone();

                swap(
                    &deps.querier,
                    env,
                    config,
                    sender,
                    recipient,
                    offer,
                    expected_return,
                )
            }
        }
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::PairInfo => {
            let config = load_config(deps)?;

            let balances = config.pair.query_balances(
                &deps.querier,
                config.contract_addr,
                config.viewing_key.0,
            )?;
            let total_liquidity = query_liquidity(&deps.querier, &config.lp_token_info)?;

            to_binary(&QueryMsgResponse::PairInfo {
                liquidity_token: config.lp_token_info,
                factory: config.factory_info,
                pair: config.pair,
                amount_0: balances[0],
                amount_1: balances[1],
                total_liquidity,
                contract_version: CONTRACT_VERSION,
            })
        }
        QueryMsg::SwapSimulation { offer } => {
            let config = load_config(deps)?;
            to_binary(&swap_simulation(deps, config, offer)?)
        }
    }
}

// Take action depending on the message coming in from
// the SNIP20 receiver interface. Doing swaps and removing liquidity
// this way, means no allowance is required.
fn receiver_callback<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    msg: Option<Binary>,
) -> StdResult<HandleResponse> {
    let msg = msg.ok_or_else(|| {
        StdError::generic_err("Receiver callback \"msg\" parameter cannot be empty.")
    })?;

    let config = load_config(deps)?;

    match from_binary(&msg)? {
        ReceiverCallbackMsg::Swap {
            recipient,
            expected_return,
        } => {
            for token in config.pair.into_iter() {
                match token {
                    TokenType::CustomToken { contract_addr, .. } => {
                        if *contract_addr == env.message.sender {
                            let offer = TokenTypeAmount {
                                token: token.clone(),
                                amount,
                            };

                            return swap(
                                &deps.querier,
                                env,
                                config,
                                from,
                                recipient,
                                offer,
                                expected_return,
                            );
                        }
                    }
                    _ => continue,
                }
            }

            Err(StdError::unauthorized())
        }
        ReceiverCallbackMsg::RemoveLiquidity { recipient } => {
            if config.lp_token_info.address != env.message.sender {
                return Err(StdError::unauthorized());
            }

            remove_liquidity(deps, env, amount, recipient)
        }
    }
}

fn add_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    deposit: TokenPairAmount<HumanAddr>,
    slippage: Option<Decimal>,
) -> StdResult<HandleResponse> {
    let config = load_config(&deps)?;

    let Config {
        pair,
        contract_addr,
        viewing_key,
        lp_token_info,
        ..
    } = config;

    if pair != deposit.pair {
        return Err(StdError::generic_err(
            "The provided tokens dont match those managed by the contract.",
        ));
    }

    // Because we asserted that the provided pair and the one that is managed by the contract
    // are identical, from here on, we must only work with the one provided (deposit.pair).
    // This is because even though pairs with orders (A,B) and (B,A) are identical, the `amount_0` and `amount_1`
    // variables correspond directly to the pair provided and not necessarily to the one stored. So in this case, order is important.

    let mut messages: Vec<CosmosMsg> = vec![];

    let mut pool_balances =
        deposit
            .pair
            .query_balances(&deps.querier, contract_addr, viewing_key.0)?;

    for (i, (amount, token)) in deposit.into_iter().enumerate() {
        match &token {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                messages.push(snip20::transfer_from_msg(
                    env.message.sender.clone(),
                    env.contract.address.clone(),
                    amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?);
            }
            TokenType::NativeToken { .. } => {
                // If the asset is native token, balance is already increased.
                // To calculate properly we should subtract user deposit from the pool.
                token.assert_sent_native_token_balance(&env, amount)?;
                pool_balances[i] = (pool_balances[i] - amount)?;
            }
        }
    }

    assert_slippage_tolerance(
        slippage,
        &[deposit.amount_0, deposit.amount_1],
        &pool_balances,
    )?;

    let liquidity_supply = query_liquidity(&deps.querier, &lp_token_info)?;

    let lp_tokens = if liquidity_supply == Uint128::zero() {
        // If the provider is minting a new pool, the number of liquidity tokens they will
        // receive will equal sqrt(x * y), where x and y represent the amount of each token provided.

        let amount_0 = Uint256::from(deposit.amount_0);
        let amount_1 = Uint256::from(deposit.amount_1);

        (amount_0 * amount_1)?.sqrt()?.clamp_u128()?
    } else {
        // When adding to an existing pool, an equal amount of each token, proportional to the
        // current price, must be deposited. So, determine how many LP tokens are minted.

        let total_share = Uint256::from(liquidity_supply);

        let amount_0 = Uint256::from(deposit.amount_0);
        let pool_0 = Uint256::from(pool_balances[0]);

        let share_0 = ((amount_0 * total_share)? / pool_0)?;

        let amount_1 = Uint256::from(deposit.amount_1);
        let pool_1 = Uint256::from(pool_balances[1]);

        let share_1 = ((amount_1 * total_share)? / pool_1)?;

        std::cmp::min(share_0, share_1).clamp_u128()?
    };

    messages.push(snip20::mint_msg(
        env.message.sender,
        Uint128(lp_tokens),
        None,
        BLOCK_SIZE,
        lp_token_info.code_hash,
        lp_token_info.address,
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "provide_liquidity"),
            log("assets", format!("{}, {}", deposit.pair.0, deposit.pair.1)),
            log("share", lp_tokens),
        ],
        data: None,
    })
}

// This function works off the assumption that it was triggered by the LP token SNIP20
// receiver callback. i.e the message flow is SNIP20.send -> pair.remove_liquidity
// So need to have checked that the request was sent by one of the
// token contracts in this pair.
fn remove_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    recipient: HumanAddr,
) -> StdResult<HandleResponse> {
    let config = load_config(&deps)?;

    let Config {
        pair,
        lp_token_info,
        contract_addr,
        viewing_key,
        ..
    } = config;

    let liquidity_supply = query_liquidity(&deps.querier, &lp_token_info)?;
    let pool_balances = pair.query_balances(&deps.querier, contract_addr, viewing_key.0)?;

    // Calculate the withdrawn amount for each token in the pair - for each token X
    // amount of X withdrawn = amount in pool for X * amount of LP tokens being burned / total liquidity pool amount

    let withdraw_amount = Uint256::from(amount);
    let total_liquidity = Uint256::from(liquidity_supply);

    let mut pool_withdrawn: [Uint128; 2] = [Uint128::zero(), Uint128::zero()];

    for (i, pool_amount) in pool_balances.iter().enumerate() {
        let pool_amount = Uint256::from(*pool_amount);
        pool_withdrawn[i] = ((pool_amount * withdraw_amount)? / total_liquidity)?
            .clamp_u128()?
            .into();
    }

    let mut messages: Vec<CosmosMsg> = Vec::with_capacity(3);

    for (i, token) in pair.into_iter().enumerate() {
        messages.push(token.create_send_msg(
            env.contract.address.clone(),
            recipient.clone(),
            pool_withdrawn[i],
        )?);
    }

    messages.push(snip20::burn_msg(
        amount,
        None,
        BLOCK_SIZE,
        lp_token_info.code_hash,
        lp_token_info.address,
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "remove_liquidity"),
            log("withdrawn_share", amount),
            log("refund_assets", format!("{}, {}", &pair.0, &pair.1)),
        ],
        data: None,
    })
}

// This function works off the assumption that it was triggered by the SNIP20
// receiver callback. i.e the message flow is SNIP20.send -> pair.swap
// So need to have checked that the request was sent by one of the
// token contracts in this pair.
// It also doesn't check for sent native balance.
fn swap(
    querier: &impl Querier,
    env: Env,
    config: Config<HumanAddr>,
    sender: HumanAddr,
    recipient: Option<HumanAddr>,
    offer: TokenTypeAmount<HumanAddr>,
    expected_return: Option<Uint128>,
) -> StdResult<HandleResponse> {
    let settings = query_exchange_settings(querier, config.factory_info.clone())?;
    let swap = do_swap(querier, &config, &settings, &offer, false)?;

    if let Some(expected_return) = expected_return {
        if swap.result.return_amount.lt(&expected_return) {
            return Err(StdError::generic_err(
                "Operation fell short of expected_return",
            ));
        }
    }

    let mut messages = Vec::with_capacity(2);

    // Transfer a small fee to the burner address
    if let Some(burner_address) = settings.sienna_burner {
        if swap.sienna_commission > Uint128::zero() {
            match &offer.token {
                TokenType::CustomToken {
                    contract_addr,
                    token_code_hash,
                } => {
                    messages.push(snip20::transfer_msg(
                        burner_address,
                        swap.sienna_commission,
                        None,
                        BLOCK_SIZE,
                        token_code_hash.clone(),
                        contract_addr.clone(),
                    )?);
                }
                TokenType::NativeToken { denom } => {
                    offer.assert_sent_native_token_balance(&env)?;

                    messages.push(CosmosMsg::Bank(BankMsg::Send {
                        from_address: env.contract.address.clone(),
                        to_address: burner_address,
                        amount: vec![Coin {
                            denom: denom.clone(),
                            amount: swap.sienna_commission,
                        }],
                    }));
                }
            }
        }
    }

    // Send the resulting amount of the output token
    let index = config.pair.get_token_index(&offer.token).unwrap(); // Safe, checked in do_swap
    let token = config.pair.get_token(index ^ 1).unwrap();

    let recipient = recipient.unwrap_or(sender);
    messages.push(token.create_send_msg(
        env.contract.address,
        recipient,
        swap.result.return_amount,
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "swap"),
            log("offer_token", offer.token),
            log("offer_amount", offer.amount),
            log("return_amount", swap.result.return_amount),
            log("spread_amount", swap.result.spread_amount),
            log("sienna_commission", swap.sienna_commission),
            log("swap_commission", swap.swap_commission),
            log("commission_amount", swap.total_commission),
        ],
        data: None,
    })
}

fn query_liquidity(
    querier: &impl Querier,
    lp_token_info: &ContractInstance<HumanAddr>,
) -> StdResult<Uint128> {
    let result = snip20::token_info_query(
        querier,
        BLOCK_SIZE,
        lp_token_info.code_hash.clone(),
        lp_token_info.address.clone(),
    )?;

    //If this happens, the LP token has been incorrectly configured
    if result.total_supply.is_none() {
        unreachable!("LP token has no available supply.");
    }

    Ok(result.total_supply.unwrap())
}

fn swap_simulation<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: Config<HumanAddr>,
    offer: TokenTypeAmount<HumanAddr>,
) -> StdResult<SwapSimulationResponse> {
    let settings = query_exchange_settings(&deps.querier, config.factory_info.clone())?;

    let swap = do_swap(&deps.querier, &config, &settings, &offer, true)?;

    Ok(SwapSimulationResponse {
        return_amount: swap.result.return_amount,
        spread_amount: swap.result.spread_amount,
        commission_amount: swap.total_commission,
    })
}

fn register_lp_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    let mut config = load_config(&deps)?;

    //This should only be set once when the LP token is instantiated.
    if config.lp_token_info.address != HumanAddr::default() {
        return Err(StdError::unauthorized());
    }

    config.lp_token_info.address = env.message.sender.clone();

    store_config(deps, &config)?;

    Ok(HandleResponse {
        messages: vec![snip20::register_receive_msg(
            env.contract_code_hash,
            None,
            BLOCK_SIZE,
            config.lp_token_info.code_hash,
            env.message.sender.clone(),
        )?],
        log: vec![log("liquidity_token_addr", env.message.sender)],
        data: None,
    })
}

fn register_custom_token(
    env: &Env,
    messages: &mut Vec<CosmosMsg>,
    token: &TokenType<HumanAddr>,
    viewing_key: &ViewingKey,
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr,
        token_code_hash,
        ..
    } = token
    {
        messages.push(snip20::set_viewing_key_msg(
            viewing_key.0.clone(),
            None,
            BLOCK_SIZE,
            token_code_hash.clone(),
            contract_addr.clone(),
        )?);
        messages.push(snip20::register_receive_msg(
            env.contract_code_hash.clone(),
            None,
            BLOCK_SIZE,
            token_code_hash.clone(),
            contract_addr.clone(),
        )?);
    }

    Ok(())
}

fn do_swap(
    querier: &impl Querier,
    config: &Config<HumanAddr>,
    settings: &ExchangeSettings<HumanAddr>,
    offer: &TokenTypeAmount<HumanAddr>,
    is_simulation: bool,
) -> StdResult<SwapInfo> {
    if !config.pair.contains(&offer.token) {
        return Err(StdError::generic_err(format!(
            "The supplied token {}, is not managed by this contract.",
            offer.token
        )));
    }

    let offer_amount = Uint256::from(offer.amount);
    let swap_commission = percentage_decrease(offer_amount, settings.swap_fee)?;

    let sienna_commission = if settings.sienna_burner.is_some() {
        percentage_decrease(offer_amount, settings.sienna_fee)?
    } else {
        Uint128::zero()
    };

    let balances = config.pair.query_balances(
        querier,
        config.contract_addr.clone(),
        config.viewing_key.0.clone(),
    )?;
    let token_index = config.pair.get_token_index(&offer.token).unwrap(); //Safe because we checked above for existence

    let mut offer_pool = balances[token_index];

    if !is_simulation {
        // If not a simulation, need to subtract the incoming amount
        // from the pool
        offer_pool = (offer_pool - offer.amount)?;
    }

    let total_commission = swap_commission + sienna_commission;
    let offer_amount = (offer.amount - total_commission)?;

    Ok(SwapInfo {
        total_commission,
        swap_commission,
        sienna_commission,
        result: compute_swap(offer_pool, balances[token_index ^ 1], offer_amount)?,
    })
}

// Based on https://github.com/enigmampc/SecretSwap/blob/ffd72d1c94096ac3a78aaf8e576f22584f49fe7a/contracts/secretswap_pair/src/contract.rs#L768
fn compute_swap(
    offer_pool: Uint128,
    ask_pool: Uint128,
    offer_amount: Uint128,
) -> StdResult<SwapResult> {
    // offer => ask
    let offer_pool = Uint256::from(offer_pool);
    let ask_pool = Uint256::from(ask_pool);
    let offer_amount = Uint256::from(offer_amount);

    let total_pool = (offer_pool * ask_pool)?;
    let return_amount = (ask_pool - (total_pool / (offer_pool + offer_amount)?)?)?;

    // spread = offer_amount * ask_pool / offer_pool - return_amount
    let spread_amount = ((offer_amount * ask_pool)? / offer_pool)?;
    let spread_amount = (spread_amount - return_amount).unwrap_or(Uint256::zero());

    Ok(SwapResult {
        return_amount: return_amount.clamp_u128()?.into(),
        spread_amount: spread_amount.clamp_u128()?.into(),
    })
}

fn percentage_decrease(amount: Uint256, fee: Fee) -> StdResult<Uint128> {
    let nom = Uint256::from(fee.nom);
    let denom = Uint256::from(fee.denom);

    let decrease_amount = ((amount * nom)? / denom)?;

    Ok(decrease_amount.clamp_u128()?.into())
}

/// The amount the price moves in a trading pair between when a transaction is submitted and when it is executed.
/// Returns an `StdError` if the range of the expected tokens to be received is exceeded.
fn assert_slippage_tolerance(
    slippage: Option<Decimal>,
    deposits: &[Uint128; 2],
    pools: &[Uint128; 2],
) -> StdResult<()> {
    if slippage.is_none() {
        return Ok(());
    }

    let one_minus_slippage_tolerance =
        decimal_math::decimal_subtraction(Decimal::one(), slippage.unwrap())?;

    // Ensure each prices are not dropped as much as slippage tolerance rate
    if decimal_math::decimal_multiplication(
        Decimal::from_ratio(deposits[0], deposits[1]),
        one_minus_slippage_tolerance,
    ) > Decimal::from_ratio(pools[0], pools[1])
        || decimal_math::decimal_multiplication(
            Decimal::from_ratio(deposits[1], deposits[0]),
            one_minus_slippage_tolerance,
        ) > Decimal::from_ratio(pools[1], pools[0])
    {
        return Err(StdError::generic_err(
            "Operation exceeds max slippage tolerance",
        ));
    }

    Ok(())
}

fn query_exchange_settings(
    querier: &impl Querier,
    factory: ContractInstance<HumanAddr>,
) -> StdResult<ExchangeSettings<HumanAddr>> {
    let result: FactoryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        callback_code_hash: factory.code_hash,
        contract_addr: factory.address,
        msg: to_binary(&FactoryQueryMsg::GetExchangeSettings)?,
    }))?;

    match result {
        FactoryResponse::GetExchangeSettings { settings } => Ok(settings),
        _ => Err(StdError::generic_err(
            "An error occurred while trying to retrieve exchange settings.",
        )),
    }
}
