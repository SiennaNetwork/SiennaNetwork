use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier,
    ReadonlyStorage, StdResult, Storage, StdError
};
use cosmwasm_storage::{PrefixedStorage, ReadonlyPrefixedStorage};
use fadroma_scrt_addr::{Humanize, Canonize};
use fadroma_scrt_callback::ContractInstance;
use fadroma_scrt_storage::{save, load};

const SIENNA_TOKEN_KEY: &[u8] = b"sienna_token";
const BURN_POOL_KEY: &[u8] = b"burn_pool";
const PAIRS_PREFIX: &[u8] = b"pairs";

pub fn save_token_info<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    info: &ContractInstance<HumanAddr>
) -> StdResult<()> {
    save(&mut deps.storage, SIENNA_TOKEN_KEY, &info.canonize(&deps.api)?)
}

pub fn load_token_info<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<ContractInstance<HumanAddr>> {
    let stored: ContractInstance<CanonicalAddr> = load(&deps.storage, SIENNA_TOKEN_KEY)?.ok_or(
        StdError::generic_err("Token info doesn't exist in storage.")
    )?;
    stored.humanize(&deps.api)
}

pub fn save_burn_pool<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<()> {
    save(&mut deps.storage, BURN_POOL_KEY, &address.canonize(&deps.api)?)
}

pub fn load_burn_pool<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<HumanAddr> {
    let address: CanonicalAddr = load(&deps.storage, BURN_POOL_KEY)?.ok_or_else(||
        StdError::generic_err("Burn pool address doesn't exist in storage.")
    )?;
    address.humanize(&deps.api)
}

pub fn save_pair_addresses<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    addresses: &Vec<HumanAddr>
) -> StdResult<()> {
    let mut storage = PrefixedStorage::new(PAIRS_PREFIX, &mut deps.storage);
    for address in addresses {
        let canonical = address.canonize(&deps.api)?;
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
        let canonical = address.canonize(&deps.api)?;
        storage.remove(canonical.as_slice());
    }
    Ok(())
}

pub fn pair_address_exists<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<bool> {
    let storage = ReadonlyPrefixedStorage::new(PAIRS_PREFIX, &deps.storage);
    let canonical = address.canonize(&deps.api)?;
    let result = storage.get(canonical.as_slice());
    Ok(result.is_some())
}
