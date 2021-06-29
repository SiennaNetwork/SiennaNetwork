use cosmwasm_std::{
    Api, Binary, Env, Extern, HandleResponse, HumanAddr, InitResponse,
    Querier, StdResult, Storage, StdError, log, to_binary,
    QueryRequest, WasmQuery, CosmosMsg, WasmMsg
};
use composable_admin::require_admin;
use composable_admin::admin::{
    save_admin, admin_handle, admin_query, DefaultHandleImpl as DefaultAdminHandle,
    DefaultQueryImpl, assert_admin
};
use fadroma_scrt_callback::{Callback, ContractInstance, ContractInstantiationInfo};
use fadroma_scrt_storage::{load, save, remove};

use sienna_rewards::msg::{InitMsg as PoolInitMsg, QueryMsg as PoolQueryMsg, QueryMsgResponse as PoolQueryResponse};

use crate::state::*;
use crate::msg::{HandleMsg, InitMsg, PoolContractInfo, PoolInitInfo, QueryMsg, QueryMsgResponse};

const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let admin = msg.admin.unwrap_or(env.message.sender);
    save_admin(deps, &admin)?;

    save_config(&mut deps.storage, &Config {
        reward_contract: msg.rewards_contract
    })?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::CreatePool { info } => create_pool(deps, env, info),
        HandleMsg::RegisterPool { signature } => register_pool(deps, env, signature),
        HandleMsg::AddPools { instances } => add_pools(deps, env, instances),
        HandleMsg::RemovePools { addresses } => remove_pools(deps, env, addresses),
        HandleMsg::ChangeRewardsContract { contract } => change_rewards_contract(deps, env, contract),
        HandleMsg::Admin(admin_msg) => admin_handle(deps, env, admin_msg, DefaultAdminHandle)
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Pools => query_pools(deps),
        QueryMsg::Admin(admin_msg) => admin_query(deps, admin_msg, DefaultQueryImpl)
    }
}

#[require_admin]
fn create_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: PoolInitInfo
) -> StdResult<HandleResponse> {
    let config = load_config(&deps.storage)?;

    // We take advantage of the serialized execution model to create a signature
    // and remove it at the end of the transaction. This signature is passed to
    // the created pair which it then returns to HandleMsg::RegisterPool so that
    // it can be compared to the one we stored. This way, we ensure that exchanges 
    // can only be created through this method.
    let signature = create_signature(&env)?;
    save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

    Ok(HandleResponse{
        messages: vec![
            CosmosMsg::Wasm(
                WasmMsg::Instantiate {
                    code_id: config.reward_contract.id,
                    callback_code_hash: config.reward_contract.code_hash,
                    send: vec![],
                    label: format!(
                        "SIENNA Reward Pool: {}, id: {}, factory: {})",
                        info.pool.lp_token.address,
                        config.reward_contract.id,
                        env.contract.address,
                    ),
                    msg: to_binary(
                        &PoolInitMsg {
                            admin: Some(info.admin.unwrap_or(env.message.sender)),
                            reward_token: info.reward_token,
                            pool: info.pool,
                            claim_interval: info.claim_interval,
                            prng_seed: info.prng_seed,
                            entropy: info.entropy,
                            callback: Callback {
                                contract: ContractInstance {
                                    address:   env.contract.address,
                                    code_hash: env.contract_code_hash,
                                },
                                msg: to_binary(&HandleMsg::RegisterPool {
                                    signature
                                })?,
                            }
                        }
                    )?
                }
            )
        ],
        log: vec![
            log("action", "create_pool"),
        ],
        data: None
    })
}

fn register_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    signature: Binary
) -> StdResult<HandleResponse> {
    ensure_correct_signature(&mut deps.storage, signature)?;

    let config = load_config(&deps.storage)?;

    store_pools(deps, vec![ 
        ContractInstance {
            address: env.message.sender.clone(),
            code_hash: config.reward_contract.code_hash
        }    
    ])?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_pool"),
            log("address", env.message.sender)
        ],
        data: None
    })
}

#[require_admin]
fn add_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    instances: Vec<ContractInstance<HumanAddr>>
) -> StdResult<HandleResponse> {
    store_pools(deps, instances)?;

    Ok(HandleResponse::default())
}

#[require_admin]
fn remove_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<HumanAddr>
) -> StdResult<HandleResponse> {
    delete_pools(deps, addresses)?;

    Ok(HandleResponse::default())
}

#[require_admin]
fn change_rewards_contract<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    contract: ContractInstantiationInfo
) -> StdResult<HandleResponse> {
    let mut config = load_config(&deps.storage)?;
    config.reward_contract = contract;

    save_config(&mut deps.storage, &config)?;

    Ok(HandleResponse::default())
}

fn query_pools<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Binary> {
    let instances = load_pools(deps)?;
    let mut result = Vec::with_capacity(instances.len());

    for instance in instances {
        let resp: PoolQueryResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            callback_code_hash: instance.code_hash,
            contract_addr: instance.address.clone(),
            msg: to_binary(&PoolQueryMsg::Pool)?
        }))?;

        match resp {
            PoolQueryResponse::Pool(pool) => {
                result.push(PoolContractInfo {
                    pool,
                    address: instance.address
                })
            },
            _ => return Err(StdError::generic_err("Pool contract returned an unexpected response."))
        }
    }

    to_binary(&QueryMsgResponse::Pools(result))
}

fn create_signature(env: &Env) -> StdResult<Binary> {
    to_binary(&[
        env.message.sender.0.as_bytes(),
        &env.block.height.to_be_bytes(),
        &env.block.time.to_be_bytes()
    ].concat())
}

fn ensure_correct_signature(storage: &mut impl Storage, signature: Binary) -> StdResult<()> {
    let stored_signature: Binary =
        load(storage, EPHEMERAL_STORAGE_KEY)?.unwrap_or_default();

    if stored_signature != signature {
        return Err(StdError::unauthorized());
    }

    remove(storage, EPHEMERAL_STORAGE_KEY);

    Ok(())
}
