pub mod interfaces;
pub mod core;

pub use fadroma;

#[macro_export]
macro_rules! impl_contract_storage {
    ($save_name:ident, $load_name:ident, $key:literal) => {
        pub fn $save_name<S: Storage, A: Api, Q: Querier>(
            deps: &mut Extern<S, A, Q>,
            contract: ContractLink<HumanAddr>
        ) -> StdResult<()> {
            let contract = contract.canonize(&deps.api)?;

            save(&mut deps.storage, $key, &contract)
        }
    
        pub fn $load_name<S: Storage, A: Api, Q: Querier>(
            deps: &Extern<S, A, Q>,
        ) -> StdResult<ContractLink<HumanAddr>> {
            let result: ContractLink<CanonicalAddr> =
                load(&deps.storage, $key)?.unwrap();

            result.humanize(&deps.api)
        }
    };
}

#[macro_export]
macro_rules! impl_contract_storage_option {
    ($save_name:ident, $load_name:ident, $key:literal) => {
        pub fn $save_name<S: Storage, A: Api, Q: Querier>(
            deps: &mut Extern<S, A, Q>,
            contract: ContractLink<HumanAddr>
        ) -> StdResult<()> {
            let contract = contract.canonize(&deps.api)?;

            save(&mut deps.storage, $key, &contract)
        }
    
        pub fn $load_name<S: Storage, A: Api, Q: Querier>(
            deps: &Extern<S, A, Q>,
        ) -> StdResult<Option<ContractLink<HumanAddr>>> {
            load(&deps.storage, $key)?.humanize(&deps.api)
        }
    };
}
