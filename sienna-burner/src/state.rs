use cosmwasm_std::{Api, CanonicalAddr, Extern, HumanAddr, Querier, ReadonlyStorage, StdResult, Storage};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use sienna_amm_shared::storage::{save, load};
use sienna_amm_shared::{ContractInfo, ContractInfoStored};

const SIENNA_TOKEN_KEY: &[u8] = b"sienna_token";
const BURN_POOL_KEY: &[u8] = b"burn_pool";
const ADMIN_KEY: &[u8] = b"admin";
const PAIRS_PREFIX: &[u8] = b"pairs";

pub fn save_token_info<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: &ContractInfo
) -> StdResult<()> {
    let stored = info.to_stored(&deps.api)?;

    save(&mut deps.storage, SIENNA_TOKEN_KEY, &stored)
}

pub fn load_token_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<ContractInfo> {
    let stored: ContractInfoStored = load(&deps.storage, SIENNA_TOKEN_KEY)?;

    stored.to_normal(&deps.api)
}

pub fn save_burn_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<()> {
    let canonical = deps.api.canonical_address(address)?;

    save(&mut deps.storage, BURN_POOL_KEY, &canonical)
}

pub fn load_burn_pool<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<HumanAddr> {
    let canonical: CanonicalAddr = load(&deps.storage, BURN_POOL_KEY)?;

    deps.api.human_address(&canonical)
}

pub fn save_admin<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<()> {
    let canonical = deps.api.canonical_address(address)?;

    save(&mut deps.storage, ADMIN_KEY, &canonical)
}

pub fn load_admin<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<HumanAddr> {
    let canonical: CanonicalAddr= load(&deps.storage, ADMIN_KEY)?;

    deps.api.human_address(&canonical)
}

pub fn save_pair_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: &Vec<HumanAddr>
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PAIRS_PREFIX, &mut deps.storage);

    for address in addresses {
        let canonical = deps.api.canonical_address(address)?;
        storage.set(&canonical.as_slice(), &[1]);
    }

    Ok(())
}

pub fn remove_pair_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: &Vec<HumanAddr>
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PAIRS_PREFIX, &mut deps.storage);

    for address in addresses {
        let canonical = deps.api.canonical_address(address)?;
        storage.remove(canonical.as_slice());
    }

    Ok(())
}

pub fn pair_address_exists<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<bool> {
    let storage = ReadonlyPrefixedStorage::new(PAIRS_PREFIX, &deps.storage);

    let canonical = deps.api.canonical_address(address)?;
    let result = storage.get(canonical.as_slice());

    Ok(result.is_some())
}
