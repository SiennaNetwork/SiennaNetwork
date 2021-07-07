use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, Binary, StdResult, Storage, QueryResult, CosmosMsg, WasmMsg,
    Uint128, log, HumanAddr, Decimal, QueryRequest, WasmQuery, BankMsg, Coin
};
use secret_toolkit::snip20;
use amm_shared::{
    ExchangeSettings, Fee, TokenPairAmount, TokenType, TokenTypeAmount,
    create_send_msg,
    msg::{
        exchange::{InitMsg, HandleMsg, QueryMsg, QueryMsgResponse, SwapSimulationResponse},
        factory::{QueryMsg as FactoryQueryMsg, QueryResponse as FactoryResponse},
        snip20::{InitConfig as Snip20InitConfig, InitMsg as Snip20InitMsg}
    }
};
use amm_shared::fadroma::utils::{u256_math, u256_math::U256, viewing_key::ViewingKey, crypto::Prng};
use amm_shared::fadroma::callback::{Callback, ContractInstance};
use amm_shared::fadroma::migrate as fadroma_scrt_migrate;
use fadroma_scrt_migrate::{get_status, with_status};

use crate::{state::{Config, store_config, load_config}, decimal_math};

// This should be incremented every time there is a change to the interface of the contract.
const CONTRACT_VERSION: u32 = 1;

struct SwapInfo {
    total_commission: Uint128,
    sienna_commission: PercentageDecreaseResult,
    swap_commission: PercentageDecreaseResult,
    result: SwapResult
}

struct SwapResult {
    return_amount: Uint128,
    spread_amount: Uint128
}

struct PercentageDecreaseResult {
    new_amount: Uint128,
    decrease_amount: Uint128
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
        return Err(StdError::generic_err("Trying to create an exchange with the same token."));
    }

    let mut messages = vec![];

    let mut rng = Prng::new(&env.message.sender.0.as_bytes(), &env.block.time.to_be_bytes());

    let viewing_key = ViewingKey::new(&env, &rng.rand_bytes(), &rng.rand_bytes());

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
                    code_hash: env.contract_code_hash
                }
            }),
            initial_balances: None,
            initial_allowances: None,
            prng_seed: Binary::from(rng.rand_bytes()),
            config: Some(Snip20InitConfig::builder()
                .public_total_supply()
                .enable_mint()
                .build()
            )
        })?,
        send: vec![],
        label: format!(
            "{}-{}-SiennaSwap-LP-Token-{}",
            &msg.pair.0,
            &msg.pair.1,
            &env.contract.address
        ),
        callback_code_hash: msg.lp_token_contract.code_hash.clone(),
    }));

    // Execute the HandleMsg::RegisterExchange method of
    // the factory contract in order to register this address
    messages.push(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: msg.callback.contract.address,
            callback_code_hash: msg.callback.contract.code_hash,
            msg: msg.callback.msg,
            send: vec![],
        })
    );

    let config = Config {
        factory_info: msg.factory_info,
        lp_token_info: ContractInstance {
            code_hash: msg.lp_token_contract.code_hash,
            // We get the address when the instantiated LP token calls OnLpTokenInit
            address: HumanAddr::default()
        },
        pair: msg.pair,
        contract_addr: env.contract.address.clone(),
        viewing_key
    };

    store_config(deps, &config)?;

    Ok(InitResponse {
        messages,
        log: vec![
            log("created_exchange_address", env.contract.address.as_str())
        ]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    with_status!(deps, env, match msg {
        HandleMsg::AddLiquidity { deposit, slippage_tolerance } => add_liquidity(deps, env, deposit, slippage_tolerance),
        HandleMsg::RemoveLiquidity { amount, recipient } => remove_liquidity(deps, env, amount, recipient),
        HandleMsg::OnLpTokenInit => register_lp_token(deps, env),
        HandleMsg::Swap { offer, expected_return } => swap(deps, env, offer, expected_return)
    })
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    match msg {
        QueryMsg::Version => to_binary(&QueryMsgResponse::Version { version: CONTRACT_VERSION }),
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::PairInfo => {
            let config = load_config(deps)?;

            let balances = config.pair.query_balances(&deps.querier, config.contract_addr, config.viewing_key.0)?;
            let total_liquidity = query_liquidity(&deps.querier, &config.lp_token_info)?;

            to_binary(&QueryMsgResponse::PairInfo {
                liquidity_token: config.lp_token_info,
                factory: config.factory_info,
                pair: config.pair,
                amount_0: balances[0],
                amount_1: balances[1],
                total_liquidity
            })
        },
        QueryMsg::SwapSimulation { offer } => {
            let config = load_config(deps)?;
            to_binary(&swap_simulation(deps, config, offer)?)
        }
    }
}

fn add_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    deposit: TokenPairAmount<HumanAddr>,
    slippage: Option<Decimal>
) -> StdResult<HandleResponse> {
    deposit.assert_sent_native_token_balance(&env)?;

    let config = load_config(&deps)?;

    let Config {
        pair,
        contract_addr,
        viewing_key,
        lp_token_info,
        ..
    } = config;

    if pair != deposit.pair {
        return Err(StdError::generic_err("The provided tokens dont match those managed by the contract."));
    }

    // Because we asserted that the provided pair and the one that is managed by the contract
    // are identical, from here on, we must only work with the one provided (deposit.pair).
    // This is because even though pairs with orders (A,B) and (B,A) are identical, the `amount_0` and `amount_1`
    // variables correspond directly to the pair provided and not necessarily to the one stored. So in this case, order is important.

    let mut messages: Vec<CosmosMsg> = vec![];

    let mut pool_balances = deposit.pair.query_balances(&deps.querier, contract_addr, viewing_key.0)?;

    for (i, (amount, token)) in deposit.into_iter().enumerate() {
        match &token {
            TokenType::CustomToken { contract_addr, token_code_hash } => {
                messages.push(snip20::transfer_from_msg(
                    env.message.sender.clone(),
                    env.contract.address.clone(),
                    amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone())?
                );
            },
            TokenType::NativeToken { .. } => {
                // If the asset is native token, balance is already increased.
                // To calculate properly we should subtract user deposit from the pool.
                pool_balances[i] = (pool_balances[i] - amount)?;
            }
        }
    }

    assert_slippage_tolerance(
        slippage,
        &[ deposit.amount_0, deposit.amount_1 ],
        &pool_balances
    )?;

    let liquidity_supply = query_liquidity(&deps.querier, &lp_token_info)?;

    let lp_tokens = if liquidity_supply == Uint128::zero() {
        // If the provider is minting a new pool, the number of liquidity tokens they will
        // receive will equal sqrt(x * y), where x and y represent the amount of each token provided.

        let amount_0 = U256::from(deposit.amount_0.u128());
        let amount_1 = U256::from(deposit.amount_1.u128());

        let initial_liquidity = u256_math::mul(Some(amount_0), Some(amount_1))
            .and_then(u256_math::sqrt)
            .ok_or(StdError::generic_err(format!(
                "Cannot calculate sqrt(deposit_0 {} * deposit_1 {})",
                amount_0, amount_1
            )))?;

        clamp(initial_liquidity)?
    } else {
        // When adding to an existing pool, an equal amount of each token, proportional to the
        // current price, must be deposited. So, determine how many LP tokens are minted.

        let total_share = Some(U256::from(liquidity_supply.u128()));

        let amount_0 = Some(U256::from(deposit.amount_0.u128()));
        let pool_0 = Some(U256::from(pool_balances[0].u128()));

        let share_0 = u256_math::div(u256_math::mul(amount_0, total_share), pool_0)
            .ok_or(StdError::generic_err(format!(
                "Cannot calculate deposits[0] {} * total_share {} / pools[0].amount {}",
                amount_0.unwrap(),
                total_share.unwrap(),
                pool_0.unwrap()
            )))?;

        let amount_1 = Some(U256::from(deposit.amount_1.u128()));
        let pool_1 = Some(U256::from(pool_balances[1].u128()));

        let share_1 = u256_math::div(u256_math::mul(amount_1, total_share), pool_1)
            .ok_or(StdError::generic_err(format!(
                "Cannot calculate deposits[1] {} * total_share {} / pools[1].amount {}",
                amount_1.unwrap(),
                total_share.unwrap(),
                pool_1.unwrap()
            )))?;

        clamp(std::cmp::min(share_0, share_1))?
    };

    messages.push(snip20::mint_msg(
        env.message.sender,
        lp_tokens,
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

fn remove_liquidity<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
    recipient: HumanAddr
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

    let withdraw_amount = Some(U256::from(amount.u128()));
    let total_liquidity = Some(U256::from(liquidity_supply.u128()));

    let mut pool_withdrawn: [Uint128; 2] = [ Uint128::zero(), Uint128::zero() ];

    for (i, pool_amount) in pool_balances.iter().enumerate() {
        let pool_amount = Some(U256::from(pool_amount.u128()));

        let withdrawn_token_amount = u256_math::div(
            u256_math::mul(pool_amount, withdraw_amount),
            total_liquidity,
        ).ok_or(StdError::generic_err(format!(
            "Cannot calculate current_pool_amount {} * withdrawn_share_amount {} / total_share {}",
            pool_amount.unwrap(),
            withdraw_amount.unwrap(),
            total_liquidity.unwrap()
        )))?;

        pool_withdrawn[i] = clamp(withdrawn_token_amount)?;
    }

    let mut messages: Vec<CosmosMsg> = Vec::with_capacity(2);

    for (i, token) in pair.into_iter().enumerate() {
        messages.push(
            create_send_msg(&token, env.contract.address.clone(), recipient.clone(), pool_withdrawn[i])?
        );
    }

    messages.push(snip20::burn_from_msg(
        env.message.sender,
        amount,
        None,
        BLOCK_SIZE,
        lp_token_info.code_hash,
        lp_token_info.address
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "remove_liquidity"),
            log("withdrawn_share", amount),
            log("refund_assets", format!("{}, {}", &pair.0, &pair.1)),
        ],
        data: None
    })
}

fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    offer: TokenTypeAmount<HumanAddr>,
    expected_return: Option<Uint128>
) -> StdResult<HandleResponse> {
    let config = load_config(deps)?;
    let settings = query_exchange_settings(&deps.querier, config.factory_info.clone())?;

    let swap = do_swap(deps, &config, &settings, &offer, false)?;

    if let Some(expected_return) = expected_return {
        if swap.result.return_amount.lt(&expected_return) {
            return Err(StdError::generic_err(
                "Operation fell short of expected_return",
            ));
        }
    }

    let mut messages = vec![];

    // Transfer a small fee to the burner address
    if let Some(burner_address) = settings.sienna_burner {
        match &offer.token {
            TokenType::CustomToken { contract_addr, token_code_hash } => {
                // Transfer the fee directly to burner
                messages.push(snip20::transfer_from_msg(
                    env.message.sender.clone(),
                    burner_address,
                    swap.sienna_commission.decrease_amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone()
                )?);

                // and the rest to the contract
                messages.push(snip20::transfer_from_msg(
                    env.message.sender.clone(),
                    env.contract.address.clone(),
                    swap.sienna_commission.new_amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone()
                )?);
            },
            TokenType::NativeToken { denom } => {
                offer.assert_sent_native_token_balance(&env)?;

                messages.push(CosmosMsg::Bank(BankMsg::Send {
                    from_address: env.contract.address.clone(),
                    to_address: burner_address,
                    amount: vec![Coin {
                        denom: denom.clone(),
                        amount: swap.sienna_commission.decrease_amount
                    }]
                }));
            }
        }
    } else {
        match &offer.token {
            TokenType::NativeToken { .. } => {
                offer.assert_sent_native_token_balance(&env)?;
            },
            TokenType::CustomToken { contract_addr, token_code_hash } => {
                messages.push(snip20::transfer_from_msg(
                    env.message.sender.clone(),
                    env.contract.address.clone(),
                    offer.amount,
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone()
                )?);
            }
        }
    }

    // Send the resulting amount of the output token
    let index = config.pair.get_token_index(&offer.token).unwrap(); // Safe, checked in do_swap
    let token = config.pair.get_token(index ^ 1).unwrap();

    messages.push(create_send_msg(&token, env.contract.address, env.message.sender, swap.result.return_amount)?);

    Ok(HandleResponse{
        messages,
        log: vec![
            log("action", "swap"),
            log("offer_token", offer.token),
            log("offer_amount", offer.amount),
            log("return_amount", swap.result.return_amount),
            log("spread_amount", swap.result.spread_amount),
            log("sienna_commission", swap.sienna_commission.decrease_amount),
            log("swap_commission", swap.swap_commission.decrease_amount),
            log("commission_amount", swap.total_commission)
        ],
        data: None
    })
}

fn query_liquidity(
    querier: &impl Querier,
    lp_token_info: &ContractInstance<HumanAddr>
) -> StdResult<Uint128> {
    let result = snip20::token_info_query(
        querier,
        BLOCK_SIZE,
        lp_token_info.code_hash.clone(),
        lp_token_info.address.clone()
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
    offer: TokenTypeAmount<HumanAddr>
) -> StdResult<SwapSimulationResponse> {
    let settings = query_exchange_settings(&deps.querier, config.factory_info.clone())?;

    let swap = do_swap(deps, &config, &settings, &offer, true)?;

    Ok(SwapSimulationResponse {
        return_amount: swap.result.return_amount,
        spread_amount: swap.result.spread_amount,
        commission_amount: swap.total_commission
    })
}

fn register_lp_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env
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
    viewing_key: &ViewingKey
) -> StdResult<()> {
    if let TokenType::CustomToken {
        contract_addr, token_code_hash, ..
    } = token {
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

fn do_swap<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: &Config<HumanAddr>,
    settings: &ExchangeSettings<HumanAddr>,
    offer: &TokenTypeAmount<HumanAddr>,
    is_simulation: bool
) -> StdResult<SwapInfo> {
    if !config.pair.contains(&offer.token) {
        return Err(StdError::generic_err(format!("The supplied token {}, is not managed by this contract.", offer.token)));
    }

    let offer_amount = U256::from(offer.amount.u128());
    let swap_commission = percentage_decrease(offer_amount, settings.swap_fee)?;

    let sienna_commission = if settings.sienna_burner.is_some() {
        percentage_decrease(offer_amount, settings.sienna_fee)?
    } else {
        PercentageDecreaseResult {
            new_amount: Uint128::zero(),
            decrease_amount: Uint128::zero()
        }
    };

    let balances = config.pair.query_balances(&deps.querier, config.contract_addr.clone(), config.viewing_key.0.clone())?;
    let token_index = config.pair.get_token_index(&offer.token).unwrap(); //Safe because we checked above for existence

    let mut offer_pool = balances[token_index];

    if !is_simulation {
        // If offer.token is not native, the balance hasn't been increased yet
        if let TokenType::NativeToken { .. } = offer.token {
            let result = U256::from(offer_pool.u128()).checked_sub(U256::from(offer.amount.u128()))
                .ok_or_else(|| StdError::generic_err("This can't really happen."))?;

            offer_pool = clamp(result)?
        }
    }

    let total_commission = swap_commission.decrease_amount + sienna_commission.decrease_amount;
    let offer_amount = (offer.amount - total_commission)?;

    Ok(SwapInfo {
        total_commission,
        swap_commission,
        sienna_commission,
        result: compute_swap(
            offer_pool,
            balances[token_index ^ 1],
            offer_amount
        )?
    })
}

// Copied from https://github.com/enigmampc/SecretSwap/blob/ffd72d1c94096ac3a78aaf8e576f22584f49fe7a/contracts/secretswap_pair/src/contract.rs#L768
fn compute_swap(
    offer_pool: Uint128,
    ask_pool: Uint128,
    offer_amount: Uint128
) -> StdResult<SwapResult> {
    // offer => ask
    let offer_pool = Some(U256::from(offer_pool.u128()));
    let ask_pool = Some(U256::from(ask_pool.u128()));
    let offer_amount = Some(U256::from(offer_amount.u128()));

    // total_pool = offer_pool * ask_pool
    let total_pool = u256_math::mul(offer_pool, ask_pool).ok_or(StdError::generic_err(format!(
        "Cannot calculate total_pool = offer_pool {} * ask_pool {}",
        offer_pool.unwrap(),
        ask_pool.unwrap()
    )))?;

    // return_amount = (ask_pool - total_pool / (offer_pool + offer_amount))
    let return_amount = u256_math::sub(ask_pool, u256_math::div(Some(total_pool), u256_math::add(offer_pool, offer_amount)))
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate return_amount = (ask_pool {} - total_pool {} / (offer_pool {} + offer_amount {}))",
            ask_pool.unwrap(),
            total_pool,
            offer_pool.unwrap(),
            offer_amount.unwrap(),
        )))?;

    // calculate spread
    // spread = offer_amount * ask_pool / offer_pool - return_amount
    let spread_amount = u256_math::div(u256_math::mul(offer_amount, ask_pool), offer_pool)
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate offer_amount {} * ask_pool {} / offer_pool {}",
            offer_amount.unwrap(),
            ask_pool.unwrap(),
            offer_pool.unwrap()
        )))?
        .saturating_sub(return_amount);

    Ok(SwapResult {
        return_amount: clamp(return_amount)?,
        spread_amount: clamp(spread_amount)?
    })
}

fn percentage_decrease(amount: U256, fee: Fee) -> StdResult<PercentageDecreaseResult> {
    let amount = Some(amount);
    let nom = Some(U256::from(fee.nom));
    let denom = Some(U256::from(fee.denom));

    let decrease_amount = u256_math::div(u256_math::mul(amount, nom), denom,)
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate amount {} * fee.nom {} / fee.denom {}",
            amount.unwrap(),
            nom.unwrap(),
            denom.unwrap()
        )))?;

    let result = u256_math::sub(amount, Some(decrease_amount))
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate amount {} - decrease_amount {}",
            amount.unwrap(),
            decrease_amount
        )))?;

    Ok(PercentageDecreaseResult {
        new_amount: clamp(result)?,
        decrease_amount: clamp(decrease_amount)?
    })
}

/// The amount the price moves in a trading pair between when a transaction is submitted and when it is executed.
/// Returns an `StdError` if the range of the expected tokens to be received is exceeded.
fn assert_slippage_tolerance(
    slippage: Option<Decimal>,
    deposits: &[Uint128; 2],
    pools: &[Uint128; 2]
) -> StdResult<()> {
    if slippage.is_none() {
        return Ok(());
    }

    let one_minus_slippage_tolerance = decimal_math::decimal_subtraction(Decimal::one(), slippage.unwrap())?;

    // Ensure each prices are not dropped as much as slippage tolerance rate
    if decimal_math::decimal_multiplication(
        Decimal::from_ratio(deposits[0], deposits[1]),
        one_minus_slippage_tolerance,
    ) > Decimal::from_ratio(pools[0], pools[1]) ||
    decimal_math::decimal_multiplication(
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

fn query_exchange_settings (
    querier: &impl Querier,
    factory: ContractInstance<HumanAddr>
) -> StdResult<ExchangeSettings<HumanAddr>> {
    let result: FactoryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        callback_code_hash: factory.code_hash,
        contract_addr: factory.address,
        msg: to_binary(&FactoryQueryMsg::GetExchangeSettings)?
    }))?;

    match result {
        FactoryResponse::GetExchangeSettings { settings } => Ok(settings),
        _ => Err(StdError::generic_err("An error occurred while trying to retrieve exchange settings."))
    }
}

fn clamp(val: U256) -> StdResult<Uint128> {
    if val > u128::MAX.into() {
        Err(StdError::generic_err(format!("cannot represent {} in 128 bits", &val)))
    } else {
        Ok(Uint128(val.low_u128()))
    }
}
