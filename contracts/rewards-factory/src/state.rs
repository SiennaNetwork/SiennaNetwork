use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier, StdResult,
    Storage
};
use serde::{Deserialize, Serialize};
use fadroma_scrt_callback::ContractInstantiationInfo;
use fadroma_scrt_addr::{Canonize, Humanize};
use fadroma_scrt_storage::{load, save};

const CONFIG_KEY: &[u8] = b"config";
const POOL_INDEX: &[u8] = b"pool_index";

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct Config {
    pub reward_contract: ContractInstantiationInfo
}

pub(crate) fn save_config(
    storage: &mut impl Storage,
    config: &Config
) -> StdResult<()> {
    save(storage, CONFIG_KEY, &config)
}

pub(crate) fn load_config(storage: &impl Storage) -> StdResult<Config> {
    load(storage, CONFIG_KEY)?.unwrap()
}

pub(crate) fn store_pool_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: Vec<HumanAddr>
) -> StdResult<()>{
    let mut index: Vec<CanonicalAddr> = load_pool_index(deps)?;

    for addr in addresses {
        let addr = addr.canonize(&deps.api)?;
        index.push(addr);
    }

    save(&mut deps.storage, POOL_INDEX, &index)
}

pub(crate) fn delete_pool_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: Vec<HumanAddr>
) -> StdResult<()>{
    let mut index: Vec<CanonicalAddr> = load_pool_index(deps)?;

    for addr in addresses {
        let addr = addr.canonize(&deps.api)?;
        let result = index.iter().position(|x| *x == addr);

        if let Some(i) = result {
            index.swap_remove(i);
        }
    }

    save(&mut deps.storage, POOL_INDEX, &index)
}

pub(crate) fn load_pools_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<HumanAddr>> {
    let index: Vec<CanonicalAddr> = load_pool_index(deps)?;
    let mut result = Vec::with_capacity(index.len());

    for addr in index {
        let addr = addr.humanize(&deps.api)?;
        result.push(addr);
    }

    Ok(result)
}

#[inline]
fn load_pool_index<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<CanonicalAddr>> {
    load(&deps.storage, POOL_INDEX)?.unwrap_or(Ok(vec![]))
}
