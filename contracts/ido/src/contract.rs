use cosmwasm_std::{
    Api, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg, log, to_binary
};
use secret_toolkit::snip20;
use fadroma_scrt_callback::{Callback, ContractInstance};
use amm_shared::TokenType;
use amm_shared::msg::ido::{InitMsg, HandleMsg, QueryMsg, QueryResponse};
use amm_shared::msg::snip20::Snip20InitMsg;
use cosmwasm_utils::convert::{convert_token, get_whole_token_representation};

use crate::state::{Config, save_config, load_config, SwapConstants};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    if msg.info.snip20_init_info.decimals > 18 {
        return Err(StdError::generic_err("Decimals must not exceed 18"));
    }

    let input_token_decimals = match &msg.info.input_token {
        TokenType::NativeToken { .. } => 6,
        TokenType::CustomToken { contract_addr, token_code_hash } => {
            let result = snip20::token_info_query(
                &deps.querier,
                BLOCK_SIZE,
                token_code_hash.clone(),
                contract_addr.clone()
            )?;

            result.decimals
        }
    };

    let config = Config {
        input_token: msg.info.input_token,
        swap_constants: SwapConstants {
            swap_token_decimals: msg.info.snip20_init_info.decimals,
            rate: msg.info.rate,
            input_token_decimals,
            whole_swap_token: Uint128(get_whole_token_representation(msg.info.snip20_init_info.decimals)),
        },
        // We get this info when the instantiated SNIP20 calls HandleMsg::OnSnip20Init
        swap_token: ContractInstance {
            code_hash: msg.snip20_contract.code_hash.clone(),
            address: HumanAddr::default()
        },
        callback: Some(msg.callback)
    };

    save_config(deps, &config)?;

    let mut messages = vec![];

    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: msg.snip20_contract.id,
        callback_code_hash: msg.snip20_contract.code_hash,
        label: format!(
            "{}({})",
            msg.info.snip20_init_info.name.clone(),
            msg.info.snip20_init_info.symbol.clone()
        ),
        msg: to_binary(&Snip20InitMsg {
            name: msg.info.snip20_init_info.name,
            symbol: msg.info.snip20_init_info.symbol,
            decimals: msg.info.snip20_init_info.decimals,
            prng_seed: msg.info.snip20_init_info.prng_seed,
            config: msg.info.snip20_init_info.config,
            admin: Some(env.contract.address.clone()),
            initial_balances: None,
            callback: Some(Callback {
                msg: to_binary(&HandleMsg::OnSnip20Init)?,
                contract: ContractInstance {
                    address: env.contract.address.clone(),
                    code_hash: env.contract_code_hash
                }
            })
        })?,
        send: vec![]
    }));

    Ok(InitResponse{
        messages,
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
        HandleMsg::OnSnip20Init => on_snip20_init(deps, env)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    match msg {
        QueryMsg::GetRate => get_rate(deps)
    }
}

fn swap<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128
) -> StdResult<HandleResponse> {
    let config = load_config(deps)?;

    config.input_token.assert_sent_native_token_balance(&env, amount)?;

    let mint_amount = convert_token(
        amount.u128(),
        config.swap_constants.rate.u128(),
        config.swap_constants.input_token_decimals,
        config.swap_constants.swap_token_decimals
    )?;

    if mint_amount == 0 {
        return Err(StdError::generic_err(format!(
            "Insufficient amount provided: the swap did not succeed because 0 new tokens would be minted"
        )));
    }

    let mut messages = vec![];

    // If native token, the balance has been increased already
    if let TokenType::CustomToken { 
        contract_addr, token_code_hash 
    } = config.input_token {
        messages.push(snip20::transfer_from_msg(
            env.message.sender.clone(),     
            env.contract.address,
            amount,
            None,
            BLOCK_SIZE,
            token_code_hash,
            contract_addr
        )?);
    }

    // Mint new tokens and transfer to sender
    messages.push(
        snip20::mint_msg(
            env.message.sender,
            Uint128(mint_amount),
            None,
            BLOCK_SIZE,
            config.swap_token.code_hash,
            config.swap_token.address
        )?
    );

    Ok(HandleResponse {
        messages,
        log: vec![
            log("action", "swap"),
            log("input amount", amount),
            log("mint amount", mint_amount)
        ],
        data: None
    })
}

fn on_snip20_init<S: Storage, A: Api, Q: Querier>( 
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse> {
    let mut config = load_config(deps)?;
    
    //This should only be set once when the SNIP20 token is instantiated.
    if config.swap_token.address != HumanAddr::default() {
        return Err(StdError::generic_err("Invalid token type!"));
    }

    let mut messages = vec![];

    messages.push(snip20::register_receive_msg(
        env.contract_code_hash,
        None,
        BLOCK_SIZE,
        config.swap_token.code_hash.clone(),
        env.message.sender.clone()
    )?);

    let callback = config.callback.unwrap(); // Safe, because this function is executed only once

    // Register with factory
    messages.push(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: callback.contract.address,
            callback_code_hash: callback.contract.code_hash,
            msg: callback.msg,
            send: vec![],
        })
    );

    config.swap_token = ContractInstance {
        code_hash: config.swap_token.code_hash,
        address: env.message.sender.clone()
    };
    config.callback = None;

    save_config(deps, &config)?;

    Ok(HandleResponse {
        messages,
        log: vec![log("swapped_token address", env.message.sender.as_str())],
        data: None,
    })
}

fn get_rate<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = load_config(deps)?;

    Ok(to_binary(&QueryResponse::GetRate {
        rate: config.swap_constants.rate
    })?)
}

/*
#[cfg(test)]
mod tests {
    use super::*;
}
*/