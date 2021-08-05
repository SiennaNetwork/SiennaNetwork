use amm_shared::{
    Pagination, TokenPair,
    admin::admin::{
        DefaultHandleImpl as AdminHandle, DefaultQueryImpl as AdminQuery,
        admin_handle, admin_query, assert_admin, load_admin, save_admin
    },
    exchange::Exchange, 
    fadroma::scrt::{
        callback::{Callback, ContractInstance},
        cosmwasm_std::{
            log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
        },
        migrate as fadroma_scrt_migrate,
        storage::{load, remove, save}
    },
    msg::{
        exchange::InitMsg as ExchangeInitMsg,
        factory::{HandleMsg, InitMsg, QueryMsg, QueryResponse},
        ido::{InitMsg as IdoInitMsg, TokenSaleConfig},
    }
};

use amm_shared::admin::require_admin;

use crate::state::{
    get_address_for_pair, get_exchanges, get_idos, ido_whitelist_add, ido_whitelist_remove,
    is_ido_whitelisted, load_config, load_prng_seed, pair_exists, save_config, save_prng_seed,
    store_exchanges, store_ido_addresses, Config,
};
use fadroma_scrt_migrate::{get_status, with_status};

pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    let admin = msg.admin.clone().unwrap_or(env.message.sender);
    save_admin(deps, &admin)?;

    save_prng_seed(&mut deps.storage, &msg.prng_seed)?;
    save_config(deps, &Config::from_init_msg(msg))?;

    Ok(InitResponse::default())
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
            HandleMsg::SetConfig { .. } => set_config(deps, env, msg),
            HandleMsg::IdoWhitelist { addresses } => ido_whitelist(deps, env, addresses),
            HandleMsg::CreateExchange { pair, entropy } => create_exchange(deps, env, pair, entropy),
            HandleMsg::CreateIdo { info, entropy } => create_ido(deps, env, info, entropy),
            HandleMsg::RegisterIdo { signature } => register_ido(deps, env, signature),
            HandleMsg::RegisterExchange { pair, signature } =>
                register_exchange(deps, env, pair, signature),
            HandleMsg::AddExchanges { exchanges } => add_exchanges(deps, env, exchanges),
            HandleMsg::AddIdos { idos } => add_idos(deps, env, idos),
            HandleMsg::Admin(msg) => admin_handle(deps, env, msg, AdminHandle),
        }
    )
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::Status => to_binary(&get_status(deps)?),
        QueryMsg::GetConfig {} => get_config(deps),
        QueryMsg::GetExchangeAddress { pair } => query_exchange_address(deps, pair),
        QueryMsg::ListExchanges { pagination } => list_exchanges(deps, pagination),
        QueryMsg::ListIdos { pagination } => list_idos(deps, pagination),
        QueryMsg::GetExchangeSettings => query_exchange_settings(deps),

        QueryMsg::Admin(msg) => admin_query(deps, msg, AdminQuery),
    }
}

#[require_admin]
pub fn set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    if let HandleMsg::SetConfig {
        snip20_contract,
        lp_token_contract,
        pair_contract,
        ido_contract,
        exchange_settings,
    } = msg
    {
        let mut config = load_config(&deps)?;

        if let Some(new_value) = snip20_contract {
            config.snip20_contract = new_value;
        }
        if let Some(new_value) = lp_token_contract {
            config.lp_token_contract = new_value;
        }
        if let Some(new_value) = pair_contract {
            config.pair_contract = new_value;
        }
        if let Some(new_value) = ido_contract {
            config.ido_contract = new_value;
        }
        if let Some(new_value) = exchange_settings {
            config.exchange_settings = new_value;
        }

        save_config(deps, &config)?;

        Ok(HandleResponse {
            messages: vec![],
            log: vec![log("action", "set_config")],
            data: None,
        })
    } else {
        unreachable!()
    }
}

pub fn get_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Binary> {
    let Config {
        snip20_contract,
        lp_token_contract,
        pair_contract,
        ido_contract,
        exchange_settings,
        ..
    } = load_config(deps)?;

    to_binary(&QueryResponse::Config {
        snip20_contract,
        lp_token_contract,
        pair_contract,
        ido_contract,
        exchange_settings,
    })
}

pub fn create_exchange<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair<HumanAddr>,
    entropy: Binary
) -> StdResult<HandleResponse> {
    if pair.0 == pair.1 {
        return Err(StdError::generic_err(
            "Cannot create an exchange with the same token.",
        ));
    }

    if pair_exists(deps, &pair)? {
        return Err(StdError::generic_err("Pair already exists"));
    }

    let config = load_config(deps)?;

    // We take advantage of the serialized execution model to create a signature
    // and remove it at the end of the transaction. This signature is passed to
    // the created pair which it then returns to HandleMsg::RegisterExchange so that
    // it can be compared to the one we stored. This way, we ensure that exchanges
    // can only be created through this method.
    let signature = create_signature(&env)?;
    save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;

    // Actually creating the exchange happens when the instantiated contract calls
    // us back via the HandleMsg::RegisterExchange so that we can get its address.

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.pair_contract.id,
            callback_code_hash: config.pair_contract.code_hash,
            send: vec![],
            label: format!(
                "{}-{}-pair-{}-{}",
                pair.0, pair.1, env.contract.address, config.pair_contract.id
            ),
            msg: to_binary(&ExchangeInitMsg {
                pair: pair.clone(),
                lp_token_contract: config.lp_token_contract.clone(),
                factory_info: ContractInstance {
                    code_hash: env.contract_code_hash.clone(),
                    address: env.contract.address.clone(),
                },
                callback: Callback {
                    contract: ContractInstance {
                        address: env.contract.address,
                        code_hash: env.contract_code_hash,
                    },
                    msg: to_binary(&HandleMsg::RegisterExchange {
                        pair: pair.clone(),
                        signature,
                    })?,
                },
                entropy,
                prng_seed: load_prng_seed(&deps.storage)?,
            })?,
        })],
        log: vec![log("action", "create_exchange"), log("pair", pair)],
        data: None,
    })
}

fn register_exchange<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair<HumanAddr>,
    signature: Binary,
) -> StdResult<HandleResponse> {
    ensure_correct_signature(&mut deps.storage, signature)?;

    let exchange = Exchange {
        pair,
        address: env.message.sender.clone(),
    };

    store_exchanges(deps, vec![exchange])?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_exchange"),
            log("address", env.message.sender),
        ],
        data: None,
    })
}

fn query_exchange_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: TokenPair<HumanAddr>,
) -> StdResult<Binary> {
    let address = get_address_for_pair(deps, &pair)?;

    to_binary(&QueryResponse::GetExchangeAddress { address })
}

fn create_ido<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    info: TokenSaleConfig,
    entropy: Binary
) -> StdResult<HandleResponse> {
    let whitelisted = is_ido_whitelisted(deps, &env.message.sender)?;
    let is_admin = load_admin(deps)? == env.message.sender;

    if !whitelisted && !is_admin {
        return Err(StdError::unauthorized());
    }

    // Remove to allow only 1 IDO created
    if whitelisted {
        ido_whitelist_remove(deps, &env.message.sender)?;
    }

    let signature = create_signature(&env)?;
    save(&mut deps.storage, EPHEMERAL_STORAGE_KEY, &signature)?;
    // Again, creating the IDO happens when the instantiated contract calls
    // us back via the HandleMsg::RegisterIdo so that we can get its address.
    let config = load_config(deps)?;

    Ok(HandleResponse {
        messages: vec![CosmosMsg::Wasm(WasmMsg::Instantiate {
            code_id: config.ido_contract.id,
            callback_code_hash: config.ido_contract.code_hash,
            send: vec![],
            label: format!(
                "SIENNA IDO for token {}, created at {}",
                info.sold_token.address,
                env.block.time // Make sure the label is unique
            ),
            msg: to_binary(&IdoInitMsg {
                admin: env.message.sender,
                info,
                prng_seed: load_prng_seed(&deps.storage)?,
                entropy,
                callback: Callback {
                    contract: ContractInstance {
                        address: env.contract.address,
                        code_hash: env.contract_code_hash,
                    },
                    msg: to_binary(&HandleMsg::RegisterIdo { signature })?,
                },
            })?,
        })],
        log: vec![log("action", "create_ido")],
        data: None,
    })
}

fn register_ido<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    signature: Binary,
) -> StdResult<HandleResponse> {
    ensure_correct_signature(&mut deps.storage, signature)?;

    store_ido_addresses(deps, vec![env.message.sender.clone()])?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "register_ido"),
            log("address", env.message.sender),
        ],
        data: None,
    })
}

#[require_admin]
fn ido_whitelist<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    addresses: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    ido_whitelist_add(deps, addresses)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "ido_whitelist")],
        data: None,
    })
}

#[require_admin]
fn add_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    exchanges: Vec<Exchange<HumanAddr>>,
) -> StdResult<HandleResponse> {
    store_exchanges(deps, exchanges)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "add_exchanges")],
        data: None,
    })
}

#[require_admin]
fn add_idos<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    idos: Vec<HumanAddr>,
) -> StdResult<HandleResponse> {
    store_ido_addresses(deps, idos)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![log("action", "add_idos")],
        data: None,
    })
}

fn list_idos<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Binary> {
    let idos = get_idos(deps, pagination)?;

    to_binary(&QueryResponse::ListIdos { idos })
}

fn list_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Binary> {
    let exchanges = get_exchanges(deps, pagination)?;

    to_binary(&QueryResponse::ListExchanges { exchanges })
}

fn query_exchange_settings<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Binary> {
    let config = load_config(deps)?;

    Ok(to_binary(&QueryResponse::GetExchangeSettings {
        settings: config.exchange_settings,
    })?)
}

pub(crate) fn create_signature(env: &Env) -> StdResult<Binary> {
    to_binary(
        &[
            env.message.sender.0.as_bytes(),
            &env.block.height.to_be_bytes(),
            &env.block.time.to_be_bytes(),
        ]
        .concat(),
    )
}

fn ensure_correct_signature(storage: &mut impl Storage, signature: Binary) -> StdResult<()> {
    let stored_signature: Binary = load(storage, EPHEMERAL_STORAGE_KEY)?.unwrap_or_default();

    if stored_signature != signature {
        return Err(StdError::unauthorized());
    }

    remove(storage, EPHEMERAL_STORAGE_KEY);

    Ok(())
}
