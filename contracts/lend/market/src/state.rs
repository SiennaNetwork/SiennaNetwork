use std::convert::{TryFrom, TryInto};

use lend_shared::{
    fadroma::{
        schemars,
        schemars::JsonSchema,
        cosmwasm_std::{
            HumanAddr, CanonicalAddr, Extern,
            StdResult, Api, Storage, Querier,
        },
        storage::{load, save},
        Canonize, Humanize,
        ContractLink, Decimal256
    },
};
use serde::{Deserialize, Serialize};

const PAGINATION_LIMIT: u8 = 30;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config<A> {
    pub underlying_asset: ContractLink<A>,
    pub overseer_contract: ContractLink<A>,
    pub sl_token: ContractLink<A>,
    pub interest_model_contract: ContractLink<A>,
}

impl Config<HumanAddr> {
    const KEY: &'static [u8] = b"config";

    pub fn save<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        config: &Self,
    ) -> StdResult<()> {
        let config = config.canonize(&deps.api)?;

        save(&mut deps.storage, Self::KEY, &config)
    }

    pub fn load<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Self> {
        let result: Config<CanonicalAddr> = load(&deps.storage, Self::KEY)?.unwrap();

        result.humanize(&deps.api)
    }
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            underlying_asset: self.underlying_asset.canonize(api)?,
            overseer_contract: self.overseer_contract.canonize(api)?,
            sl_token: self.sl_token.canonize(api)?,
            interest_model_contract: self.interest_model_contract.canonize(api)?,
        })
    }
}

impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
         Ok(Config {
            underlying_asset: self.underlying_asset.humanize(api)?,
            overseer_contract: self.overseer_contract.humanize(api)?,
            sl_token: self.sl_token.humanize(api)?,
            interest_model_contract: self.interest_model_contract.humanize(api)?,
        })
    }
}
