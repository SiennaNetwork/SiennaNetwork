use cosmwasm_std::{Api, Extern, HumanAddr, Querier, StdResult, Storage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_utils::{ContractInfo, ContractInfoStored};
use cosmwasm_utils::storage::{load, save};
use cosmwasm_utils::viewing_key::ViewingKey;

use crate::data::*;

const CONFIG_KEY: &[u8] = b"config";
const POOLS_KEY: &[u8] = b"pools";
const ACCOUNTS_KEY: &[u8] = b"accounts";

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Config {
    pub sienna_token: ContractInfo,
    pub viewing_key: ViewingKey
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigStored {
    pub sienna_token: ContractInfoStored,
    pub viewing_key: ViewingKey
}

pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config
) -> StdResult<()> {
    let config = ConfigStored {
        sienna_token: config.sienna_token.to_stored(&deps.api)?,
        viewing_key: config.viewing_key.clone()
    };

    save(&mut deps.storage, CONFIG_KEY, &config)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Config> {
    let config: ConfigStored = load(&deps.storage, CONFIG_KEY)?;

    Ok(Config {
        sienna_token: config.sienna_token.to_normal(&deps.api)?,
        viewing_key: config.viewing_key
    })
}

pub(crate) fn add_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pools: Vec<RewardPool>
) -> StdResult<()> {
    // Don't call load_pools to avoid unnecessary converions.
    let mut stored_pools: Vec<RewardPoolStored> = load(&mut deps.storage, POOLS_KEY)?;

    for pool in pools {
        let pool = pool.to_stored(&deps.api)?;
        stored_pools.push(pool);
    }

    save(&mut deps.storage, POOLS_KEY, &stored_pools)?;

    Ok(())
}

pub(crate) fn load_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
) -> StdResult<Vec<RewardPool>> {
    let pools: Vec<RewardPoolStored> = load(&mut deps.storage, POOLS_KEY)?;

    let mut result = Vec::with_capacity(pools.len());

    for pool in pools {
        result.push(pool.to_normal(&deps.api)?)
    }

    Ok(result)
}

pub(crate) fn remove_pools<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: &Vec<HumanAddr>
) -> StdResult<()> {
    // Don't call load_pools to avoid unnecessary converions.
    let mut stored_pools: Vec<RewardPoolStored> = load(&mut deps.storage, POOLS_KEY)?;

    for addr in addresses {
        let canonical = deps.api.canonical_address(addr)?;

        let index = stored_pools
            .iter()
            .position(|p| p.lp_token.address == canonical);

        if let Some(i) = index {
            stored_pools.swap_remove(i);
        }
    }

    save(&mut deps.storage, POOLS_KEY, &stored_pools)?;

    Ok(())
}