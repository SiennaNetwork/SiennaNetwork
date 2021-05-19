use std::ops::Add;
use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse, Querier,
    StdError, Binary, StdResult, Storage, QueryResult, CosmosMsg, WasmMsg,
    Uint128, log, HumanAddr, Decimal, QueryRequest, WasmQuery
};
use secret_toolkit::snip20;
use sienna_amm_shared::{
    TokenPairAmount, TokenType,
    TokenTypeAmount, create_send_msg, Fee, ExchangeSettings,
    msg::{
        exchange::{InitMsg, HandleMsg, QueryMsg, QueryMsgResponse, SwapSimulationResponse},
        snip20::{Snip20InitConfig, Snip20InitMsg},
        factory::{QueryMsg as FactoryQueryMsg, QueryResponse as FactoryResponse},
        sienna_burner::HandleMsg as BurnerHandleMsg,
    },
    u256_math, u256_math::U256, viewing_key::ViewingKey, crypto::Prng
};

use crate::{state::{Config, store_config, load_config}, decimal_math};
use fadroma_scrt_migrate::{is_operational, can_set_status, set_status, get_status};
use fadroma_scrt_callback::{Callback, ContractInstance};

type SwapResult = StdResult<(Uint128, Uint128, Uint128)>;

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

    try_register_custom_token(&env, &mut messages, &msg.pair.0, &viewing_key)?;
    try_register_custom_token(&env, &mut messages, &msg.pair.1, &viewing_key)?;

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
            prng_seed: Binary::from(rng.rand_bytes()),
            config: Some(Snip20InitConfig {
                public_total_supply: Some(true)
            })
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
        viewing_key,
        pool_cache: [ Uint128::zero(), Uint128::zero() ]
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
    let config = load_config(deps)?;
    match msg {
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::PairInfo => query_pair_info(config),
        QueryMsg::FactoryInfo => to_binary(&QueryMsgResponse::FactoryInfo(config.factory_info)),
        QueryMsg::Pool => query_pool_amount(deps, config),
        QueryMsg::SwapSimulation { offer } => swap_simulation(deps, config, offer)
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
            log("withdrawn_share", amount.to_string()),
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
    let mut messages = vec![];

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

    let mut config = load_config(deps)?;

    let settings = query_exchange_settings(&deps.querier, config.factory_info.clone())?;

    let (mut return_amount, spread_amount, mut commission_amount) = do_swap(deps, &mut config, &offer, settings.swap_fee, false)?;
    store_config(deps, &config)?; // Save in order to update the pool_cache

    if let Some(expected_return) = expected_return {
        if return_amount.lt(&expected_return) {
            return Err(StdError::generic_err(
                "Operation fell short of expected_return",
            ));
        }
    }

    // SIENNA needs to be burned on each swap in order to decrease the total supply
    if let Some(info) = settings.sienna_burner {
        let (result, decrease_amount) = percentage_decrease(U256::from(return_amount.u128()), settings.sienna_fee)?;
        let decrease_amount = clamp(decrease_amount)?;

        commission_amount = commission_amount + decrease_amount;
        return_amount = clamp(result)?;

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            callback_code_hash: info.code_hash,
            contract_addr: info.address,
            msg: to_binary(&BurnerHandleMsg::Burn {
                amount: decrease_amount
            })?,
            send: vec![]
        }))
    }

    let index = config.pair.get_token_index(&offer.token).unwrap(); // Safe, checked in do_swap
    let token = config.pair.get_token(index ^ 1).unwrap();

    messages.push(create_send_msg(&token, env.contract.address, env.message.sender, return_amount)?);

    Ok(HandleResponse{
        messages,
        log: vec![
            log("action", "swap"),
            log("offer_token", offer.token.to_string()),
            log("offer_amount", offer.amount.to_string()),
            log("return_amount", return_amount.to_string()),
            log("spread_amount", spread_amount.to_string()),
            log("commission_amount", commission_amount.to_string())
        ],
        data: None
    })
}

fn query_pool_amount<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: Config<HumanAddr>
) -> QueryResult {
    let result = config.pair.query_balances(&deps.querier, config.contract_addr, config.viewing_key.0)?;

    to_binary(&QueryMsgResponse::Pool(
        TokenPairAmount {
            pair: config.pair,
            amount_0: result[0],
            amount_1: result[1]
        }
    ))
}

fn query_pair_info(
    config: Config<HumanAddr>
) -> QueryResult {
    to_binary(&QueryMsgResponse::PairInfo{
        pair: config.pair,
        liquidity_token: config.lp_token_info
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
    mut config: Config<HumanAddr>,
    offer: TokenTypeAmount<HumanAddr>
) -> QueryResult {
    let settings = query_exchange_settings(&deps.querier, config.factory_info.clone())?;

    let (mut return_amount, spread_amount, mut commission_amount) = do_swap(deps, &mut config, &offer, settings.swap_fee, true)?;

    if let Some(_) = settings.sienna_burner {
        let (result, decrease_amount) = percentage_decrease(U256::from(return_amount.u128()), settings.sienna_fee)?;
        let decrease_amount = clamp(decrease_amount)?;

        commission_amount += decrease_amount;
        return_amount = clamp(result)?;
    }

    to_binary(&SwapSimulationResponse { return_amount, spread_amount, commission_amount })
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
        log: vec![log("liquidity_token_addr", env.message.sender.as_str())],
        data: None,
    })
}

fn try_register_custom_token(
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
    config: &mut Config<HumanAddr>,
    offer: &TokenTypeAmount<HumanAddr>,
    fee: Fee,
    is_simulation: bool
) -> SwapResult {
    if !config.pair.contains(&offer.token) {
        return Err(StdError::generic_err(format!("The supplied token {}, is not managed by this contract.", offer.token)));
    }

    let balances = config.pair.query_balances(&deps.querier, config.contract_addr.clone(), config.viewing_key.0.clone())?;
    let token_index = config.pair.get_token_index(&offer.token).unwrap(); //Safe because we checked above for existence
    
    let mut offer_pool = balances[token_index];

    if !is_simulation {
        config.pool_cache[token_index] = config.pool_cache[token_index].add(offer.amount);

        // If offer.token is not native, the balance hasn't been increased yet
        if let TokenType::NativeToken { .. } = offer.token {
            let result = U256::from(offer_pool.u128()).checked_sub(U256::from(offer.amount.u128()))
                .ok_or(StdError::generic_err("This can't really happen."))?;

            offer_pool = clamp(result)?
        }
    }

    compute_swap(
        offer_pool,
        balances[token_index ^ 1],
        offer.amount,
        fee
    )
}

// Copied from https://github.com/enigmampc/SecretSwap/blob/ffd72d1c94096ac3a78aaf8e576f22584f49fe7a/contracts/secretswap_pair/src/contract.rs#L768
fn compute_swap(
    offer_pool: Uint128,
    ask_pool: Uint128,
    offer_amount: Uint128,
    fee: Fee
) -> SwapResult {
    // offer => ask
    let offer_pool = Some(U256::from(offer_pool.u128()));
    let ask_pool = Some(U256::from(ask_pool.u128()));
    let offer_amount = Some(U256::from(offer_amount.u128()));

    // cp = offer_pool * ask_pool
    let cp = u256_math::mul(offer_pool, ask_pool).ok_or(StdError::generic_err(format!(
        "Cannot calculate cp = offer_pool {} * ask_pool {}",
        offer_pool.unwrap(),
        ask_pool.unwrap()
    )))?;

    // return_amount = (ask_pool - cp / (offer_pool + offer_amount))
    // ask_amount = return_amount * (1 - commission_rate)
    let return_amount = u256_math::sub(ask_pool, u256_math::div(Some(cp), u256_math::add(offer_pool, offer_amount)))
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate return_amount = (ask_pool {} - cp {} / (offer_pool {} + offer_amount {}))",
            ask_pool.unwrap(),
            cp,
            offer_pool.unwrap(),
            offer_amount.unwrap(),
        )))?;

    // calculate spread & commission
    // spread = offer_amount * ask_pool / offer_pool - return_amount
    let spread_amount = u256_math::div(u256_math::mul(offer_amount, ask_pool), offer_pool)
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate offer_amount {} * ask_pool {} / offer_pool {}",
            offer_amount.unwrap(),
            ask_pool.unwrap(),
            offer_pool.unwrap()
        )))?
        .saturating_sub(return_amount);

    // commission_amount = return_amount * fee.nom / fee.denom
    let (return_amount, commission_amount) = percentage_decrease(return_amount, fee)?;

    Ok((clamp(return_amount)?, clamp(spread_amount)?, clamp(commission_amount)?))
}

fn percentage_decrease(amount: U256, fee: Fee) -> StdResult<(U256, U256)> {
    let amount = Some(amount);
    let nom = Some(U256::from(fee.nom));
    let denom = Some(U256::from(fee.denom));

    let decrease_amount = u256_math::div(u256_math::mul(amount, nom), denom,)
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate return_amount {} * commission_rate_nom {} / commission_rate_denom {}",
            amount.unwrap(),
            nom.unwrap(),
            denom.unwrap()
        )))?;

    let result = u256_math::sub(amount, Some(decrease_amount))
        .ok_or(StdError::generic_err(format!(
            "Cannot calculate return_amount {} - commission_amount {}",
            amount.unwrap(),
            decrease_amount
        )))?;

    Ok((result, decrease_amount))
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

/*
#[cfg(test)]
mod tests {
    use super::*;
*/

fn clamp(val: U256) -> StdResult<Uint128> {
    if val > u128::MAX.into() {
        Err(StdError::generic_err(format!("cannot represent {} in 128 bits", &val)))
    } else {
        Ok(Uint128(val.low_u128()))
    }
}
