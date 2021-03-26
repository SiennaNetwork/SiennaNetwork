use cosmwasm_std::{
    Api, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg, log, to_binary
};
use secret_toolkit::snip20;
use shared::{Callback, ContractInfo, IdoInitMsg, Snip20InitMsg, TokenType, U256};

use shared::u256_math;

use crate::msg::{HandleMsg, QueryMsg, QueryMsgResponse};
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
        rate: msg.rate,
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

    let mint_amount = calc_output_amount(amount, config.rate)?;

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
            mint_amount,
            None,
            BLOCK_SIZE,
            config.swapped_token.code_hash,
            config.swapped_token.address
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

fn get_rate<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> QueryResult {
    let config = load_config(deps)?;

    Ok(to_binary(&QueryMsgResponse::GetRate {
        rate: config.rate
    })?)
}

fn calc_output_amount(amount: Uint128, rate: Uint128) -> StdResult<Uint128> {
    // Technically the numbers here should be very far
    // from overflowing an Uint128 but who knows...
    
    let amount = Some(U256::from(amount.u128()));
    let rate = Some(U256::from(rate.u128()));

    let result = u256_math::mul(amount, rate).ok_or_else(|| 
        StdError::generic_err(format!("Couldn't calculate output_amount"))
    )?;

    // TODO: This 1_000_000_000 is hardcoded for now but shouldn't be.
    // It should have N zeroes = decimals of our token
    let result = u256_math::div(Some(result), Some(U256::from(1_000_000_000))).ok_or_else(||
        StdError::generic_err(format!("Couldn't calculate rate"))
    )?;

    Ok(Uint128(result.low_u128()))
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_output_amount() {
        // Assuming the user friendly (in the UI) exchange rate has been set to
        // 1 swapped_token (9 decimals) == 1.5 input_token (9 decimals):
        // the rate would be 1 / 1.5 = 0.(6) or 666666666 (0.(6) ** 10 * 9)
        // meaning the price for 1 whole swapped_token is
        // 1500000000 (1.5 * 10 ** 9 decimals) of input_token.

        // If we want to get 2 of swapped_token, we need to send 3 input_token
        // i.e. amount = 3000000000 (3 * 10 ** 9 decimals)

        let amount = Uint128(3000000000);
        let rate = Uint128(666666666);

        // TODO: the current formula works correctly only if both tokens have the same number of decimals
        let result = calc_output_amount(amount, rate).unwrap();
        assert_eq!(result, Uint128(1999999998));
    }
}

