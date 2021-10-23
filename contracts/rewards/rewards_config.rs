use fadroma::{
    scrt_addr::{Humanize, Canonize},
    scrt_link::ContractLink,
    scrt::{ReadonlyStorage, Storage, Api, StdResult, StdError, HumanAddr, CanonicalAddr},
    scrt_storage::{load, save},
    scrt_vk::ViewingKey,
};

macro_rules! error { ($info:expr) => {
    Err(StdError::GenericErr { msg: $info.into(), backtrace: None })
} }

const POOL_SELF_REFERENCE: &[u8] = b"self";

pub fn load_self_reference(
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_SELF_REFERENCE)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing self reference")
    }
}

pub fn save_self_reference (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractLink<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_SELF_REFERENCE, &link.canonize(api)?)
}

const POOL_LP_TOKEN: &[u8] = b"lp_token";

pub fn load_lp_token (
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_LP_TOKEN)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing liquidity provision token")
    }
}

pub fn save_lp_token (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractLink<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_LP_TOKEN, &link.canonize(api)?)
}

const POOL_REWARD_TOKEN: &[u8] = b"reward_token";

pub fn load_reward_token (
    storage: &impl ReadonlyStorage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_REWARD_TOKEN)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => error!("missing liquidity provision token")
    }
}

pub fn save_reward_token (
    storage: &mut impl Storage,
    api:     &impl Api,
    link:    &ContractLink<HumanAddr>
) -> StdResult<()> {
    save(storage, POOL_REWARD_TOKEN, &link.canonize(api)?)
}

const POOL_REWARD_TOKEN_VK: &[u8] = b"reward_token_vk";

pub fn load_viewing_key (
    storage: &impl ReadonlyStorage,
) -> StdResult<ViewingKey> {
    let result: Option<ViewingKey> = load(storage, POOL_REWARD_TOKEN_VK)?;
    match result {
        Some(key) => Ok(key),
        None => error!("missing reward token viewing key")
    }
}

pub fn save_viewing_key (
    storage: &mut impl Storage,
    key:     &ViewingKey
) -> StdResult<()> {
    save(storage, POOL_REWARD_TOKEN_VK, &key)
}
