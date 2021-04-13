
use std::usize;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Api, CanonicalAddr, Extern, HumanAddr, Querier, StdError, StdResult, Storage};
use cosmwasm_utils::storage::{save, load};
use cosmwasm_utils::{ContractInstantiationInfo, ContractInfo, ContractInfoStored};
use shared::{TokenPair, TokenPairStored, TokenTypeStored};

use crate::msg::InitMsg;

const CONFIG_KEY: &[u8] = b"config";
const IDO_PREFIX: &[u8; 1] = b"I";
const EXCHANGES_KEY: &[u8] = b"exchanges";

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8
}

/// Represents the address of an exchange and the pair that it manages
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct Exchange {
    /// The pair that the contract manages.
    pub pair: TokenPair,
    /// Address of the contract that manages the exchange.
    pub address: HumanAddr
}

#[derive(Serialize, Deserialize)]
pub(crate) struct Config {
    pub snip20_contract: ContractInstantiationInfo,
    pub lp_token_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub ido_contract: ContractInstantiationInfo,
    pub sienna_token: ContractInfo,
    pub pair_count: u64,
    pub ido_count: u64
}

#[derive(Serialize, Deserialize)]
struct ConfigStored {
    pub snip20_contract: ContractInstantiationInfo,
    pub lp_token_contract: ContractInstantiationInfo,
    pub pair_contract: ContractInstantiationInfo,
    pub ido_contract: ContractInstantiationInfo,
    pub sienna_token: ContractInfoStored,
    pub pair_count: u64,
    pub ido_count: u64
}

#[derive(Serialize, Deserialize)]
struct ExchangeStored {
    pub pair: TokenPairStored,
    pub address: CanonicalAddr
}

impl Config {
    pub fn from_init_msg(msg: InitMsg) -> Self {
        Self {
            snip20_contract: msg.snip20_contract,
            lp_token_contract: msg.lp_token_contract,
            pair_contract: msg.pair_contract,
            ido_contract: msg.ido_contract,
            sienna_token: msg.sienna_token,
            pair_count: 0,
            ido_count: 0
        }
    }
}

/// Returns StdResult<()> resulting from saving the config to storage
pub(crate) fn save_config<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    config: &Config) -> StdResult<()> {
    let config = ConfigStored {
        snip20_contract: config.snip20_contract.clone(),
        lp_token_contract: config.lp_token_contract.clone(),
        pair_contract: config.pair_contract.clone(),
        ido_contract: config.ido_contract.clone(),
        sienna_token: config.sienna_token.to_stored(&deps.api)?,
        pair_count: config.pair_count,
        ido_count: config.ido_count
    };

    save(&mut deps.storage, CONFIG_KEY, &config)
}

/// Returns StdResult<Config> resulting from retrieving the config from storage
pub(crate) fn load_config<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<Config> {
    let result: ConfigStored = load(&deps.storage, CONFIG_KEY)?;

    let config = Config {
        snip20_contract: result.snip20_contract,
        lp_token_contract: result.lp_token_contract,
        pair_contract: result.pair_contract,
        ido_contract: result.ido_contract,
        sienna_token: result.sienna_token.to_normal(&deps.api)?,
        pair_count: result.pair_count,
        ido_count: result.ido_count
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
    pair: &TokenPair,
    address: &HumanAddr
) -> StdResult<()> {
    let canonical = deps.api.canonical_address(&address)?;

    let pair = pair.to_stored(&deps.api)?;
    let key = generate_pair_key(&pair);

    if let Some(_) = deps.storage.get(&key) {
        return Err(StdError::generic_err("Exchange already exists"));
    }
    
    save(&mut deps.storage, &key, &canonical)?;

    let mut exchanges = load_exchanges(&deps.storage)?;

    if exchanges.iter().any(|e| e.address == canonical) {
        return Err(StdError::generic_err("Exchange address already exists"));
    }

    let exchange = ExchangeStored {
        pair,
        address: canonical
    };
    exchanges.push(exchange);

    save_exchanges(&mut deps.storage, &exchanges)?;

    Ok(())
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
    address: &HumanAddr,
    config: &mut Config
) -> StdResult<()> {
    let address = deps.api.canonical_address(&address)?;
    let index = generate_ido_index(&config.ido_count);
    
    save(&mut deps.storage, index.as_slice(), &address)?;

    config.ido_count += 1;
    save_config(deps, &config)?;

    Ok(())
}

pub(crate) fn get_idos<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    config: &Config,
    pagination: Pagination
) -> StdResult<Vec<HumanAddr>> {
    if pagination.start >= config.ido_count {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(Pagination::MAX_LIMIT);
    let end = (pagination.start + limit as u64).min(config.ido_count);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);
    
    for i in pagination.start..end {
        let index = generate_ido_index(&i);
        let addr: CanonicalAddr = load(&deps.storage, index.as_slice())?;

        let human_addr = deps.api.human_address(&addr)?;
        result.push(human_addr);
    }

    Ok(result)
}

pub(crate) fn get_exchanges<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    pagination: Pagination
) -> StdResult<Vec<Exchange>> {
    let mut exchanges = load_exchanges(&deps.storage)?;

    if pagination.start as usize >= exchanges.len() {
        return Ok(vec![]);
    }

    let limit = pagination.limit.min(Pagination::MAX_LIMIT);
    let end = (pagination.start + limit as u64).min(exchanges.len() as u64);

    let mut result = Vec::with_capacity((end - pagination.start) as usize);
    
    for exchange in exchanges.drain((pagination.start as usize)..(end as usize)).collect::<Vec<ExchangeStored>>() {     
        result.push(Exchange {
            pair: exchange.pair.to_normal(&deps.api)?,
            address: deps.api.human_address(&exchange.address)?
        })
    }

    Ok(result)
}

fn load_exchanges(storage: &impl Storage) -> StdResult<Vec<ExchangeStored>> {
    let result: StdResult<Vec<ExchangeStored>> = load(storage, EXCHANGES_KEY);

    match result {
        Ok(vec) => Ok(vec),
        Err(kind) => match kind {
            StdError::SerializeErr { .. }=> Ok(vec![]),
            _ => Err(kind)
        }
    }
}

fn save_exchanges(storage: &mut impl Storage, exchanges: &Vec<ExchangeStored>) -> StdResult<()> {
    save(storage, EXCHANGES_KEY, exchanges)
}

fn generate_ido_index(index: &u64) -> Vec<u8> {
    [ IDO_PREFIX, index.to_string().as_bytes() ].concat()
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

impl Pagination {
    const MAX_LIMIT: u8 = 30;

    pub fn new(start: u64, limit: u8) -> Self {
        Self {
            start,
            limit
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::mock_dependencies;
    use cosmwasm_std::{HumanAddr, Storage};
    use shared::TokenType;

    use crate::msg::InitMsg;

    fn create_deps() -> Extern<impl Storage, impl Api, impl Querier> {
        mock_dependencies(10, &[])
    }

    fn swap_pair(pair: &TokenPair) -> TokenPair {
        TokenPair(
            pair.1.clone(),
            pair.0.clone()
        )
    }

    fn mock_config() -> Config {
        Config::from_init_msg(InitMsg {
            snip20_contract: ContractInstantiationInfo {
                id: 1,
                code_hash: "snip20_contract".into()
            },
            lp_token_contract: ContractInstantiationInfo {
                id: 2,
                code_hash: "lp_token_contract".into()
            },
            ido_contract: ContractInstantiationInfo {
                id: 3,
                code_hash: "ido_contract".into()
            },
            pair_contract: ContractInstantiationInfo {
                id: 4,
                code_hash: "pair_contract".into()
            },
            sienna_token: ContractInfo {
                code_hash: "sienna_token".into(),
                address: HumanAddr::from("sienna_ad")
            }
        })
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

        store_exchange(&mut deps, &pair, &address)?;

        let retrieved_address = get_address_for_pair(&deps, &pair)?;
        
        assert!(pair_exists(&mut deps, &pair)?);
        assert_eq!(address, retrieved_address);

        Ok(())
    }

    #[test]
    fn only_one_exchange_per_factory() -> StdResult<()> {
        let ref mut deps = create_deps();
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

        store_exchange(deps, &pair, &"first_addr".into())?;

        let swapped = swap_pair(&pair);
        
        match store_exchange(deps, &swapped, &"other_addr".into()) {
            Ok(_) => Err(StdError::generic_err("Exchange already exists")),
            Err(_) => Ok(())
        }
    }

    #[test]
    fn test_get_idos() -> StdResult<()> {
        let ref mut deps = create_deps();
        let mut config = mock_config();

        save_config(deps, &config)?;

        let mut addresses = vec![];

        for i in 0..33 {
            let addr = HumanAddr::from(format!("addr_{}", i));

            store_ido_address(deps, &addr, &mut config)?;
            addresses.push(addr);
        }

        let mut config = load_config(deps)?;

        let result = get_idos(deps, &mut config, Pagination::new(addresses.len() as u64, 20))?;
        assert_eq!(result.len(), 0);

        let result = get_idos(deps, &mut config, Pagination::new((addresses.len() - 1) as u64, 20))?;
        assert_eq!(result.len(), 1);

        let result = get_idos(deps, &mut config, Pagination::new(0, Pagination::MAX_LIMIT + 10))?;
        assert_eq!(result.len(), Pagination::MAX_LIMIT as usize);

        let result = get_idos(deps, &mut config, Pagination::new(3, Pagination::MAX_LIMIT))?;
        assert_eq!(result, addresses[3..]);

        Ok(())
    }

    #[test]
    fn test_get_exchanges() -> StdResult<()> {
        let ref mut deps = create_deps();

        let mut exchanges = vec![];

        for i in 0..33 {
            let pair = TokenPair (
                TokenType::CustomToken {
                    contract_addr: HumanAddr(format!("addr_{}", i)),
                    token_code_hash: format!("code_hash_{}", i)
                },
                TokenType::NativeToken {
                    denom: format!("denom_{}", i)
                }, 
            );
            let address = HumanAddr(format!("address_{}", i));

            store_exchange(deps, &pair, &address)?;

            exchanges.push(Exchange {
                pair,
                address
            });
        }

        let result = get_exchanges(deps, Pagination::new(exchanges.len() as u64, 20))?;
        assert_eq!(result.len(), 0);

        let result = get_exchanges(deps, Pagination::new((exchanges.len() - 1) as u64, 20))?;
        assert_eq!(result.len(), 1);

        let result = get_exchanges(deps, Pagination::new(0, Pagination::MAX_LIMIT + 10))?;
        assert_eq!(result.len(), Pagination::MAX_LIMIT as usize);

        let result = get_exchanges(deps, Pagination::new(3, Pagination::MAX_LIMIT))?;
        assert_eq!(result, exchanges[3..]);

        Ok(())
    }
}
