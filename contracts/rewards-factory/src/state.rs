use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier, StdResult,
    Storage
};
use serde::{Deserialize, Serialize};
use fadroma_scrt_callback::{ContractInstantiationInfo, ContractInstance};
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

pub(crate) fn store_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    instances: Vec<ContractInstance<HumanAddr>>
) -> StdResult<()>{
    let mut index = load_pool_index(deps)?;

    for instance in instances {
        let instance = instance.canonize(&deps.api)?;
        index.push(instance);
    }

    save(&mut deps.storage, POOL_INDEX, &index)
}

pub(crate) fn delete_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: Vec<HumanAddr>
) -> StdResult<()>{
    let mut index = load_pool_index(deps)?;

    for addr in addresses {
        let addr = addr.canonize(&deps.api)?;
        let result = index.iter().position(|x| x.address == addr);

        if let Some(i) = result {
            index.swap_remove(i);
        }
    }

    save(&mut deps.storage, POOL_INDEX, &index)
}

pub(crate) fn load_pools<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<ContractInstance<HumanAddr>>> {
    let index = load_pool_index(deps)?;
    let mut result = Vec::with_capacity(index.len());

    for instance in index {
        let instance = instance.humanize(&deps.api)?;
        result.push(instance);
    }

    Ok(result)
}

#[inline]
fn load_pool_index<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
) -> StdResult<Vec<ContractInstance<CanonicalAddr>>> {
    load(&deps.storage, POOL_INDEX)?.unwrap_or(Ok(vec![]))
}
