use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr,
    Querier, StdResult, Storage, StdError
};
use serde::{Serialize,Deserialize};

use amm_shared::fadroma::address::{Humanize, Canonize};
use amm_shared::fadroma::callback::ContractInstance;
use amm_shared::fadroma::storage::{load, save};
use amm_shared::fadroma::utils::viewing_key::ViewingKey;
use amm_shared::TokenPair;

const CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct Config<A: Clone> {
    pub factory_info:  ContractInstance<A>,
    pub lp_token_info: ContractInstance<A>,
    pub pair:          TokenPair<A>,
    /// The address of the current contract.
    pub contract_addr: A,
    /// Viewing key used for custom SNIP20 tokens.
    pub viewing_key:   ViewingKey,
}

impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            factory_info:  self.factory_info.canonize(api)?,
            lp_token_info: self.lp_token_info.canonize(api)?,
            pair:          self.pair.canonize(api)?,
            contract_addr: self.contract_addr.canonize(api)?,
            viewing_key:   self.viewing_key.clone()
        })
    }
}
impl Humanize<Config<HumanAddr>> for Config<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Config<HumanAddr>> {
        Ok(Config {
            factory_info:  self.factory_info.humanize(api)?,
            lp_token_info: self.lp_token_info.humanize(api)?,
            pair:          self.pair.humanize(api)?,
            contract_addr: self.contract_addr.humanize(api)?,
            viewing_key:   self.viewing_key.clone()
        })
    }
}

pub(crate) fn store_config <S: Storage, A: Api, Q: Querier>(
    deps:   &mut Extern<S, A, Q>,
    config: &Config<HumanAddr>
) -> StdResult<()> {
    save(&mut deps.storage, CONFIG_KEY, &config.canonize(&deps.api)?)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Config<HumanAddr>> {
    let result: Config<CanonicalAddr> = load(&deps.storage, CONFIG_KEY)?.ok_or(
        StdError::generic_err("Config doesn't exist in storage.")
    )?;
    result.humanize(&deps.api)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use amm_shared::TokenType;

    #[test]
    fn properly_stores_config() -> StdResult<()> {
        let mut deps = mock_dependencies(10, &[]);

        let config = Config {
            factory_info: ContractInstance {
                code_hash: "factory_hash".into(),
                address: HumanAddr("factory".into())
            },
            lp_token_info: ContractInstance {
                code_hash: "token_hash".into(),
                address: HumanAddr("lp_token".into())
            },
            pair: TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into()
                }
            ),
            contract_addr: HumanAddr("this".into()),
            viewing_key: ViewingKey("vk".into())
        };

        store_config(&mut deps, &config)?;

        let result = load_config(&deps)?;

        assert_eq!(config, result);

        Ok(())
    }
}
