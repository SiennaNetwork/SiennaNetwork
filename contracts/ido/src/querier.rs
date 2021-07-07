use fadroma::scrt::cosmwasm_std::testing::MockQuerier as StdMockQuerier;
use fadroma::scrt::cosmwasm_std::{
    from_slice, to_binary, Coin, Empty, HumanAddr, Querier, QuerierResult, QueryRequest,
    SystemError, WasmQuery,
};
use fadroma::scrt::toolkit::snip20::TokenInfo;
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct IntTokenInfoResponse {
    token_info: TokenInfo,
}

/// MockQuerier holds an immutable table of bank balances
/// TODO: also allow querying contracts
pub struct MockQuerier<C: DeserializeOwned = Empty> {
    pub std_mock_querier: StdMockQuerier<C>,
    pub wasm: InternalWasmQuerier,
}

impl<C: DeserializeOwned> MockQuerier<C> {
    pub fn new(balances: &[(&HumanAddr, &[Coin])]) -> Self {
        MockQuerier {
            std_mock_querier: StdMockQuerier::new(balances),
            wasm: InternalWasmQuerier,
        }
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

pub struct InternalWasmQuerier;

impl InternalWasmQuerier {
    fn query(&self, request: &WasmQuery) -> QuerierResult {
        let addr = match request {
            WasmQuery::Smart { contract_addr, .. } => contract_addr,
            WasmQuery::Raw { contract_addr, .. } => contract_addr,
        }
        .clone();

        if addr.to_string().as_str() != "sold-token" {
            return Err(SystemError::NoSuchContract {
                addr: HumanAddr::from(format!("{}", addr)),
            });
        }

        let token_info = TokenInfo {
            name: "Sold token".to_string(),
            symbol: "SDT".to_string(),
            decimals: 8,
            total_supply: None,
        };

        let token_info = IntTokenInfoResponse { token_info };

        Ok(to_binary(&token_info))
    }
}
