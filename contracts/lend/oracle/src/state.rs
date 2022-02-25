use std::borrow::Borrow;

use lend_shared::{
    impl_contract_storage,
    fadroma::{
        HumanAddr, CanonicalAddr, StdResult,
        Storage, Api, Querier, Extern, StdError,
        Canonize, Humanize,
        ContractLink,
        storage::{load, save, ns_load, ns_save},
    },
    interfaces::oracle::{Asset, AssetType}
};

pub struct Contracts;

impl Contracts {
    impl_contract_storage!(save_source, load_source, b"source");
    impl_contract_storage!(save_overseer, load_overseer, b"overseer");
}

pub struct SymbolTable;

impl SymbolTable {
    const NS: &'static [u8] = b"asset";

    pub fn save<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S,A,Q>,
        asset: &Asset
    ) -> StdResult<()> {
        let address = asset.address.borrow().canonize(&deps.api)?;

        ns_save(&mut deps.storage, Self::NS, address.as_slice(), &asset.symbol)
    }

    pub fn load_symbol<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S,A,Q>,
        address: &HumanAddr
    ) -> StdResult<String> {
        let canonical = address.canonize(&deps.api)?;
        let result: Option<String> =
            ns_load(&deps.storage, Self::NS, canonical.as_slice())?;

        match result {
            Some(symbol) => Ok(symbol),
            None => Err(StdError::generic_err(format!(
                "No asset symbol found for address: {}",
                address
            )))
        }
    }
}

pub fn get_symbol<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S,A,Q>,
    asset: AssetType
) -> StdResult<String> {
    match asset {
        AssetType::Symbol(symbol) => Ok(symbol),
        AssetType::Address(address) => {
            SymbolTable::load_symbol(deps, &address)
        }
    }
}
