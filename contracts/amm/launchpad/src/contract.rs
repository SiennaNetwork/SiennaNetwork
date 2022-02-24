use amm_shared::fadroma as fadroma;

use fadroma::{
    platform::{
        to_binary, Api, CosmosMsg, Env, Extern, HandleResponse, InitResponse, Querier,
        QueryResult, StdResult, Storage, WasmMsg,
        secret_toolkit::snip20,
        BLOCK_SIZE,
        ContractLink
    },
    ViewingKey,
    auth::{
        admin::{
            handle as admin_handle,
            query as admin_query,
            DefaultImpl as AdminImpl,
            save_admin
        },
        vk_auth::{
            HandleMsg as AuthHandleMsg,
            handle as auth_handle,
            DefaultImpl as AuthImpl
        }
    },
    killswitch::get_status,
    killswitch::with_status,
    storage::save,
};
use amm_shared::TokenType;
use amm_shared::msg::launchpad::{HandleMsg, InitMsg, QueryMsg};

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
                    ContractLink {
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
                    token_decimals,
                });
            }
            _ => config.tokens.push(TokenConfig {
                token_type: token.token_type,
                segment: token.segment,
                bounding_period: token.bounding_period,
                token_decimals: 6,
            }),
        }
    }

    save_viewing_key(&mut deps.storage, &viewing_key)?;
    save_admin(deps, &msg.admin)?;
    save(&mut deps.storage, b"config", &config)?;

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
        log: vec![],
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
            HandleMsg::AdminAddToken { config } =>
                crate::handle::admin_add_token(deps, env, config),
            HandleMsg::AdminRemoveToken { index } =>
                crate::handle::admin_remove_token(deps, env, index),
            HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, AdminImpl),
            HandleMsg::CreateViewingKey { entropy, padding } => {
                let msg = AuthHandleMsg::CreateViewingKey { entropy, padding };
                auth_handle(deps, env, msg, AuthImpl)
            }
            HandleMsg::SetViewingKey { key, padding } => {
                let msg = AuthHandleMsg::SetViewingKey { key, padding };
                auth_handle(deps, env, msg, AuthImpl)
            }
        }
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>, msg: QueryMsg) -> QueryResult {
    match msg {
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::LaunchpadInfo => crate::query::launchpad_info(deps),
        QueryMsg::UserInfo { address, key } => crate::query::user_info(deps, address, key),
        QueryMsg::Draw {
            tokens,
            number,
            timestamp,
        } => crate::query::draw_addresses(deps, tokens, number, timestamp),
        QueryMsg::Admin(admin_msg) => to_binary(&admin_query(deps, admin_msg, AdminImpl)?),
    }
}
