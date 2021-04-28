use cosmwasm_std::{
    to_binary, Api, Binary, Env, Extern, HandleResponse, HumanAddr,
    InitResponse, Querier, StdError, StdResult, Storage, Uint128, log
};

use sienna_amm_shared::msg::sienna_burner::{HandleMsg, InitMsg, QueryAnswer, QueryMsg, ResponseStatus};
use sienna_amm_shared::ContractInfo;
use sienna_amm_shared::snip20;

use crate::state::*;

const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    save_token_info(deps, &msg.sienna_token)?;
    save_burn_pool(deps, &msg.burn_pool)?;
    
    let admins = if let Some(mut admins) = msg.admins {
        admins.push(msg.factory_address);
        admins
    } else {
        vec![ msg.factory_address, env.message.sender ]
    };

    save_admins(deps, &admins)?;

    if let Some(pairs) = msg.pairs {
        save_pair_addresses(deps, &pairs)?;
    }

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::Burn {
            amount
        } => burn(deps, env, amount),
        HandleMsg::AddPairs { pairs } => add_pairs(deps, env, pairs),
        HandleMsg::RemovePairs { pairs } => remove_pairs(deps, env, pairs),
        HandleMsg::AddAdmins { addresses } => add_admins(deps, env, addresses),
        HandleMsg::SetBurnPool {address } => set_burn_pool(deps, env, address),
        HandleMsg::SetSiennaToken { info } => set_token(deps, env, info)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::BurnPool => query_burn_pool(deps),
        QueryMsg::Admins => query_admins(deps),
        QueryMsg::SiennaToken => query_token(deps)
    }
}

fn burn<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    amount: Uint128
) -> StdResult<HandleResponse> {
    if !pair_address_exists(&deps, &env.message.sender)? {
        return Err(StdError::unauthorized());
    }

    let burn_pool = load_burn_pool(&deps)?;
    let sienna_token = load_token_info(&deps)?;

    Ok(HandleResponse {
        messages: vec![
            snip20::burn_from_msg(
                burn_pool,
                amount,
                None,
                BLOCK_SIZE,
                sienna_token.code_hash,
                sienna_token.address
            )?
        ],
        log: vec![
            log("sienna_burned", amount)
        ],
        data: Some(to_binary(&ResponseStatus::Success)?),
    })
}

fn add_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pairs: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    enforce_admin(deps, env)?;
    save_pair_addresses(deps, &pairs)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&ResponseStatus::Success)?),
    })
}

fn remove_pairs<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pairs: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    enforce_admin(deps, env)?;
    remove_pair_addresses(deps, &pairs)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&ResponseStatus::Success)?),
    })
}

fn add_admins<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    enforce_admin(deps, env)?;
    save_admins(deps, &addresses)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&ResponseStatus::Success)?),
    })
}

fn set_burn_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    enforce_admin(deps, env)?;
    save_burn_pool(deps, &address)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&ResponseStatus::Success)?),
    })
}

fn set_token<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: ContractInfo,
) -> StdResult<HandleResponse> {
    enforce_admin(deps, env)?;
    save_token_info(deps, &info)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![],
        data: Some(to_binary(&ResponseStatus::Success)?),
    })
}

fn enforce_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
) -> StdResult<()> {
    let admins = load_admins(deps)?;

    if admins.contains(&env.message.sender) {
        return Ok(());
    }

    Err(StdError::unauthorized())
}

fn query_burn_pool<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let address = load_burn_pool(deps)?;

    to_binary(&QueryAnswer::BurnPool {
        address,
    })
}

fn query_admins<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let addresses = load_admins(deps)?;

    to_binary(&QueryAnswer::Admins { 
        addresses
    })
}

fn query_token<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let info = load_token_info(deps)?;

    to_binary(&QueryAnswer::SiennaToken { 
        info 
    })
}
