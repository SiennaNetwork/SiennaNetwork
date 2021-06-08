use cosmwasm_std::{
    Api, CosmosMsg, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, QueryResult, StdError, StdResult, Storage, Uint128, WasmMsg, log, to_binary
};
use secret_toolkit::snip20;
use sienna_amm_shared::{Callback, ContractInfo, TokenType};
use sienna_amm_shared::msg::ido::{IdoInitMsg, HandleMsg, QueryMsg, QueryMsgResponse};
use sienna_amm_shared::msg::snip20::Snip20InitMsg;
use sienna_amm_shared::u256_math;
use sienna_amm_shared::u256_math::U256;

use crate::state::{Config, save_config, load_config, SwapConstants};

/// Pad handle responses and log attributes to blocks
/// of 256 bytes to prevent leaking info based on response size
const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: IdoInitMsg,
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
            whole_swap_token: get_whole_token_representation(msg.info.snip20_init_info.decimals),
        },
        // We get this info when the instantiated SNIP20 calls HandleMsg::OnSnip20Init
        swap_token: ContractInfo {
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
                contract: ContractInfo {
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

    let mint_amount = calc_output_amount(amount, config.swap_constants)?;

    if mint_amount.u128() == 0 {
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
            mint_amount,
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

    config.swap_token = ContractInfo {
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

    Ok(to_binary(&QueryMsgResponse::GetRate {
        rate: config.swap_constants.rate
    })?)
}

fn calc_output_amount(amount: Uint128, constants: SwapConstants) -> StdResult<Uint128> {
    // Technically the numbers here should be very far
    // from overflowing an Uint128 but who knows...

    let err_msg = "An error occurred when calculating the amount of new tokens to be minted.";

    let amount = Some(U256::from(amount.u128()));
    let rate = Some(U256::from(constants.rate.u128()));
    
    // result amount * rate / one whole swap_token (constants.whole_swap_token)
    let mut result = u256_math::mul(amount, rate).ok_or_else(|| 
        StdError::generic_err(err_msg)
    )?;

    // But, if tokens have different number of decimals, we need to compensate either by 
    // dividing or multiplying (depending on which token has more decimals) the difference
    if constants.input_token_decimals < constants.swap_token_decimals {
        let compensation = get_whole_token_representation(
            constants.swap_token_decimals - constants.input_token_decimals
        );
        let compensation = Some(U256::from(compensation.u128()));

        result = u256_math::mul(Some(result), compensation).ok_or_else(|| 
            StdError::generic_err(err_msg) 
        )?;
    } else if constants.swap_token_decimals < constants.input_token_decimals {
        let compensation = get_whole_token_representation(
            constants.input_token_decimals - constants.swap_token_decimals
        );
        let compensation = Some(U256::from(compensation.u128()));

        result = u256_math::div(Some(result), compensation).ok_or_else(|| 
            StdError::generic_err(err_msg) 
        )?;
    }

    let whole_token = Some(U256::from(constants.whole_swap_token.u128()));
    let result = u256_math::div(Some(result), whole_token).ok_or_else(||
        StdError::generic_err(err_msg)
    )?;

    Ok(Uint128(result.low_u128()))
}

/// Get the amount needed to represent 1 whole token given its decimals.
/// Ex. Given token A that has 3 decimals, 1 A == 1000
fn get_whole_token_representation(decimals: u8) -> Uint128 {
    let mut whole_token = 1u128;

    for _ in 0..decimals {
        whole_token *= 10;
    };

    Uint128(whole_token)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_output_amount() {
        fn create_constants(input_token_decimals: u8, swap_token_decimals: u8, rate: Uint128) -> SwapConstants {
            SwapConstants { 
                whole_swap_token: get_whole_token_representation(swap_token_decimals),
                rate,
                input_token_decimals,
                swap_token_decimals
            }
        }

        // Assuming the user friendly (in the UI) exchange rate has been set to
        // 1 swapped_token (9 decimals) == 1.5 input_token (9 decimals):
        // the rate would be 1 / 1.5 = 0.(6) or 666666666 (0.(6) ** 10 * 9)
        // meaning the price for 1 whole swapped_token is
        // 1500000000 (1.5 * 10 ** 9 decimals) of input_token.

        // If we want to get 2 of swapped_token, we need to send 3 input_token
        // i.e. amount = 3000000000 (3 * 10 ** 9 decimals)

        let constants = create_constants(9, 9, Uint128(666_666_666));
        let amount = Uint128(3_000_000_000);

        let result = calc_output_amount(amount, constants).unwrap();
        assert_eq!(result, Uint128(1_999_999_998));

        // Should work the same even if input_token has less decimals (ex. 6)
        let constants = create_constants(6, 9, Uint128(666_666_666));

        // Here amount has 3 zeroes less because input_token now has 6 decimals, so
        // 1 input_token = 3000000 (3 * 10 ** 6)
        let amount = Uint128(3_000_000);

        let result = calc_output_amount(amount, constants).unwrap();
        assert_eq!(result, Uint128(1_999_999_998));

        // And the other way around - when swap_token has 6 decimals.
        // Here the rate and result have 3 less digits - to account for the less decimals
        let constants = create_constants(9, 6, Uint128(666_666));
        let amount = Uint128(3_000_000_000);

        let result = calc_output_amount(amount, constants).unwrap();
        assert_eq!(result, Uint128(1_999_998));
    }
}
