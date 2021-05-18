use cosmwasm_std::{
    Querier, QueryRequest, Empty, SystemError, from_slice,
    WasmQuery, to_binary, QuerierResult, Uint128, Extern,
    HumanAddr, from_binary, Env, MessageInfo, ContractInfo as StdContractInfo,
    BlockInfo
};
use cosmwasm_std::testing::{MockApi, MockStorage, MOCK_CONTRACT_ADDR};
use serde::{Serialize, Deserialize};
use secret_toolkit::snip20::query::{Balance, TokenInfo};
use cosmwasm_utils::ContractInfo;

#[allow(dead_code)]
pub fn mock_dependencies(
    canonical_length: usize,
    reward_token: ContractInfo,
    reward_token_supply: Uint128,
    reward_token_decimals: u8
) -> Extern<MockStorage, MockApi, MockSnip20Querier> {
    Extern {
        storage: MockStorage::default(),
        api: MockApi::new(canonical_length),
        querier: MockSnip20Querier {
            reward_token,
            reward_token_supply,
            reward_token_decimals
        }
    }
}

#[allow(dead_code)]
pub fn mock_env_with_time(sender: impl Into<HumanAddr>, time: u64) -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time,
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        message: MessageInfo {
            sender: sender.into(),
            sent_funds: vec![],
        },
        contract: StdContractInfo {
            address: HumanAddr::from(MOCK_CONTRACT_ADDR),
        },
        contract_key: Some("".to_string()),
        contract_code_hash: "".to_string(),
    }
}

pub struct MockSnip20Querier {
    pub reward_token_supply: Uint128,
    reward_token: ContractInfo,
    reward_token_decimals: u8
}

// Redefine here, so we can deserialize
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum QueryMsg {
    TokenInfo {},
    Balance {
        address: HumanAddr,
        key: String,
    }
}

#[derive(Serialize, Deserialize)]
struct TokenInfoResponse {
    pub token_info: TokenInfo,
}

#[derive(Serialize, Deserialize)]
struct BalanceResponse {
    pub balance: Balance,
}

impl Querier for MockSnip20Querier {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                });
            }
        };

        match request {
            QueryRequest::Wasm(WasmQuery::Smart { 
                callback_code_hash, contract_addr, msg
             }) => {
                let msg: QueryMsg = from_binary(&msg).unwrap();

                match msg {
                    QueryMsg::Balance { .. } => {
                        let info = ContractInfo {
                            code_hash: callback_code_hash,
                            address: contract_addr
                        };
        
                        if info != self.reward_token {
                            panic!("MockSnip20Querier: Expected balance query for {:?}", self.reward_token)
                        }
        
                        Ok(to_binary(&BalanceResponse { 
                            balance: Balance {
                                amount: self.reward_token_supply
                            }
                        }))
                    },
                    QueryMsg::TokenInfo { } => {
                        Ok(to_binary(&TokenInfoResponse { 
                                token_info: TokenInfo {
                                name: "reward_token".into(),
                                symbol: "REWARD".into(),
                                decimals: self.reward_token_decimals,
                                total_supply: None
                            }
                        }))
                    }
                }

            },
            _ => panic!("MockSnip20Querier: Expected WasmQuery::Smart.")
        }
    }
}
