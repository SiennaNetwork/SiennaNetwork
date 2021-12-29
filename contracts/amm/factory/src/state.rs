use std::usize;

use amm_shared::{
    fadroma::{
        platform::{
            Api, Binary, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage,
            Canonize, Humanize,
            ContractInstantiationInfo, ContractLink,
        },
        storage::{load, ns_load, ns_remove, ns_save, remove, save, IterableStorage},
    },
    Pagination, TokenPair, TokenType, Exchange, ExchangeSettings
};
use serde::{Deserialize, Serialize};

const CONFIG_KEY: &[u8] = b"config";
const PRNG_KEY: &[u8] = b"prng_seed";
const LAUNCHPAD_KEY: &[u8] = b"launchpad_instance";
const MIGRATION_KEY: &[u8] = b"migration";
const ROUTER_KEY: &[u8] = b"router";

const NS_IDO_WHITELIST: &[u8] = b"ido_whitelist";
const NS_IDOS: &[u8] = b"idos";
const NS_EXCHANGES: &[u8] = b"exchanges";

pub const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Config<A> {
    pub snip20_contract: ContractInstantiationInfo,
    pub lp_token_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub launchpad_contract: ContractInstantiationInfo,
    pub ido_contract: ContractInstantiationInfo,
    pub router_contract: ContractInstantiationInfo,
    pub exchange_settings: ExchangeSettings<A>,
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            snip20_contract: self.snip20_contract.clone(),
            lp_token_contract: self.lp_token_contract.clone(),
            pair_contract: self.pair_contract.clone(),
            launchpad_contract: self.launchpad_contract.clone(),
            ido_contract: self.ido_contract.clone(),
            router_contract: self.router_contract.clone(),
            exchange_settings: self.exchange_settings.canonize(api)?,
        })
    }
}
impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            snip20_contract: self.snip20_contract.clone(),
            lp_token_contract: self.lp_token_contract.clone(),
            pair_contract: self.pair_contract.clone(),
            launchpad_contract: self.launchpad_contract.clone(),
            ido_contract: self.ido_contract.clone(),
            router_contract: self.router_contract.clone(),
            exchange_settings: self.exchange_settings.clone().humanize(api)?,
        })
    }
}

/// Returns StdResult<()> resulting from saving the config to storage
pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config<HumanAddr>,
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

/// Returns StdResult<Config> resulting from retrieving the config from storage
pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Config<HumanAddr>> {
    let config: Option<Config<CanonicalAddr>> = load(&deps.storage, CONFIG_KEY)?;

    config
        .ok_or_else(|| StdError::generic_err("Config doesn't exist in storage."))?
        .humanize(&deps.api)
}

pub(crate) fn save_prng_seed(storage: &mut impl Storage, prng_seed: &Binary) -> StdResult<()> {
    save(storage, PRNG_KEY, prng_seed)
}

pub(crate) fn load_prng_seed(storage: &impl Storage) -> StdResult<Binary> {
    let prng_seed: Option<Binary> = load(storage, PRNG_KEY)?;

    prng_seed.ok_or_else(|| StdError::generic_err("Prng seed doesn't exist in storage."))
}

pub(crate) fn save_migration_address(storage: &mut impl Storage, pass: &HumanAddr) -> StdResult<()> {
    save(storage, MIGRATION_KEY, pass)
}

pub(crate) fn load_migration_address(storage: &impl Storage) -> StdResult<HumanAddr> {
    let pass: Option<HumanAddr> = load(storage, MIGRATION_KEY)?;

    pass.ok_or_else(|| StdError::unauthorized())
}

pub(crate) fn remove_migration_address(storage: &mut impl Storage) {
    remove(storage, MIGRATION_KEY)
}

pub(crate) fn save_launchpad_instance(
    storage: &mut impl Storage,
    instance: &ContractLink<HumanAddr>,
) -> StdResult<()> {
    save(storage, LAUNCHPAD_KEY, instance)
}

pub(crate) fn load_launchpad_instance(
    storage: &impl Storage,
) -> StdResult<Option<ContractLink<HumanAddr>>> {
    load(storage, LAUNCHPAD_KEY)
}

pub(crate) fn save_router_instance(
    storage: &mut impl Storage,
    instance: &ContractLink<HumanAddr>,
) -> StdResult<()> {
    save(storage, ROUTER_KEY, instance)
}

pub(crate) fn load_router_instance(
    storage: &impl Storage,
) -> StdResult<Option<ContractLink<HumanAddr>>> {
    load(storage, ROUTER_KEY)
}

#[inline]
pub(crate) fn exchanges_store() -> IterableStorage<Exchange<CanonicalAddr>> {
    IterableStorage::new(NS_EXCHANGES)
}

#[inline]
pub(crate) fn idos_store() -> IterableStorage<CanonicalAddr> {
    IterableStorage::new(NS_IDOS)
}

/// Returns StdResult<bool> indicating whether a pair has been created before or not.
/// Note that TokenPair(A, B) and TokenPair(B, A) is considered to be same.
pub(crate) fn pair_exists<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pair: &TokenPair<HumanAddr>,
) -> StdResult<bool> {
    let key = generate_pair_key(pair.canonize(&deps.api)?);
    let result: Option<CanonicalAddr> = ns_load(&deps.storage, NS_EXCHANGES, &key)?;

    Ok(result.is_some())
}

pub(crate) fn store_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    exchanges: Vec<Exchange<HumanAddr>>,
) -> StdResult<()> {
    let mut exchanges_store = exchanges_store();

    for exchange in exchanges {
        let exchange = exchange.canonize(&deps.api)?;
        let key = generate_pair_key(exchange.pair.clone());

        let result: Option<CanonicalAddr> = ns_load(&deps.storage, NS_EXCHANGES, &key)?;
        if result.is_some() {
            return Err(StdError::generic_err(format!(
                "Exchange ({}) already exists",
                exchange.pair
            )));
        }

        ns_save(
            &mut deps.storage,
            NS_EXCHANGES,
            &key,
            &exchange.contract.address,
        )?;
        exchanges_store.push(&mut deps.storage, &exchange)?;
    }

    Ok(())
}

/// Get the address of an exchange contract which manages the given pair.
pub(crate) fn get_address_for_pair<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: &TokenPair<HumanAddr>,
) -> StdResult<HumanAddr> {
    let key = generate_pair_key(pair.canonize(&deps.api)?);

    let canonical = ns_load(&deps.storage, NS_EXCHANGES, &key)?
        .ok_or_else(|| StdError::generic_err("Address doesn't exist in storage."))?;

    deps.api.human_address(&canonical)
}

pub(crate) fn store_ido_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: Vec<HumanAddr>,
) -> StdResult<()> {
    let mut idos_store = idos_store();

    for address in addresses {
        let address = deps.api.canonical_address(&address)?;
        idos_store.push(&mut deps.storage, &address)?;
    }

    Ok(())
}

pub(crate) fn get_idos<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Vec<HumanAddr>> {
    let limit = pagination.limit.min(PAGINATION_LIMIT);

    let idos_store = idos_store();
    let iterator = idos_store
        .iter(&deps.storage)?
        .skip(pagination.start as usize)
        .take(limit as usize);

    let mut result = Vec::with_capacity(iterator.len());

    for addr in iterator {
        let addr = addr?;
        result.push(addr.humanize(&deps.api)?);
    }

    Ok(result)
}

pub(crate) fn get_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination,
) -> StdResult<Vec<Exchange<HumanAddr>>> {
    let limit = pagination.limit.min(PAGINATION_LIMIT);

    let exchanges_store = exchanges_store();
    let iterator = exchanges_store
        .iter(&deps.storage)?
        .skip(pagination.start as usize)
        .take(limit as usize);

    let mut result = Vec::with_capacity(iterator.len());

    for exchange in iterator {
        let exchange = exchange?;
        result.push(exchange.humanize(&deps.api)?);
    }

    Ok(result)
}

pub(crate) fn ido_whitelist_add<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: Vec<HumanAddr>,
) -> StdResult<()> {
    for address in addresses {
        let address = deps.api.canonical_address(&address)?;
        ns_save(
            &mut deps.storage,
            NS_IDO_WHITELIST,
            address.as_slice(),
            &1u8,
        )?;
    }

    Ok(())
}

pub(crate) fn ido_whitelist_remove<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    address: &HumanAddr,
) -> StdResult<()> {
    let address = deps.api.canonical_address(address)?;

    Ok(ns_remove(
        &mut deps.storage,
        NS_IDO_WHITELIST,
        address.as_slice(),
    ))
}

pub(crate) fn is_ido_whitelisted<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr,
) -> StdResult<bool> {
    let address = deps.api.canonical_address(address)?;
    let result: Option<u8> = ns_load(&deps.storage, NS_IDO_WHITELIST, address.as_slice())?;

    Ok(result.is_some())
}

pub(crate) fn generate_pair_key(pair: TokenPair<CanonicalAddr>) -> Vec<u8> {
    let mut bytes = vec![
        token_type_to_slice(pair.0),
        token_type_to_slice(pair.1)
    ];
    bytes.sort();

    let mut result = Vec::with_capacity(bytes[0].len() + bytes[1].len());

    for slice in bytes.into_iter() {
        result.extend(slice)
    }

    result
}

#[inline]
fn token_type_to_slice(token: TokenType<CanonicalAddr>) -> Vec<u8> {
    match token {
        TokenType::NativeToken { mut denom } => {
            denom.make_ascii_lowercase();

            Vec::from(denom)
        },
        TokenType::CustomToken { contract_addr, .. } => contract_addr.0.0,
    }
}
