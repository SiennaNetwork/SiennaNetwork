use amm_shared::admin::require_admin;
use amm_shared::{
    admin::admin::{assert_admin, load_admin},
    fadroma::scrt::{
        addr::Canonize,
        cosmwasm_std::{
            from_binary, log, Api, BankMsg, Binary, CanonicalAddr, Coin, Env, Extern,
            HandleResponse, HumanAddr, Querier, StdError, StdResult, Storage, Uint128,
        },
        storage::Storable,
        toolkit::snip20,
        utils::convert::convert_token,
        BLOCK_SIZE,
    },
    msg::ido::{ReceiverCallbackMsg, SaleType},
    TokenType,
};

use crate::data::{
    increment_total_pre_lock_amount, load_viewing_key, Account, Config, SaleSchedule,
};

use crate::helpers::*;

/// Universal handler for receive callback from snip20 interface of sold token and possibly custom input token
pub(crate) fn receive_callback<S: Storage, A: Api, Q: Querier>(
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
        ReceiverCallbackMsg::Activate {
            start_time,
            end_time,
        } => {
            // If the sender is sold_token, we will treat this like activation
            // handle call that will activate the contract if enough funds is sent
            if env.message.sender == config.sold_token.address {
                return activate(deps, env, from, amount, config, start_time, end_time);
            }
        }
        ReceiverCallbackMsg::PreLock {} => {
            if let TokenType::CustomToken { contract_addr, .. } = &config.input_token {
                if env.message.sender == *contract_addr {
                    return pre_lock(deps, env.block.time, config, amount, from);
                }
            }
        }
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
pub(crate) fn activate<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    from: HumanAddr,
    amount: Uint128,
    mut config: Config<HumanAddr>,
    start: Option<u64>,
    end: u64,
) -> StdResult<HandleResponse> {
    if load_admin(deps)? != from {
        return Err(StdError::unauthorized());
    }

    let required_amount = config.total_allocation();
    let token_balance = get_token_balance(
        &deps,
        env.contract.address,
        config.sold_token.clone(),
        load_viewing_key(&deps.storage)?,
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

/// Pre lock input token before the sale has started
/// Checks if the account is whitelisted
/// Checks if the sold token is currently swapable (sale has started and has not yet ended)
/// Checks if the account hasn't gone over the sale limit and is above the sale minimum.
pub(crate) fn pre_lock<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    now: u64,
    config: Config<HumanAddr>,
    amount: Uint128,
    from: HumanAddr,
) -> StdResult<HandleResponse> {
    let schedule = config
        .schedule
        .ok_or_else(|| StdError::generic_err("Contract is not yet active"))?;

    if SaleType::PreLockAndSwap != config.sale_type && SaleType::PreLockOnly != config.sale_type {
        return Err(StdError::generic_err(
            "Pre-lock option is not enabled for this sale.",
        ));
    }

    if schedule.has_started(now) {
        return Err(StdError::generic_err(
            "Sale has already started, use swap instead.",
        ));
    }

    let mut account = Account::<CanonicalAddr>::load_self(&deps, &from)?;

    let single_amount = convert_token(
        amount.u128(),
        config.swap_constants.rate.u128(),
        config.swap_constants.input_token_decimals,
        config.swap_constants.sold_token_decimals,
    )?;

    let total_amount = convert_token(
        (account.pre_lock_amount + amount).u128(),
        config.swap_constants.rate.u128(),
        config.swap_constants.input_token_decimals,
        config.swap_constants.sold_token_decimals,
    )?;

    if single_amount < config.min_allocation.u128() {
        return Err(StdError::generic_err(format!(
            "Insufficient amount provided: the resulting amount fell short of the minimum purchase expected: {}, got: {}",
            config.min_allocation,
            single_amount
        )));
    }

    if total_amount > config.max_allocation.u128() {
        return Err(StdError::generic_err(format!(
            "This purchase exceeds the total maximum allowed amount for a single address: {}",
            config.max_allocation
        )));
    }

    account.pre_lock_amount += amount;

    account.save(deps)?;

    // Save the amount that was pre locked
    increment_total_pre_lock_amount(deps, Uint128(single_amount))?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "pre_lock"),
            log("input_amount", amount),
            log("pre_lock_amount", account.pre_lock_amount),
        ],
        data: None,
    })
}

/// Swap input token for sold token.
/// Checks if the account is whitelisted
/// Checks if the sold token is currently swapable (sale has started and has not yet ended)
/// Checks if the account hasn't gone over the sale limit and is above the sale minimum.
pub(crate) fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    now: u64,
    config: Config<HumanAddr>,
    amount: Uint128,
    from: HumanAddr,
    recipient: Option<HumanAddr>,
) -> StdResult<HandleResponse> {
    config.is_swapable(now)?;

    let mut account = Account::<CanonicalAddr>::load_self(&deps, &from)?;

    if SaleType::PreLockAndSwap != config.sale_type
        && SaleType::SwapOnly != config.sale_type
        && amount.u128() > 0 as u128
    {
        return Err(StdError::generic_err(
            "This sale was only a pre-lock sale, please send 0 amount and you will get tokens for your pre-locked amount.",
        ));
    }

    let amount = account.pre_lock_amount + amount;

    let mint_amount = convert_token(
        amount.u128(),
        config.swap_constants.rate.u128(),
        config.swap_constants.input_token_decimals,
        config.swap_constants.sold_token_decimals,
    )?;

    if mint_amount < config.min_allocation.u128() {
        return Err(StdError::generic_err(format!(
            "Insufficient amount provided: the resulting amount fell short of the minimum purchase expected: {}, got: {} for {}. Rate: {}. Input decimals: {}. Sold decimals: {}",
            config.min_allocation,
            mint_amount,
            amount,
            config.swap_constants.rate.u128(),
            config.swap_constants.input_token_decimals,
            config.swap_constants.sold_token_decimals,
        )));
    }

    account.total_bought = account
        .total_bought
        .u128()
        .checked_add(mint_amount)
        .ok_or_else(|| StdError::generic_err("Upper bound overflow detected."))?
        .into();

    if account.total_bought > config.max_allocation {
        return Err(StdError::generic_err(format!(
            "This purchase exceeds the total maximum allowed amount for a single address: {}",
            config.max_allocation
        )));
    }

    account.pre_lock_amount = Uint128(0_u128);

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
pub(crate) fn refund<S: Storage, A: Api, Q: Querier>(
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
        load_viewing_key(&deps.storage)?,
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
pub(crate) fn claim<S: Storage, A: Api, Q: Querier>(
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
        load_viewing_key(&deps.storage)?.to_string(),
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

/// Add new address to whitelist
pub(crate) fn add_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    let admin = load_admin(deps)?;
    let mut config = Config::<CanonicalAddr>::load_self(&deps)?;

    // Only admin of the IDO contract and launchpad contract can access this action
    if let Some(launchpad) = &config.launchpad {
        if admin != env.message.sender && launchpad.address != env.message.sender {
            return Err(StdError::unauthorized());
        }
    } else if admin != env.message.sender {
        return Err(StdError::unauthorized());
    }

    if let Some(schedule) = config.schedule {
        if schedule.has_ended(env.block.time) {
            return Err(StdError::generic_err(
                "Cannot whitelist addresses after the sale has finished.",
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
            log("new_addresses", added_count),
        ],
        data: None,
    })
}
