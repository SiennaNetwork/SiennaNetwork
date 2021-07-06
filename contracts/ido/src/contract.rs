use amm_shared::admin::admin::{
    admin_handle, admin_query, assert_admin, save_admin, DefaultHandleImpl, DefaultQueryImpl,
};
use amm_shared::msg::ido::{HandleMsg, InitMsg, QueryMsg, QueryResponse};
use amm_shared::TokenType;
use fadroma::scrt::callback::ContractInstance;
use fadroma::scrt::cosmwasm_std::{
    log, to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse, Querier,
    QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg,
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

fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128,
) -> StdResult<HandleResponse> {
    let mut account = Account::<HumanAddr>::load_self(deps, &env.message.sender)?;
    let config = Config::<HumanAddr>::load_self(&deps)?;
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
            config.min_allocation
        )));
    }

    account.save(deps)?;

    let mut messages = vec![];

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

    // Transfer the resulting amount to the sender
    messages.push(snip20::transfer_msg(
        env.message.sender,
        Uint128(mint_amount),
        None,
        BLOCK_SIZE,
        config.sold_token.code_hash,
        config.sold_token.address,
    )?);

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "swap"),
            log("input_amount", amount),
            log("purchased_amount", mint_amount),
        ],
        data: None,
    })
}

/// After the contract has ended, admin can ask for a return of his tokens.
fn refund<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    env: Env,
) -> StdResult<HandleResponse> {
    assert_admin(&deps, &env)?;
    let config = Config::<HumanAddr>::load_self(&deps)?;
    config.is_refundable(env.block.time)?;

    let mut messages = vec![];

    // TODO: calculate the amount swapped and deduct it from total possible amount to be paid,
    // that is the left over that needs to be returned to admin of the contract.

    let refund_amount = 0;
    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "refund"),
            log("refunded_amount", refund_amount),
        ],
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
/*
#[cfg(test)]
mod tests {
    use super::*;
}
*/
