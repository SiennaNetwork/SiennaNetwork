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

const CONFIG_KEY: &[u8] = b"config";
const POOL_KEY: &[u8] = b"pools";
const ACCOUNTS_KEY: &[u8] = b"accounts";

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Config<A> {
    pub reward_token: ContractInstance<A>,
    pub this_contract: ContractInstance<A>,
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

pub(crate) fn get_or_create_account<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<Account<HumanAddr>> {
    let result: Option<Account<HumanAddr>> = get_account(deps, address)?;

    if let Some(acc) = result {
        Ok(acc)
    } else {
        Ok(Account::new(address.clone()))
    }
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

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            reward_token: self.reward_token.humanize(api)?,
            this_contract: self.this_contract.humanize(api)?,
            token_decimals: self.token_decimals,
            viewing_key: self.viewing_key.clone(),
            prng_seed: self.prng_seed.clone(),
            claim_interval: self.claim_interval
        })
    }
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            reward_token: self.reward_token.canonize(api)?,
            this_contract: self.this_contract.canonize(api)?,
            token_decimals: self.token_decimals,
            viewing_key: self.viewing_key.clone(),
            prng_seed: self.prng_seed.clone(),
            claim_interval: self.claim_interval
        })
    }
}
