use amm_shared::{
    fadroma::{
        auth::admin::{
            assert_admin, handle as admin_handle, query as admin_query, Admin,
            DefaultImpl as AdminImpl,
        },
        auth_proc::require_admin,
        platform::{
            log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HumanAddr,
            InitResponse, Querier, StdError, StdResult, Storage, WasmMsg,
            Humanize,
            Callback,
            ContractLink,
        },
        killswitch,
        killswitch::{get_status, ContractStatusLevel},
        storage::{load, remove, save},
    },
    msg::{
        exchange::{HandleMsg as ExchangeHandleMsg, InitMsg as ExchangeInitMsg},
        factory::{HandleMsg, InitMsg, QueryMsg, QueryResponse}
    },
    Pagination, TokenPair, Exchange
};

use crate::state::{
    exchanges_store, get_address_for_pair, get_exchanges, load_config,
    load_migration_address, load_prng_seed, pair_exists,
    remove_migration_address, save_config, save_migration_address,
    save_prng_seed, store_exchanges, Config,
};

pub const EPHEMERAL_STORAGE_KEY: &[u8] = b"ephemeral_storage";
pub const TRANSFER_LIMIT: usize = 30;

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: InitMsg,
) -> StdResult<InitResponse> {
    save_prng_seed(&mut deps.storage, &msg.prng_seed)?;
    save_config(
        deps,
        &Config {
            lp_token_contract: msg.lp_token_contract,
            pair_contract: msg.pair_contract,
            exchange_settings: msg.exchange_settings,
        },
    )?;

    AdminImpl.new(msg.admin, deps, env)
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    match msg {
        HandleMsg::SetStatus {
            level,
            reason,
            new_address,
        } => {
            killswitch::set_status(deps, env, level, reason, new_address)?;
            return Ok(HandleResponse::default());
        }
        HandleMsg::TransferExchanges {
            new_instance,
            skip,
        } => {
            return transfer_exchanges(deps, env, new_instance, skip);
        }
        _ => {}
    }

    killswitch::is_operational(&deps)?;

    match msg {
        HandleMsg::SetStatus { .. } => unreachable!(),
        HandleMsg::TransferExchanges { .. } => unreachable!(),
        HandleMsg::SetConfig { .. } => set_config(deps, env, msg),
        HandleMsg::CreateExchange { pair, entropy } => create_exchange(deps, env, pair, entropy),
        HandleMsg::RegisterExchange { pair, signature } => {
            register_exchange(deps, env, pair, signature)
        }
        HandleMsg::ReceiveExchanges {
            finalize,
            exchanges,
        } => receive_exchanges(deps, env, finalize, exchanges),
        HandleMsg::SetMigrationAddress { address } => set_migration_address(deps, env, address),
        HandleMsg::Admin(msg) => admin_handle(deps, env, msg, AdminImpl),
    }
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
        QueryMsg::GetExchangeSettings => query_exchange_settings(deps),

        QueryMsg::Admin(msg) => to_binary(&admin_query(deps, msg, AdminImpl)?),
    }
}

#[require_admin]
pub fn set_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> StdResult<HandleResponse> {
    if let HandleMsg::SetConfig {
        lp_token_contract,
        pair_contract,
        exchange_settings,
    } = msg
    {
        let mut config = load_config(&deps)?;

        if let Some(new_value) = lp_token_contract {
            config.lp_token_contract = new_value;
        }

        if let Some(new_value) = pair_contract {
            config.pair_contract = new_value;
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
        lp_token_contract,
        pair_contract,
        exchange_settings
    } = load_config(deps)?;

    to_binary(&QueryResponse::Config {
        lp_token_contract,
        pair_contract,
        exchange_settings,
    })
}

pub fn create_exchange<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    pair: TokenPair<HumanAddr>,
    entropy: Binary,
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
                factory_info: ContractLink {
                    code_hash: env.contract_code_hash.clone(),
                    address: env.contract.address.clone(),
                },
                callback: Callback {
                    contract: ContractLink {
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

    let config = load_config(deps)?;

    let exchange = Exchange {
        pair,
        contract: ContractLink {
            address: env.message.sender.clone(),
            code_hash: config.pair_contract.code_hash,
        },
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

#[require_admin]
fn transfer_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    new_instance: ContractLink<HumanAddr>,
    skip: Option<Vec<HumanAddr>>,
) -> StdResult<HandleResponse> {
    let status = killswitch::get_status(&deps)?;

    if status.level != ContractStatusLevel::Migrating {
        killswitch::set_status(
            deps,
            env,
            ContractStatusLevel::Migrating,
            "Migrating to new factory.".into(),
            Some(new_instance.address.clone()),
        )?;
    }

    let skip = skip.unwrap_or(vec![]);
    let mut exchanges_store = exchanges_store();

    let len = exchanges_store
        .len(&deps.storage)?
        .min(TRANSFER_LIMIT as u64);
    let mut messages = Vec::with_capacity(len as usize - skip.len() + 1);
    let mut exchanges = Vec::with_capacity(len as usize - skip.len());

    for e in exchanges_store
        .iter(&deps.storage)?
        .rev()
        .take(TRANSFER_LIMIT)
    {
        let e = e?.humanize(&deps.api)?;

        if skip.contains(&e.contract.address) {
            continue;
        }

        messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: e.contract.address.clone(),
            callback_code_hash: e.contract.code_hash.clone(),
            send: vec![],
            msg: to_binary(&ExchangeHandleMsg::ChangeFactory {
                contract: new_instance.clone(),
            })?,
        }));

        exchanges.push(e);
    }

    for _ in 0..len {
        exchanges_store.pop(&mut deps.storage)?;
    }

    // len has changed after we removed the items above
    let len = exchanges_store.len(&deps.storage)?;

    messages.push(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: new_instance.address,
        callback_code_hash: new_instance.code_hash,
        send: vec![],
        msg: to_binary(&HandleMsg::ReceiveExchanges {
            finalize: len == 0,
            exchanges,
        })?,
    }));

    Ok(HandleResponse {
        messages,
        log: vec![log("action", "transfer_exchanges"), log("remaining", len)],
        data: None,
    })
}

fn receive_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    finalize: bool,
    exchanges: Vec<Exchange<HumanAddr>>,
) -> StdResult<HandleResponse> {
    let address = load_migration_address(&deps.storage)?;

    if address != env.message.sender {
        return Err(StdError::unauthorized());
    }

    if finalize {
        remove_migration_address(&mut deps.storage);
    }

    let len = exchanges.len();
    store_exchanges(deps, exchanges)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "receive_exchanges"),
            log("received", len),
            log("finalize", finalize)
        ],
        data: None,
    })
}

#[require_admin]
fn set_migration_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    address: HumanAddr,
) -> StdResult<HandleResponse> {
    save_migration_address(&mut deps.storage, &address)?;

    Ok(HandleResponse {
        messages: vec![],
        log: vec![
            log("action", "set_migration_address"),
            log("migration_address", address)
        ],
        data: None,
    })
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
    let stored_signature: Binary =
        load(storage, EPHEMERAL_STORAGE_KEY)?.ok_or_else(|| StdError::unauthorized())?;
    if stored_signature != signature {
        return Err(StdError::unauthorized());
    }

    remove(storage, EPHEMERAL_STORAGE_KEY);

    Ok(())
}
