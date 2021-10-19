use fadroma::{
    scrt::{
        from_binary, from_slice, testing::MockQuerier as StdMockQuerier, to_binary, Coin, Empty,
        HumanAddr, Querier, QuerierResult, QueryRequest, StdResult, SystemError, Uint128, WasmQuery,
        secret_toolkit::snip20::{Balance, TokenInfo}
    },
    scrt_link::ContractLink,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

// Redefine here, so we can deserialize
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum QueryMsg {
    TokenInfo {},
    Balance { address: HumanAddr, key: String },
}

// Redefine here, so we can serialize
#[derive(Serialize, Deserialize)]
struct IntTokenInfoResponse {
    token_info: TokenInfo,
}

// Redefine here, so we can serialize
#[derive(Serialize, Deserialize)]
struct IntBalanceResponse {
    pub balance: Balance,
}

/// MockQuerier holds an immutable table of bank balances
/// TODO: also allow querying contracts
pub struct MockQuerier<C: DeserializeOwned = Empty> {
    pub std_mock_querier: StdMockQuerier<C>,
    pub wasm: InternalWasmQuerier,
}

impl<C: DeserializeOwned> MockQuerier<C> {
    pub fn new(balances: &[(&HumanAddr, &[Coin])], tokens: Vec<MockContractInstance>) -> Self {
        MockQuerier {
            std_mock_querier: StdMockQuerier::new(balances),
            wasm: InternalWasmQuerier { tokens },
        }
    }

    /// Subtract amount from balance displayed after spending it.
    pub fn sub_balance(&mut self, amount: Uint128, address: &HumanAddr) -> StdResult<()> {
        for mut token in &mut self.wasm.tokens {
            if &token.instance.address == address {
                token.token_supply = (token.token_supply - amount)?;
            }
        }

        Ok(())
    }
}

impl<C: DeserializeOwned> Querier for MockQuerier<C> {
    fn raw_query(&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<C> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                return Err(SystemError::InvalidRequest {
                    error: format!("Parsing query request: {}", e),
                    request: bin_request.into(),
                })
            }
        };
        self.handle_query(&request)
    }
}

impl<C: DeserializeOwned> MockQuerier<C> {
    pub fn handle_query(&self, request: &QueryRequest<C>) -> QuerierResult {
        match &request {
            QueryRequest::Wasm(msg) => self.wasm.query(msg),
            _ => self.std_mock_querier.handle_query(request),
        }
    }
}

pub struct InternalWasmQuerier {
    pub tokens: Vec<MockContractInstance>,
}

pub struct MockContractInstance {
    pub instance: ContractLink<HumanAddr>,
    pub token_decimals: u8,
    pub token_supply: Uint128,
}

impl InternalWasmQuerier {
    fn query(&self, request: &WasmQuery) -> QuerierResult {
        match request {
            WasmQuery::Smart {
                callback_code_hash: _,
                contract_addr,
                msg,
            } => {
                let msg: QueryMsg = from_binary(&msg).unwrap();

                for token in &self.tokens {
                    if &token.instance.address == contract_addr {
                        match msg {
                            QueryMsg::Balance { .. } => {
                                return Ok(to_binary(&IntBalanceResponse {
                                    balance: Balance {
                                        amount: token.token_supply,
                                    },
                                }));
                            }
                            QueryMsg::TokenInfo {} => {
                                return Ok(to_binary(&IntTokenInfoResponse {
                                    token_info: TokenInfo {
                                        name: token.instance.address.to_string(),
                                        symbol: token.instance.address.to_string(),
                                        decimals: token.token_decimals,
                                        total_supply: None,
                                    },
                                }))
                            }
                        }
                    }
                }

                Err(SystemError::NoSuchContract {
                    addr: HumanAddr::from(format!("{}", contract_addr)),
                })
            }
            _ => panic!("MockQuerier: Expected WasmQuery::Smart."),
        }
    }
}
