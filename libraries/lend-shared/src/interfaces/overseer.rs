use fadroma::{
    admin, cosmwasm_std,
    cosmwasm_std::{
        to_binary, Api, Binary, CanonicalAddr, HandleResponse, HumanAddr, InitResponse, Querier,
        QueryRequest, StdError, StdResult, Uint128, WasmQuery,
    },
    derive_contract::*,
    permit::Permit,
    schemars, Canonize, ContractInstantiationInfo, ContractLink, Decimal256, Humanize, Uint256,
};

use serde::{Deserialize, Serialize};

#[interface(component(path = "admin"))]
pub trait Overseer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        close_factor: Decimal256,
        // Liquidation incentive
        premium: Decimal256,
        // Oracle instantiation info
        oracle_contract: ContractInstantiationInfo,
        // Price source for the oracle
        oracle_source: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse>;

    #[handle]
    fn register_oracle() -> StdResult<HandleResponse>;

    #[handle]
    fn whitelist(market: Market<HumanAddr>) -> StdResult<HandleResponse>;

    #[handle]
    fn enter(markets: Vec<HumanAddr>) -> StdResult<HandleResponse>;

    #[handle]
    fn exit(market_address: HumanAddr) -> StdResult<HandleResponse>;

    #[query("whitelist")]
    fn markets(pagination: Pagination) -> StdResult<Vec<Market<HumanAddr>>>;

    #[query("entered_markets")]
    fn entered_markets(permit: Permit<OverseerPermissions>) -> StdResult<Vec<Market<HumanAddr>>>;

    #[query("liquidity")]
    fn account_liquidity(
        permit: Permit<OverseerPermissions>,
        market: Option<HumanAddr>,
        redeem_amount: Uint128,
        borrow_amount: Uint128,
    ) -> StdResult<AccountLiquidity>;

    #[query("id")]
    fn id(permit: Permit<OverseerPermissions>) -> StdResult<Binary>;

    #[query("config")]
    fn config() -> StdResult<Config>;
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OverseerPermissions {
    AccountInfo,
    Id,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AccountLiquidity {
    /// The USD value borrowable by the user, before it reaches liquidation.
    pub liquidity: Uint256,
    /// If > 0 the account is currently below the collateral requirement and is subject to liquidation.
    pub shortfall: Uint256,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// The percentage of a liquidatable account's borrow that can be repaid in a single liquidate transaction.
    /// If a user has multiple borrowed assets, the closeFactor applies to any single borrowed asset,
    /// not the aggregated value of a user’s outstanding borrowing.
    pub close_factor: Decimal256,
    /// The discount on collateral that a liquidator receives.
    pub premium: Decimal256,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Market<A> {
    pub contract: ContractLink<A>,
    /// The symbol of the underlying asset.
    pub symbol: String,
    /// The percentage rate at which tokens can be borrowed given the size of the collateral.
    pub ltv_ratio: Decimal256,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}

impl<T> Market<T> {
    pub fn validate(&self) -> StdResult<()> {
        if self.ltv_ratio > Decimal256::one() {
            return Err(StdError::generic_err("LTV ratio must be between 0 and 1."));
        }

        Ok(())
    }
}

impl Canonize<Market<CanonicalAddr>> for Market<HumanAddr> {
    fn canonize(&self, api: &impl Api) -> StdResult<Market<CanonicalAddr>> {
        Ok(Market {
            symbol: self.symbol.clone(),
            contract: self.contract.canonize(api)?,
            ltv_ratio: self.ltv_ratio,
        })
    }
}

impl Humanize<Market<HumanAddr>> for Market<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Market<HumanAddr>> {
        Ok(Market {
            symbol: self.symbol.clone(),
            contract: self.contract.humanize(api)?,
            ltv_ratio: self.ltv_ratio,
        })
    }
}

pub fn query_account_liquidity(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    permit: Permit<OverseerPermissions>,
    market: Option<HumanAddr>,
    redeem_amount: Uint128,
    borrow_amount: Uint128,
) -> StdResult<AccountLiquidity> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::AccountLiquidity {
            permit,
            market,
            redeem_amount,
            borrow_amount,
        })?,
    }))?;

    match result {
        QueryResponse::AccountLiquidity { liquidity } => Ok(liquidity),
        _ => Err(StdError::generic_err(
            "Expecting QueryResponse::AccountLiquidity",
        )),
    }
}
