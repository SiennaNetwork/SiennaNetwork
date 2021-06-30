use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier, StdResult,
    Storage, Binary
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use fadroma_scrt_callback::ContractInstance;
use fadroma_scrt_addr::{Canonize, Humanize};
use fadroma_scrt_storage::{load, save, ns_load, ns_save};
use cosmwasm_utils::viewing_key::ViewingKey;

use crate::data::*;

pub(crate) const MAX_PORTIONS: u64 = 30;

const CONFIG_KEY: &[u8] = b"config";
const POOL_KEY: &[u8] = b"pools";
const ACCOUNTS_KEY: &[u8] = b"accounts";
const SNAPSHOT_COUNT_KEY: &[u8] = b"snapshot_count";
const SNAPSHOTS_PREFIX: &[u8] = b"snapshots";

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub(crate) struct Config<A> {
    pub reward_token: ContractInstance<A>,
    pub this_contract: ContractInstance<A>,
    pub factory_address: A,
    pub token_decimals: u8,
    pub viewing_key: ViewingKey,
    pub prng_seed: Binary,
    pub claim_interval: u64
}

pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config<HumanAddr>
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Config<HumanAddr>> {
    let config: Config<CanonicalAddr> = load(&deps.storage, CONFIG_KEY)?.unwrap();

    config.humanize(&deps.api)
}

pub(crate) fn load_pool<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<RewardPool<HumanAddr>> {
    let pool: RewardPool<CanonicalAddr> = load(&deps.storage, POOL_KEY)?.unwrap();

    pool.humanize(&deps.api)
}

pub(crate) fn save_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pool: &RewardPool<HumanAddr>
) -> StdResult<()> {
    let pool = pool.canonize(&deps.api)?;

    save(&mut deps.storage, POOL_KEY, &pool)
}

pub(crate) fn save_account<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    account: &Account<HumanAddr>
) -> StdResult<()> {
    let account = account.canonize(&deps.api)?;
    
    ns_save(
        &mut deps.storage,
        ACCOUNTS_KEY,
        account.owner.as_slice(),
        &account
    )
}

pub(crate) fn get_account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<Option<Account<HumanAddr>>> {
    let address = deps.api.canonical_address(address)?;

    let result: Option<Account<CanonicalAddr>> = ns_load(&deps.storage, ACCOUNTS_KEY, address.as_slice())?;

    if let Some(acc) = result {
        Ok(Some(acc.humanize(&deps.api)?))
    } else {
        Ok(None)
    }
}

pub(crate) fn create_snapshot<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>
) -> StdResult<()> {
    let mut count = load_snapshot_count(&deps.storage)?;
    count += 1;
    save_snapshot_count(&mut deps.storage, count)?;

    let pool = load_pool(deps)?;

    let key = generate_snapshot_key(count);
    let snapshot = Snapshot {
        index: count,
        amount: pool.size
    };

    save(&mut deps.storage, &key, &snapshot)
}

/// The last element of the resulting array is the latest snapshot.
pub(crate) fn get_snapshots(
    storage: &impl Storage,
    mut from_index: u64
) -> StdResult<Vec<Snapshot>> {
    let mut result = vec![]; 
    let count = load_snapshot_count(storage)?;
    
    from_index = from_index.min(count);

    // Retrieve a maximum of `MAX_PORTIONS`
    if count - from_index > MAX_PORTIONS {
        from_index = count - MAX_PORTIONS;
    }

    for i in from_index..=count {
        let key = generate_snapshot_key(i);
        let snapshot: Option<Snapshot> = load(storage, &key)?;

        if let Some(snapshot) = snapshot {
            result.push(snapshot);
        } else {
            break;
        }
    }

    Ok(result)
}

#[inline]
pub(crate) fn load_snapshot_count(storage: &impl Storage) -> StdResult<u64> {
    Ok(load(storage, SNAPSHOT_COUNT_KEY)?.unwrap_or(0))
}

#[inline]
fn generate_snapshot_key(index: u64) -> Vec<u8> {
    [ SNAPSHOTS_PREFIX, index.to_string().as_bytes() ].concat()
}

#[inline]
fn save_snapshot_count(storage: &mut impl Storage, count: u64) -> StdResult<()> {
    save(storage, SNAPSHOT_COUNT_KEY, &count)
}

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            reward_token: self.reward_token.humanize(api)?,
            this_contract: self.this_contract.humanize(api)?,
            factory_address: self.factory_address.humanize(api)?,
            token_decimals: self.token_decimals,
            viewing_key: self.viewing_key.clone(),
            prng_seed: self.prng_seed.clone(),
            claim_interval: self.claim_interval,
        })
    }
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            reward_token: self.reward_token.canonize(api)?,
            this_contract: self.this_contract.canonize(api)?,
            factory_address: self.factory_address.canonize(api)?,
            token_decimals: self.token_decimals,
            viewing_key: self.viewing_key.clone(),
            prng_seed: self.prng_seed.clone(),
            claim_interval: self.claim_interval,
        })
    }
}
