use amm_shared::{
    admin::admin::{admin_handle, admin_query, save_admin, DefaultHandleImpl, DefaultQueryImpl},
    auth::{auth_handle, AuthHandleMsg, DefaultHandleImpl as AuthHandle},
    fadroma::scrt::{
        callback::ContractInstance,
        cosmwasm_std::{
            log, to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, InitResponse, Querier,
            QueryResult, StdError, StdResult, Storage, WasmMsg,
        },
        migrate as fadroma_scrt_migrate,
        storage::Storable,
        toolkit::snip20,
        utils::viewing_key::ViewingKey,
        BLOCK_SIZE,
    },
    msg::launchpad::{HandleMsg, InitMsg, QueryMsg},
    TokenType,
};
use fadroma_scrt_migrate::{get_status, with_status};

use crate::data::{save_contract_address, save_viewing_key, Config, TokenConfig};
use crate::helpers::*;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    save_contract_address(deps, &env.contract.address)?;
    let viewing_key = ViewingKey::new(&env, msg.prng_seed.as_slice(), msg.entropy.as_slice());

    let mut messages = vec![];
    let mut config = Config { tokens: vec![] };

    for token in msg.tokens {
        match token.token_type {
            TokenType::CustomToken {
                contract_addr,
                token_code_hash,
            } => {
                // Set created viewing key onto the contract so we can check the balance later
                messages.push(snip20::set_viewing_key_msg(
                    viewing_key.to_string(),
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?);

                // Register this contract as a receiver of the callback messages
                // from the custom input token. This will allow us to receive
                // message after the tokens have been sent and will make the lock
                // happen in a single transaction
                messages.push(snip20::register_receive_msg(
                    env.contract_code_hash.clone(),
                    None,
                    BLOCK_SIZE,
                    token_code_hash.clone(),
                    contract_addr.clone(),
                )?);

                // Get the number of token decimals
                let token_decimals = get_token_decimals(
                    &deps.querier,
                    ContractInstance {
                        address: contract_addr.clone(),
                        code_hash: token_code_hash.clone(),
                    },
                )?;

                config.tokens.push(TokenConfig {
                    token_type: TokenType::CustomToken {
                        contract_addr,
                        token_code_hash,
                    },
                    segment: token.segment,
                    bounding_period: token.bounding_period,
                    active: true,
                    token_decimals,
                });
            }
            _ => config.tokens.push(TokenConfig {
                token_type: token.token_type,
                segment: token.segment,
                bounding_period: token.bounding_period,
                active: true,
                token_decimals: 6,
            }),
        }
    }

    save_viewing_key(&mut deps.storage, &viewing_key)?;
    save_admin(deps, &msg.admin)?;
    config.save(deps)?;

    // Execute the HandleMsg::RegisterLaunchpad method of
    // the factory contract in order to register this address
    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: msg.callback.contract.address,
        callback_code_hash: msg.callback.contract.code_hash,
        msg: msg.callback.msg,
        send: vec![],
    }));

    Ok(InitResponse {
        messages,
        log: vec![log("viewkey", viewing_key.to_string())],
    })
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    with_status!(
        deps,
        env,
        match msg {
            HandleMsg::Receive {
                from, amount, msg, ..
            } => crate::handle::receive_callback(deps, env, from, amount, msg),
            HandleMsg::Lock { amount } => {
                let token_config = Config::load_self(deps)?.get_token_config(None)?;
                token_config
                    .token_type
                    .assert_sent_native_token_balance(&env, amount)?;

                crate::handle::lock(deps, env, None, token_config, amount)
            }
            HandleMsg::Unlock { entries } => {
                let token_config = Config::load_self(deps)?.get_token_config(None)?;
                crate::handle::unlock(deps, env, None, token_config, entries)
            }
            HandleMsg::Draw {
                callback,
                tokens,
                number,
            } => crate::handle::draw_addresses(deps, env, callback, tokens, number),
            HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultHandleImpl),
            HandleMsg::CreateViewingKey { entropy, padding } => {
                let msg = AuthHandleMsg::CreateViewingKey { entropy, padding };
                auth_handle(deps, env, msg, AuthHandle)
            }
            HandleMsg::SetViewingKey { key, padding } => {
                let msg = AuthHandleMsg::SetViewingKey { key, padding };
                auth_handle(deps, env, msg, AuthHandle)
            }
            _ => Err(StdError::unauthorized()),
        }
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::LaunchpadInfo => crate::query::launchpad_info(deps),
        QueryMsg::UserInfo { address, key } => crate::query::user_info(deps, address, key),
        QueryMsg::Draw { tokens, number } => crate::query::draw_addresses(deps, tokens, number),
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl),
    }
}
