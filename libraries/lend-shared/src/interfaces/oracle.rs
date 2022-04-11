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
        overseer: OverseerRef
    ) -> StdResult<InitResponse>;

    #[handle]
    fn update_assets(assets: Vec<Asset>) -> StdResult<HandleResponse>;

    #[query]
    fn config() -> StdResult<ConfigResponse>;

    #[query]
    fn price(
        base: AssetType,
        quote: AssetType,
        decimals: u8,
    ) -> StdResult<PriceResponse>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum OverseerRef {
    NewInstance(Callback<HumanAddr>),
    ExistingInstance(ContractLink<HumanAddr>)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum AssetType {
    Symbol(String),
    Address(HumanAddr)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Asset {
    pub address: HumanAddr,
    pub symbol: String
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PriceResponse {
    pub rate: Decimal256,
    pub last_updated_base: u64,
    pub last_updated_quote: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PricesResponse {
    pub prices: Vec<PriceResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TimeConstraints {
    pub block_time: u64,
    pub valid_timeframe: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ConfigResponse {
    pub overseer: ContractLink<HumanAddr>,
    pub source: ContractLink<HumanAddr>
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
    decimals: u8,
    time_contraints: Option<TimeConstraints>,
) -> StdResult<PriceResponse> {
    let price: PriceResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: oracle.address,
        callback_code_hash: oracle.code_hash,
        msg: to_binary(&QueryMsg::Price { base, quote, decimals })?,
    }))?;

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
