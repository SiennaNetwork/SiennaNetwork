use fadroma::{
    load, save, Api, CanonicalAddr, Canonize, ContractLink, Extern, HumanAddr,
    Humanize, Querier, StdResult, Storage, Uint128,
};

use crate::Distribution;

pub type Portion = Uint128;

pub struct State;

impl State {
    const KEY_PORTION: &'static [u8] = b"portion";
    const KEY_DISTRIBUTION: &'static [u8] = b"config";
    const KEY_TOKEN: &'static [u8] = b"token";
    const KEY_MGMT: &'static [u8] = b"mgmt";

    pub fn save_portion<S, A, Q>(deps: &mut Extern<S, A, Q>, portion: Portion) -> StdResult<()>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        save(&mut deps.storage, Self::KEY_PORTION, &portion)
    }

    pub fn load_portion<S, A, Q>(deps: &Extern<S, A, Q>) -> StdResult<Portion>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        let portion = load(&deps.storage, Self::KEY_PORTION)?.unwrap();

        Ok(portion)
    }
    pub fn save_token<S, A, Q>(
        deps: &mut Extern<S, A, Q>,
        token: ContractLink<HumanAddr>,
    ) -> StdResult<()>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        let token = token.canonize(&deps.api)?;

        save(&mut deps.storage, Self::KEY_TOKEN, &token)
    }

    pub fn load_token<S, A, Q>(deps: &Extern<S, A, Q>) -> StdResult<ContractLink<HumanAddr>>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        let token: ContractLink<CanonicalAddr> = load(&deps.storage, Self::KEY_TOKEN)?.unwrap();

        token.humanize(&deps.api)
    }

    pub fn save_mgmt<S, A, Q>(
        deps: &mut Extern<S, A, Q>,
        mgmt: ContractLink<HumanAddr>,
    ) -> StdResult<()>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        let mgmt = mgmt.canonize(&deps.api)?;

        save(&mut deps.storage, Self::KEY_MGMT, &mgmt)
    }
    pub fn load_mgmt<S, A, Q>(deps: &Extern<S, A, Q>) -> StdResult<ContractLink<HumanAddr>>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        let mgmt: ContractLink<CanonicalAddr> = load(&deps.storage, Self::KEY_MGMT)?.unwrap();

        mgmt.humanize(&deps.api)
    }
    pub fn save_distribution<S, A, Q>(
        deps: &mut Extern<S, A, Q>,
        distribution: Distribution<HumanAddr>,
    ) -> StdResult<()>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        let distribution = distribution.canonize(&deps.api)?;

        save(&mut deps.storage, Self::KEY_DISTRIBUTION, &distribution)
    }
    pub fn load_distribution<S, A, Q>(deps: &Extern<S, A, Q>) -> StdResult<Distribution<HumanAddr>>
    where
        S: Storage,
        A: Api,
        Q: Querier,
    {
        let distribution: Distribution<CanonicalAddr> =
            load(&deps.storage, Self::KEY_DISTRIBUTION)?.unwrap();

        Ok(distribution.humanize(&deps.api)?)
    }
}
