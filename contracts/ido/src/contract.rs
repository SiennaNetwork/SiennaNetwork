use cosmwasm_std::{
    Api, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, QueryResult, StdError, StdResult,
    Storage, Uint128, WasmMsg, log, to_binary
};
use secret_toolkit::snip20;
use amm_shared::TokenType;
use amm_shared::fadroma::callback::{ContractInstance};
use amm_shared::msg::ido::{InitMsg, HandleMsg, QueryMsg, QueryResponse};
use amm_shared::fadroma::utils::convert::convert_token;
use amm_shared::admin::admin::{
    DefaultHandleImpl, DefaultQueryImpl, save_admin, admin_handle,
    admin_query
};

use crate::state::{
    Config, SwapConstants, load_config,
    save_config
};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    _env: Env,
    msg: InitMsg
) -> StdResult<InitResponse> {
    let input_token_decimals = match &msg.info.input_token {
        TokenType::NativeToken { .. } => 6,
        TokenType::CustomToken { contract_addr, token_code_hash } => {
            get_token_decimals(&deps.querier, ContractInstance {
                address: contract_addr.clone(),
                code_hash: token_code_hash.clone()
            })?
        }
    };

    save_admin(deps, &msg.admin)?;

    let config = Config {
        input_token: msg.info.input_token,
        sold_token: msg.info.sold_token.clone(),
        swap_constants: SwapConstants {
            sold_token_decimals: get_token_decimals(&deps.querier, msg.info.sold_token)?,
            rate: msg.info.rate,
            input_token_decimals
        }
    };

    save_config(deps, &config)?;

    Ok(InitResponse{
        messages: vec![
            // Execute the HandleMsg::RegisterIdo method of
            // the factory contract in order to register this address
            CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: msg.callback.contract.address,
                callback_code_hash: msg.callback.contract.code_hash,
                msg: msg.callback.msg,
                send: vec![]
            })
        ],
        log: vec![]
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Swap { amount } => swap(deps, env, amount),
        HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultHandleImpl)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    match msg {
        QueryMsg::GetRate => get_rate(deps),
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl)
    }
}

fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128
) -> StdResult<HandleResponse> {
    let config = load_config(deps)?;

    let mint_amount = convert_token(
        amount.u128(),
        config.swap_constants.rate.u128(),
        config.swap_constants.input_token_decimals,
        config.swap_constants.sold_token_decimals
    )?;

    if mint_amount == 0 {
        return Err(StdError::generic_err(format!(
            "Insufficient amount provided: the swap did not succeed because 0 new tokens would be minted."
        )));
    }

    let mut messages = vec![];

    match config.input_token {
        TokenType::CustomToken { contract_addr, token_code_hash } => {
            messages.push(snip20::transfer_from_msg(
                env.message.sender.clone(),     
                env.contract.address,
                amount,
                None,
                BLOCK_SIZE,
                token_code_hash,
                contract_addr
            )?);
        },
        TokenType::NativeToken { .. } => {
            config.input_token.assert_sent_native_token_balance(&env, amount)?;
        }
    }

    // Mint new tokens and transfer to sender
    messages.push(
        snip20::mint_msg(
            env.message.sender,
            Uint128(mint_amount),
            None,
            BLOCK_SIZE,
            config.sold_token.code_hash,
            config.sold_token.address
        )?
    );

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "swap"),
            log("input_amount", amount),
            log("mint_amount", mint_amount)
        ],
        data: None
    })
}

fn get_rate<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = load_config(deps)?;

    Ok(to_binary(&QueryResponse::GetRate {
        rate: config.swap_constants.rate
    })?)
}

fn get_token_decimals(
    querier: &impl Querier,
    instance: ContractInstance<HumanAddr>
) -> StdResult<u8> {
    let result = snip20::token_info_query(
        querier,
        BLOCK_SIZE,
        instance.code_hash,
        instance.address
    )?;

    Ok(result.decimals)
}
/*
#[cfg(test)]
mod tests {
    use super::*;
}
*/
