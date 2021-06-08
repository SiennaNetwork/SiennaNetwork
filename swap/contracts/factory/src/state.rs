use cosmwasm_std::{Api, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use fadroma_scrt_addr::{Humanize, Canonize};
use fadroma_scrt_callback::ContractInstantiationInfo;
use fadroma_scrt_storage::{save, load};
use serde::{Deserialize, Serialize};
use amm_shared::{
    Exchange, ExchangeSettings, TokenPair, TokenType, Pagination,
    msg::factory::InitMsg
};
use std::usize;

const CONFIG_KEY: &[u8] = b"config";
const IDO_PREFIX: &[u8; 1] = b"I";
const EXCHANGES_KEY: &[u8] = b"exchanges";

pub const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Config<A> {
    pub snip20_contract:   ContractInstantiationInfo,
    pub lp_token_contract: ContractInstantiationInfo,
    pub pair_contract:     ContractInstantiationInfo,
    pub ido_contract:      ContractInstantiationInfo,
    pub exchange_settings: ExchangeSettings<A>,
    pub pair_count:        u64,
    pub ido_count:         u64
}
impl Config<HumanAddr> {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            snip20_contract:   msg.snip20_contract,
            lp_token_contract: msg.lp_token_contract,
            pair_contract:     msg.pair_contract,
            ido_contract:      msg.ido_contract,
            exchange_settings: msg.exchange_settings,
            pair_count:        0,
            ido_count:         0
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
            exchange_settings: self.exchange_settings.canonize(api)?,
            pair_count:        self.pair_count,
            ido_count:         self.ido_count
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
            exchange_settings: self.exchange_settings.clone().humanize(api)?,
            pair_count:        self.pair_count,
            ido_count:         self.ido_count
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
    pair:    &TokenPair<HumanAddr>,
    address: &HumanAddr
) -> StdResult<()> {
    let pair = pair.canonize(&deps.api)?;
    let key = generate_pair_key(&pair);
    if deps.storage.get(&key).is_some() {
        return Err(StdError::generic_err("Exchange already exists"));
    }

    let address = address.canonize(&deps.api)?;
    save(&mut deps.storage, &key, &address)?;

    let mut exchanges = load_exchanges(&deps.storage)?;
    if exchanges.iter().any(|e| e.address == address) {
        return Err(StdError::generic_err("Exchange address already exists"));
    }

    exchanges.push(Exchange { pair, address });
    save_exchanges(&mut deps.storage, &exchanges)
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
    address: &HumanAddr,
    config:  &mut Config<HumanAddr>
) -> StdResult<()> {
    let address = deps.api.canonical_address(&address)?;
    let index = generate_ido_index(&config.ido_count);

    save(&mut deps.storage, index.as_slice(), &address)?;

    config.ido_count += 1;
    save_config(deps, &config)
}

pub(crate) fn get_idos<S: Storage, A: Api, Q: Querier>(
    deps:       &Extern<S, A, Q>,
    config:     &Config<HumanAddr>,
    pagination: Pagination
) -> StdResult<Vec<HumanAddr>> {
    if pagination.start >= config.ido_count {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(PAGINATION_LIMIT);
    let end = (pagination.start + limit as u64).min(config.ido_count);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);

    for i in pagination.start..end {
        let index = generate_ido_index(&i);
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

pub(crate) fn load_exchanges(storage: &impl Storage) -> StdResult<Vec<Exchange<CanonicalAddr>>> {
    let result: Option<Vec<Exchange<CanonicalAddr>>> = load(storage, EXCHANGES_KEY)?;
    Ok(result.unwrap_or(vec![]))
}

fn save_exchanges(
    storage: &mut impl Storage,
    exchanges: &Vec<Exchange<CanonicalAddr>>
) -> StdResult<()> {
    save(storage, EXCHANGES_KEY, exchanges)
}

fn generate_ido_index(index: &u64) -> Vec<u8> {
    [ IDO_PREFIX, index.to_string().as_bytes() ].concat()
}

pub fn generate_pair_key(
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
