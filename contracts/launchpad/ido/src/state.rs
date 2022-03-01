use amm_shared::fadroma::scrt::{
    cosmwasm_std::{ 
        StdResult, Extern, Storage, Querier, Api,
        CanonicalAddr, HumanAddr, StdError
    },
    addr::{Canonize, Humanize},
    storage::{save, load, ns_save, ns_load}
};

use crate::data::{Account, Config};

impl Config<HumanAddr> {
    const KEY: &'static [u8] = b"config";

    pub fn save<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
    ) -> StdResult<()> {
        save(&mut deps.storage, Self::KEY, &self.canonize(&deps.api)?)
    }

    pub fn load<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Self> {
        let result: Option<Config<CanonicalAddr>> = load(&deps.storage, Self::KEY)?;
        result
            .ok_or(StdError::generic_err("Config doesn't exist in storage."))?
            .humanize(&deps.api)
    }
}

impl Account<HumanAddr> {
    const KEY: &'static [u8] = b"accounts";

    pub fn save<S: Storage, A: Api, Q: Querier>(
        &self,
        deps: &mut Extern<S, A, Q>,
    ) -> StdResult<()> {
        let account = self.canonize(&deps.api)?;

        ns_save(
            &mut deps.storage,
            Self::KEY,
            account.owner.as_slice(),
            &account,
        )
    }

    pub fn load<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        address: &HumanAddr,
    ) -> StdResult<Option<Self>> {
        let address = address.canonize(&deps.api)?;
        let result: Option<Account<CanonicalAddr>> =
            ns_load(&deps.storage, Self::KEY, address.as_slice())?;
        if let Some(acc) = result {
            Ok(Some(acc.humanize(&deps.api)?))
        } else {
            Ok(None)
        }
    }
}
