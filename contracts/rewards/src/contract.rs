use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, InitResponse, Querier,
    StdResult, Storage,
};
use composable_admin::admin::{
    save_admin, admin_handle, admin_query, DefaultHandleImpl, DefaultQueryImpl
};
use secret_toolkit::snip20;
use cosmwasm_utils::viewing_key::ViewingKey;

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{save_config, Config, add_pools};

const BLOCK_SIZE: usize = 256;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice());

    let admin = msg.admin.unwrap_or(env.message.sender);
    save_admin(deps, &admin)?;

    let config = Config {
        sienna_token: msg.sienna_token,
        viewing_key
    };

    save_config(deps, &config)?;

    add_pools(deps, msg.reward_pools)?;

    Ok(InitResponse {
        messages: vec![
            snip20::set_viewing_key_msg(
                config.viewing_key.0,
                None,
                BLOCK_SIZE,
                config.sienna_token.code_hash,
                config.sienna_token.address
            )?
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
        HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultHandleImpl)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl)
    }
}

/*
#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
}
*/
