
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Api, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use utils::storage::{save, load};
use shared::{
    TokenPair, TokenPairStored, TokenTypeStored, ContractInstantiationInfo,
    ContractInfo, ContractInfoStored
};

use crate::msg::InitMsg;

const CONFIG_KEY: &[u8] = b"config";
const IDO_PREFIX: &[u8] = b"ido_";

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub snip20_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub ido_contract: ContractInstantiationInfo,
    pub sienna_token: ContractInfo
}

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub(crate) struct Exchange {
    /// The pair that the contract manages.
    pub pair: TokenPair,
    /// Address of the contract that manages the exchange.
    pub address: HumanAddr
}

#[derive(Serialize, Deserialize)]
struct ConfigStored {
    pub snip20_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub ido_contract: ContractInstantiationInfo,
    pub sienna_token: ContractInfoStored
}

impl Config {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            snip20_contract: msg.snip20_contract,
            pair_contract: msg.pair_contract,
            ido_contract: msg.ido_contract,
            sienna_token: msg.sienna_token
        }
    }
}

/// Returns StdResult<()> resulting from saving the config to storage
pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config) -> StdResult<()> {
    let config = ConfigStored {
        snip20_contract: config.snip20_contract.clone(),
        pair_contract: config.pair_contract.clone(),
        ido_contract: config.ido_contract.clone(),
        sienna_token: config.sienna_token.to_stored(&deps.api)?
    };

    save(&mut deps.storage, CONFIG_KEY, &config)
}

/// Returns StdResult<Config> resulting from retrieving the config from storage
pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Config> {
    let result: ConfigStored = load(&deps.storage, CONFIG_KEY)?;

    let config = Config {
        snip20_contract: result.snip20_contract,
        pair_contract: result.pair_contract,
        ido_contract: result.ido_contract,
        sienna_token: result.sienna_token.to_normal(&deps.api)?
    };

    Ok(config)
}

/// Returns StdResult<bool> indicating whether a pair has been created before or not.
/// Note that TokenPair(A, B) and TokenPair(B, A) is considered to be same.
pub(crate) fn pair_exists<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    pair: &TokenPair
) -> StdResult<bool> {
    let pair = pair.to_stored(&deps.api)?;
    let key = generate_pair_key(&pair);

    if let Some(_) = deps.storage.get(&key) {
        return Ok(true);
    }

    Ok(false)
}

/// Stores information about an exchange contract. Returns an `StdError` if the exchange
/// already exists or if something else goes wrong.
pub(crate) fn store_exchange<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    exchange: &Exchange
) -> StdResult<()> {
    let Exchange {
        pair,
        address
    } = exchange;

    let canonical = deps.api.canonical_address(&address)?;
    
    if let Some(_) = deps.storage.get(canonical.as_slice()) {
        return Err(StdError::generic_err("Exchange already exists"));
    }

    let pair = pair.to_stored(&deps.api)?;
    let key = generate_pair_key(&pair);

    if let Some(_) = deps.storage.get(&key) {
        return Err(StdError::generic_err("Exchange address already exists"));
    }

    save(&mut deps.storage, canonical.as_slice(), &pair)?;
    save(&mut deps.storage, &key, &canonical)?;

    Ok(())
}

/// Get the exchange pair that the given contract address manages.
pub(crate) fn get_pair_for_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    exchange_addr: &HumanAddr
) -> StdResult<TokenPair> {
    let canonical = deps.api.canonical_address(exchange_addr)?;

    let result: TokenPairStored = load(&deps.storage, canonical.as_slice())?;
    result.to_normal(&deps.api)
}

/// Get the address of an exchange contract which manages the given pair.
pub(crate) fn get_address_for_pair<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pair: &TokenPair
) -> StdResult<HumanAddr> {
    let pair = pair.to_stored(&deps.api)?;
    let key = generate_pair_key(&pair);

    let canonical = load(&deps.storage, &key)?;

    Ok(deps.api.human_address(&canonical)?)
}

pub(crate) fn store_ido_address<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<()> {
    let address = deps.api.canonical_address(&address)?;

    save(&mut deps.storage, generate_ido_key(&address).as_slice(), &address)
}
/*
pub(crate) fn get_ido_address<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    address: &HumanAddr
) -> StdResult<()> {
    let address = deps.api.canonical_address(&address)?;

    load(&deps.storage, generate_ido_key(&address).as_slice())
}
*/

fn generate_ido_key(address: &CanonicalAddr) -> Vec<u8> {
    [ IDO_PREFIX, address.as_slice() ].concat()
}

fn generate_pair_key(
    pair: &TokenPairStored
) -> Vec<u8> {
    let mut bytes: Vec<&[u8]> = Vec::new();

    match &pair.0 {
        TokenTypeStored::NativeToken { denom } => bytes.push(denom.as_bytes()),
        TokenTypeStored::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_slice())
    }

    match &pair.1 {
        TokenTypeStored::NativeToken { denom } => bytes.push(denom.as_bytes()),
        TokenTypeStored::CustomToken { contract_addr, .. } => bytes.push(contract_addr.as_slice())
    }

    bytes.sort_by(|a, b| a.cmp(&b));

    bytes.concat()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies};
    use cosmwasm_std::{HumanAddr, Storage};
    use shared::TokenType;

    fn create_deps() -> Extern<impl Storage, impl Api, impl Querier> {
        mock_dependencies(10, &[])
    }

    #[test]
    fn generates_the_same_key_for_swapped_pairs() -> StdResult<()> {
        fn cmp_pair<S: Storage, A: Api, Q: Querier>(
            deps: &Extern<S, A, Q>,
            pair: TokenPair
        ) -> StdResult<()> {
            let stored_pair = pair.to_stored(&deps.api)?;
            let key = generate_pair_key(&stored_pair);
    
            let pair = swap_pair(&pair);
    
            let stored_pair = pair.to_stored(&deps.api)?;
            let swapped_key = generate_pair_key(&stored_pair);
    
            assert_eq!(key, swapped_key);

            Ok(())
        }

        fn swap_pair(pair: &TokenPair) -> TokenPair {
            TokenPair(
                pair.1.clone(),
                pair.0.clone()
            )
        }

        let ref deps = create_deps();

        cmp_pair(
            deps,
            TokenPair(
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into()
                }
            )
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test1".into()
                },
                TokenType::NativeToken {
                    denom: "test2".into()
                },
            )
        )?;

        cmp_pair(
            deps,
            TokenPair(
                TokenType::NativeToken {
                    denom: "test3".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("third_addr".into()),
                    token_code_hash: "asd21312asd".into()
                }
            )
        )?;

        Ok(())
    }

    #[test]
    fn query_correct_exchange_info() -> StdResult<()> {
        let mut deps = create_deps();

        let pair = TokenPair (
            TokenType::CustomToken {
                contract_addr: HumanAddr("first_addr".into()),
                token_code_hash: "13123adasd".into()
            },
            TokenType::CustomToken {
                contract_addr: HumanAddr("scnd_addr".into()),
                token_code_hash: "4534qwerqqw".into()
            }  
        );

        let address = HumanAddr("ctrct_addr".into());

        let exchange = Exchange {
            pair: pair.clone(),
            address: address.clone()
        };

        store_exchange(&mut deps, &exchange)?;

        let retrieved_pair = get_pair_for_address(&deps, &exchange.address)?;
        let retrieved_address = get_address_for_pair(&deps, &pair)?;
        
        assert_eq!(pair, retrieved_pair);
        assert_eq!(address, retrieved_address);

        Ok(())
    }

    #[test]
    fn only_one_exchange_per_factory() -> StdResult<()> {
        let ref mut deps = create_deps();

        let exchange = Exchange {
            pair: TokenPair (
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into()
                }  
            ),
            address: HumanAddr("ctrct_addr".into())
        };

        store_exchange(deps, &exchange)?;

        let exchange = Exchange {
            pair: TokenPair (
                TokenType::CustomToken {
                    contract_addr: HumanAddr("scnd_addr".into()),
                    token_code_hash: "4534qwerqqw".into()
                },
                TokenType::CustomToken {
                    contract_addr: HumanAddr("first_addr".into()),
                    token_code_hash: "13123adasd".into()
                },
            ),
            address: HumanAddr("other_addr".into())
        };
        
        match store_exchange(deps, &exchange) {
            Ok(_) => Err(StdError::generic_err("Exchange already exists")),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn test_pair_exists() -> StdResult<()> {
        let ref mut deps = create_deps();

        let pair = TokenPair (
            TokenType::CustomToken {
                contract_addr: HumanAddr("first_addr".into()),
                token_code_hash: "13123adasd".into()
            },
            TokenType::NativeToken {
                denom: "test1".into()
            },
        );

        let address = HumanAddr("ctrct_addr".into());

        let exchange = Exchange {
            pair: pair.clone(),
            address: address.clone()
        };

        store_exchange(deps, &exchange)?;

        assert!(pair_exists(deps, &pair)?);

        Ok(())
    }
}
