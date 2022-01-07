use lend_shared::{
    impl_contract_storage,
    fadroma::{
        HumanAddr, CanonicalAddr, StdResult,
        Storage, Api, Querier, Extern, StdError,
        Canonize, Humanize,
        ContractLink,
        storage::{load, save, ns_load, ns_save},
    },
    interfaces::oracle::PriceAsset
};

pub struct Contracts;

impl Contracts {
    impl_contract_storage!(save_source, load_source, b"source");
    impl_contract_storage!(save_overseer, load_overseer, b"overseer");
}

pub struct Asset;

impl Asset {
    const NS: &'static [u8] = b"asset";

    pub fn save<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S,A,Q>,
        asset: &PriceAsset
    ) -> StdResult<()> {
        let address = asset.address.canonize(&deps.api)?;

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
