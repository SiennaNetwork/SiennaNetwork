use cosmwasm_std::{
    to_binary, Api, Env, Extern, HandleResponse, InitResponse, Querier, StdError,
    StdResult, Storage, QueryResult, HumanAddr, CosmosMsg, WasmMsg, log
};
use secret_toolkit::snip20;
use shared::{Callback, ContractInfo, IdoInitMsg, Snip20InitMsg};

use crate::msg::{HandleMsg, QueryMsg};
use crate::state::{Config, save_config, load_config};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: IdoInitMsg,
) -> StdResult<InitResponse> {
    let config = Config {
        input_token: msg.input_token,
        swap_ratio: msg.swap_ratio,
        // We get this info when the instantiated SNIP20 calls HandleMsg::OnSnip20Init
        swapped_token: ContractInfo {
            code_hash: msg.snip20_contract.code_hash.clone(),
            address: HumanAddr::default()
        }
    };

    save_config(deps, &config)?;

    let mut messages = vec![];

    messages.push(CosmosMsg::Wasm(WasmMsg::Instantiate {
        code_id: msg.snip20_contract.id,
        callback_code_hash: msg.snip20_contract.code_hash,
        label: format!(
            "{}({})",
            msg.snip20_init_info.name.clone(),
            msg.snip20_init_info.symbol.clone()
        ),
        msg: to_binary(&Snip20InitMsg {
            name: msg.snip20_init_info.name,
            symbol: msg.snip20_init_info.symbol,
            decimals: msg.snip20_init_info.decimals,
            prng_seed: msg.snip20_init_info.prng_seed,
            config: msg.snip20_init_info.config,
            admin: Some(env.contract.address.clone()),
            initial_balances: None,
            callback: Some(Callback {
                msg: to_binary(&HandleMsg::OnSnip20Init)?,
                contract_addr: env.contract.address.clone(),
                contract_code_hash: env.contract_code_hash
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
        HandleMsg::OnSnip20Init => on_snip20_init(deps, env)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> QueryResult {
    unimplemented!();
}

fn on_snip20_init<S: Storage, A: Api, Q: Querier>( 
    deps: &mut Extern<S, A, Q>,
    env: Env
) -> StdResult<HandleResponse> {
    let mut config = load_config(deps)?;
    
    //This should only be set once when the SNIP20 token is instantiated.
    if config.swapped_token.address != HumanAddr::default() {
        return Err(StdError::generic_err("Invalid token type!"));
    }

    config.swapped_token = ContractInfo {
        code_hash: config.swapped_token.code_hash,
        address: env.message.sender.clone()
    };

    save_config(deps, &config)?;

    Ok(HandleResponse {
        messages: vec![snip20::register_receive_msg(
            env.contract_code_hash,
            None,
            BLOCK_SIZE,
            config.swapped_token.code_hash,
            env.message.sender.clone(),
        )?],
        log: vec![log("swapped_token address", env.message.sender.as_str())],
        data: None,
    })
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

}
*/
