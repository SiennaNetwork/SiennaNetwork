use fadroma::{
    schemars,
    admin,
    derive_contract::*,
    cosmwasm_std::{
        StdResult, InitResponse, HumanAddr, HandleResponse,
        QueryRequest, WasmQuery, Querier, StdError, to_binary
    },
    cosmwasm_std,
    Callback, ContractLink, Decimal256
};
use serde::{Deserialize, Serialize};

#[interface(component(path = "admin"))]
pub trait Oracle {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        source: ContractLink<HumanAddr>,
        initial_assets: Vec<Asset>,
        callback: Callback<HumanAddr>
    ) -> StdResult<InitResponse>;

    #[handle]
    fn update_assets(assets: Vec<Asset>) -> StdResult<HandleResponse>;

    #[query("price")]
    fn price(
        base: AssetType,
        quote: AssetType
    ) -> StdResult<PriceResponse>;

    #[query("prices")]
    fn prices(
        base: Vec<AssetType>,
        quote: Vec<AssetType>
    ) -> StdResult<PricesResponse>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetType {
    Symbol(String),
    Address(HumanAddr)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Asset {
    pub address: HumanAddr,
    pub symbol: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct PriceResponse {
    pub rate: Decimal256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct PricesResponse {
    pub prices: Vec<PriceResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
pub struct TimeConstraints {
    pub block_time: u64,
    pub valid_timeframe: u64,
}

impl From<HumanAddr> for AssetType {
    fn from(address: HumanAddr) -> Self {
        AssetType::Address(address)
    }
}

impl From<&str> for AssetType {
    fn from(symbol: &str) -> Self {
        AssetType::Symbol(symbol.to_string())
    }
}

impl From<String> for AssetType {
    fn from(symbol: String) -> Self {
        AssetType::Symbol(symbol)
    }
}

pub fn query_price(
    querier: &impl Querier,
    oracle: ContractLink<HumanAddr>,
    base: AssetType,
    quote: AssetType,
    time_contraints: Option<TimeConstraints>,
) -> StdResult<PriceResponse> {
    let oracle_price = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: oracle.address,
        callback_code_hash: oracle.code_hash,
        msg: to_binary(&QueryMsg::Price { base, quote })?,
    }))?;

    match oracle_price {
        QueryResponse::Price { price } => {
            if let Some(time_contraints) = time_contraints {
                let valid_update_time =
                    time_contraints.block_time - time_contraints.valid_timeframe;
                if price.last_updated_base < valid_update_time
                    || price.last_updated_quote < valid_update_time
                {
                    return Err(StdError::generic_err("Price is too old"));
                }
            }

            Ok(price)
        }
        _ => Err(StdError::generic_err(
            "Expecting OracleQueryResponse::Price",
        )),
    }
}