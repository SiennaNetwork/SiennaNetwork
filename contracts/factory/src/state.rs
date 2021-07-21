use amm_shared::{
    Exchange, ExchangeSettings, TokenPair, TokenType, Pagination,
    fadroma::scrt::{
        cosmwasm_std::{
            Api, CanonicalAddr, Extern, HumanAddr,
            Querier, StdError, StdResult, Storage,
            Binary
        },
        addr::{Humanize, Canonize},
        callback::ContractInstantiationInfo,
        storage::{save, load, ns_save, ns_load, ns_remove}
    },
    msg::factory::InitMsg,
};
use serde::{Deserialize, Serialize};
use std::usize;

const CONFIG_KEY: &[u8] = b"config";
const PRNG_KEY: &[u8] = b"prng_seed";
const IDO_PREFIX: &[u8; 1] = b"I";
const IDO_COUNT_KEY: &[u8] = b"ido_count";
const NS_IDO_WHITELIST: &[u8] = b"ido_whitelist";
const EXCHANGES_KEY: &[u8] = b"exchanges";

pub const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Config<A> {
    pub snip20_contract:   ContractInstantiationInfo,
    pub lp_token_contract: ContractInstantiationInfo,
    pub pair_contract:     ContractInstantiationInfo,
    pub ido_contract:      ContractInstantiationInfo,
    pub exchange_settings: ExchangeSettings<A>,
}

impl Config<HumanAddr> {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            snip20_contract:   msg.snip20_contract,
            lp_token_contract: msg.lp_token_contract,
            pair_contract:     msg.pair_contract,
            ido_contract:      msg.ido_contract,
            exchange_settings: msg.exchange_settings
        }
    }
}
impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            snip20_contract:   self.snip20_contract.clone(),
            lp_token_contract: self.lp_token_contract.clone(),
            pair_contract:     self.pair_contract.clone(),
            ido_contract:      self.ido_contract.clone(),
            exchange_settings: self.exchange_settings.canonize(api)?
        })
    }
}
impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            snip20_contract:   self.snip20_contract.clone(),
            lp_token_contract: self.lp_token_contract.clone(),
            pair_contract:     self.pair_contract.clone(),
            ido_contract:      self.ido_contract.clone(),
            exchange_settings: self.exchange_settings.clone().humanize(api)?
        })
    }
}

/// Returns StdResult<()> resulting from saving the config to storage
pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(
    deps:   &mut Extern<S, A, Q>,
    config: &Config<HumanAddr>
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

/// Returns StdResult<Config> resulting from retrieving the config from storage
pub(crate) fn load_config <S: Storage, A: Api, Q: Querier> (
    deps: &Extern<S, A, Q>
) -> StdResult<Config<HumanAddr>> {
    let config: Option<Config<CanonicalAddr>> = load(&deps.storage, CONFIG_KEY)?;
    config.ok_or(StdError::generic_err("Config doesn't exist in storage."))?.humanize(&deps.api)
}

pub(crate) fn save_prng_seed(
    storage: &mut impl Storage,
    prng_seed: &Binary
) -> StdResult<()> {
    save(storage, PRNG_KEY, prng_seed)
}

pub(crate) fn load_prng_seed(
    storage: &impl Storage
) -> StdResult<Binary> {
    let prng_seed: Option<Binary> = load(storage, PRNG_KEY)?;
    prng_seed.ok_or(StdError::generic_err("Prng seed doesn't exist in storage."))
}

/// Returns StdResult<bool> indicating whether a pair has been created before or not.
/// Note that TokenPair(A, B) and TokenPair(B, A) is considered to be same.
pub(crate) fn pair_exists<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pair: &TokenPair<HumanAddr>
) -> StdResult<bool> {
    let key = generate_pair_key(&pair.canonize(&deps.api)?);
    Ok(deps.storage.get(&key).is_some())
}

/// Stores information about an exchange contract. Returns an `StdError` if the exchange
/// already exists or if something else goes wrong.
pub(crate) fn store_exchange<S: Storage, A: Api, Q: Querier>(
    deps:    &mut Extern<S, A, Q>,
    exchange: Exchange<HumanAddr>
) -> StdResult<()> {
    let mut exchanges = load_exchanges(&deps.storage)?;

    store_exchange_impl(deps, exchange, &mut exchanges)?;
    save_exchanges(&mut deps.storage, &exchanges)
}

pub(crate) fn store_exchanges<S: Storage, A: Api, Q: Querier>(
    deps:    &mut Extern<S, A, Q>,
    exchanges: Vec<Exchange<HumanAddr>>
) -> StdResult<()> {
    let mut list = load_exchanges(&deps.storage)?;

    for exchange in exchanges {
        store_exchange_impl(deps, exchange, &mut list)?;
    }

    save_exchanges(&mut deps.storage, &list)
}

/// Get the address of an exchange contract which manages the given pair.
pub(crate) fn get_address_for_pair<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: &TokenPair<HumanAddr>
) -> StdResult<HumanAddr> {
    let key = generate_pair_key(&pair.canonize(&deps.api)?);

    let canonical = load(&deps.storage, &key)?.ok_or_else(||
        StdError::generic_err("Address doesn't exist in storage.")
    )?;

    deps.api.human_address(&canonical)
}

pub(crate) fn store_ido_address<S: Storage, A: Api, Q: Querier>(
    deps:    &mut Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<()> {
    let mut count = load_ido_count(&deps.storage)?;

    store_ido_address_impl(deps, address, &mut count)?;
    save_ido_count(&mut deps.storage, count)
}

pub(crate) fn store_ido_addresses<S: Storage, A: Api, Q: Querier>(
    deps:    &mut Extern<S, A, Q>,
    addresses: Vec<HumanAddr>
) -> StdResult<()> {
    let mut count = load_ido_count(&deps.storage)?;

    for addr in addresses {
        store_ido_address_impl(deps, &addr, &mut count)?;
    }

    save_ido_count(&mut deps.storage, count)
}

pub(crate) fn get_idos<S: Storage, A: Api, Q: Querier>(
    deps:       &Extern<S, A, Q>,
    pagination: Pagination
) -> StdResult<Vec<HumanAddr>> {
    let ido_count = load_ido_count(&deps.storage)?;

    if pagination.start >= ido_count {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(PAGINATION_LIMIT);
    let end = (pagination.start + limit as u64).min(ido_count);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);

    for i in pagination.start..end {
        let index = generate_ido_index(i);
        let addr: CanonicalAddr = load(&deps.storage, index.as_slice())?.ok_or_else(||
            StdError::generic_err("IDO address doesn't exist in storage.")
        )?;

        let human_addr = deps.api.human_address(&addr)?;
        result.push(human_addr);
    }

    Ok(result)
}

pub(crate) fn get_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination
) -> StdResult<Vec<Exchange<HumanAddr>>> {
    let mut exchanges = load_exchanges(&deps.storage)?;

    if pagination.start as usize >= exchanges.len() {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(PAGINATION_LIMIT);
    let end = (pagination.start + limit as u64).min(exchanges.len() as u64);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);

    let exchanges = exchanges
        .drain((pagination.start as usize)..(end as usize))
        .collect::<Vec<Exchange<CanonicalAddr>>>();
    
    for exchange in exchanges {
        result.push(exchange.humanize(&deps.api)?)
    }

    Ok(result)
}

#[inline]
pub(crate) fn load_exchanges(storage: &impl Storage) -> StdResult<Vec<Exchange<CanonicalAddr>>> {
    Ok(load(storage, EXCHANGES_KEY)?.unwrap_or(vec![]))
}

pub(crate) fn ido_whitelist_add<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: Vec<HumanAddr>
) -> StdResult<()> {
    for address in addresses {
        let address = deps.api.canonical_address(&address)?;
        ns_save(&mut deps.storage, NS_IDO_WHITELIST, address.as_slice(), &1u8)?;
    }

    Ok(())
}

pub(crate) fn ido_whitelist_remove<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<()> {
    let address = deps.api.canonical_address(address)?;

    Ok(ns_remove(&mut deps.storage, NS_IDO_WHITELIST, address.as_slice()))
}

pub(crate) fn is_ido_whitelisted<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<bool> {
    let address = deps.api.canonical_address(address)?;
    let result: Option<u8> = ns_load(&deps.storage, NS_IDO_WHITELIST, address.as_slice())?;

    Ok(result.is_some())
}

pub(crate) fn generate_pair_key(
    pair: &TokenPair<CanonicalAddr>
) -> Vec<u8> {
    let mut bytes: Vec<&[u8]> = Vec::new();

    match &pair.0 {
        TokenType::NativeToken { denom } => bytes.push(denom.as_bytes()),
        TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_slice())
    }

    match &pair.1 {
        TokenType::NativeToken { denom } => bytes.push(denom.as_bytes()),
        TokenType::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_slice())
    }

    bytes.sort();

    bytes.concat()
}

fn store_exchange_impl<S: Storage, A: Api, Q: Querier>(
    deps:    &mut Extern<S, A, Q>,
    exchange: Exchange<HumanAddr>,
    exchanges: &mut Vec<Exchange<CanonicalAddr>>
) -> StdResult<()> {
    let exchange = exchange.canonize(&deps.api)?;
    let key = generate_pair_key(&exchange.pair);

    if deps.storage.get(&key).is_some() {
        return Err(StdError::generic_err(format!("Exchange ({}) already exists", exchange.pair)));
    }

    save(&mut deps.storage, &key, &exchange.address)?;

    if exchanges.iter().any(|e| e.address == exchange.address) {
        return Err(StdError::generic_err(format!("Exchange address ({}) already exists", exchange.address)));
    }

    exchanges.push(exchange);

    Ok(())
}

fn store_ido_address_impl<S: Storage, A: Api, Q: Querier>(
    deps:    &mut Extern<S, A, Q>,
    address: &HumanAddr,
    count: &mut u64
) -> StdResult<()> {
    let address = deps.api.canonical_address(&address)?;
    let index = generate_ido_index(*count);

    *count += 1;

    save(&mut deps.storage, index.as_slice(), &address)
}

#[inline]
fn save_exchanges(
    storage: &mut impl Storage,
    exchanges: &Vec<Exchange<CanonicalAddr>>
) -> StdResult<()> {
    save(storage, EXCHANGES_KEY, exchanges)
}

#[inline]
fn load_ido_count(storage: &impl Storage) -> StdResult<u64> {
    Ok(load(storage,IDO_COUNT_KEY)?.unwrap_or(0))
}

#[inline]
fn save_ido_count(storage: &mut impl Storage, count: u64) -> StdResult<()> {
    save(storage, IDO_COUNT_KEY, &count)
}

#[inline]
fn generate_ido_index(index: u64) -> Vec<u8> {
    [ IDO_PREFIX, index.to_string().as_bytes() ].concat()
}
