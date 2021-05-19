use cosmwasm_std::{
    Api, CanonicalAddr, Extern, HumanAddr, Querier,
    StdResult, Storage, Uint128, StdError
};
use serde::{Serialize,Deserialize};

use fadroma_scrt_addr::{Humanize, Canonize};
use fadroma_scrt_callback::ContractInstance;
use fadroma_scrt_storage::{load, save};
use cosmwasm_utils::viewing_key::ViewingKey;
use sienna_amm_shared::TokenPair;

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
    /// Typically, smart contracts which need tokens to perform some functionality
    /// require callers to first make an approval on the token contract,
    /// then call a function that in turn calls transferFrom on the token contract.
    ///
    /// This is not how Uniswap pairs accept tokens.
    /// Instead, pairs check their token balances at the end of every interaction.
    ///
    /// Then, at the beginning of the next interaction, current balances are differenced
    /// against the stored values to determine the amount of tokens that were sent by the
    /// current interactor.
    pub pool_cache: [Uint128; 2]
}
impl Canonize<Config<CanonicalAddr>> for Config<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Config<CanonicalAddr>> {
        Ok(Config {
            factory_info:  self.factory_info.canonize(api)?,
            lp_token_info: self.lp_token_info.canonize(api)?,
            pair:          self.pair.canonize(api)?,
            contract_addr: self.contract_addr.canonize(api)?,
            viewing_key:   self.viewing_key.clone(),
            pool_cache:    self.pool_cache
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
            viewing_key:   self.viewing_key.clone(),
            pool_cache:    self.pool_cache
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
    use sienna_amm_shared::TokenType;

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
            viewing_key: ViewingKey("vk".into()),
            pool_cache: [ Uint128::zero(), Uint128(123) ]
        };

        store_config(&mut deps, &config)?;

        let result = load_config(&deps)?;

        assert_eq!(config, result);

        Ok(())
    }
}
