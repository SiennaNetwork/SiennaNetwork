use cosmwasm_std::{Api, CanonicalAddr, Extern, HumanAddr, Querier, StdResult, Storage, Uint128};
use serde::{Serialize,Deserialize};

use sienna_amm_shared::{TokenPair, TokenPairStored, ContractInfo, ContractInfoStored};
use sienna_amm_shared::storage::{load, save};
use sienna_amm_shared::viewing_key::ViewingKey;

const CONFIG_KEY: &[u8] = b"config"; 

#[derive(Serialize, Deserialize, PartialEq, Debug)]
pub(crate) struct Config {
    pub factory_info: ContractInfo,
    pub lp_token_info: ContractInfo,
    pub pair: TokenPair,
    /// The address of the current contract.
    pub contract_addr: HumanAddr,
    /// Viewing key used for custom snip20 tokens.
    pub viewing_key: ViewingKey,
    /// Typically, smart contracts which need tokens to perform some functionality 
    /// require callers to first make an approval on the token contract, then call a function
    /// that in turn calls transferFrom on the token contract. This is not how Uniswap pairs accept tokens.
    /// Instead, pairs check their token balances at the end of every interaction.
    /// Then, at the beginning of the next interaction, current balances are differenced against the stored values
    /// to determine the amount of tokens that were sent by the current interactor.
    pub pool_cache: [Uint128; 2]
}

#[derive(Serialize, Deserialize, Debug)]
struct ConfigStored {
    pub factory_info: ContractInfoStored,
    pub lp_token_info: ContractInfoStored,
    pub pair: TokenPairStored,
    pub contract_addr: CanonicalAddr,
    pub viewing_key: ViewingKey,
    pub pool_cache: [Uint128; 2]
}

pub(crate) fn store_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config
) -> StdResult<()> {
    let stored = ConfigStored {
        factory_info: config.factory_info.to_stored(&deps.api)?,
        lp_token_info: config.lp_token_info.to_stored(&deps.api)?,
        pair: config.pair.to_stored(&deps.api)?,
        contract_addr: deps.api.canonical_address(&config.contract_addr)?,
        viewing_key: config.viewing_key.clone(),
        pool_cache: config.pool_cache.clone()
    };

    save(&mut deps.storage, CONFIG_KEY, &stored)
}

pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>
) -> StdResult<Config> {
    let result: ConfigStored = load(&deps.storage, CONFIG_KEY)?;

    Ok(Config {
        factory_info: result.factory_info.to_normal(&deps.api)?,
        lp_token_info: result.lp_token_info.to_normal(&deps.api)?,
        pair: result.pair.to_normal(&deps.api)?,
        contract_addr: deps.api.human_address(&result.contract_addr)?,
        viewing_key: result.viewing_key,
        pool_cache: result.pool_cache
    })
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
            factory_info: ContractInfo {
                code_hash: "factory_hash".into(),
                address: HumanAddr("factory".into())
            },
            lp_token_info: ContractInfo {
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
