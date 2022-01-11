use fadroma::{
    admin,
    derive_contract::*,
    permit::Permit,
    schemars,
    cosmwasm_std,
    cosmwasm_std::{
        HumanAddr, Binary, StdResult,
        HandleResponse, InitResponse,
        Api, CanonicalAddr, StdError
    },
    Humanize, Canonize, ContractLink,
    Decimal256, Uint256, Uint128
};

use serde::{Serialize, Deserialize};

#[interface(component(path = "admin"))]
pub trait Overseer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        close_factor: Decimal256,
        premium: Decimal256
    ) -> StdResult<InitResponse>;

    #[handle]
    fn whitelist(market: Market<HumanAddr>) -> StdResult<HandleResponse>;

    #[handle]
    fn enter(markets: Vec<HumanAddr>) -> StdResult<HandleResponse>;

    #[handle]
    fn exit(market_address: HumanAddr) -> StdResult<HandleResponse>;

    #[query("whitelist")]
    fn markets(
        pagination: Pagination
    ) -> StdResult<Vec<Market<HumanAddr>>>;

    #[query("entered_markets")]
    fn entered_markets(
        permit: Permit<OverseerPermissions>
    ) -> StdResult<Vec<Market<HumanAddr>>>;

    #[query("liquidity")]
    fn account_liquidity(
        permit: Permit<OverseerPermissions>,
        market: Option<HumanAddr>,
        redeeem_amount: Uint128,
        borrow_amount: Uint128
    ) -> StdResult<AccountLiquidity>;

    #[query("config")]
    fn config() -> StdResult<Config>;
}


#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum OverseerPermissions {
    AccountInfo
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct AccountLiquidity {
    /// The USD value borrowable by the user, before it reaches liquidation.
    pub liquidity: Uint256,
    /// If > 0 the account is currently below the collateral requirement and is subject to liquidation.
    pub shortfall: Uint256
}

#[derive(Serialize, Deserialize, Clone, PartialEq, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    /// The percentage of a liquidatable account's borrow that can be repaid in a single liquidate transaction.
    /// If a user has multiple borrowed assets, the closeFactor applies to any single borrowed asset,
    /// not the aggregated value of a user’s outstanding borrowing.
    pub close_factor: Decimal256,
    /// The discount on collateral that a liquidator receives.
    pub premium: Decimal256
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Market<A> {
    pub contract: ContractLink<A>,
    /// The symbol of the underlying asset.
    pub symbol: String,
    /// The percentage rate at which tokens can be borrowed given the size of the collateral.
    pub ltv_ratio: Decimal256
}

#[derive(Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Pagination {
    pub start: u64,
    pub limit: u8
}

impl<T> Market<T> {
    pub fn validate(&self) -> StdResult<()> {
        if self.ltv_ratio > Decimal256::one() {
            return Err(StdError::generic_err("LTV ratio must be between 0 and 1."))
        }

        Ok(())
    }
}

impl Canonize<Market<CanonicalAddr>> for Market<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Market<CanonicalAddr>> {
        Ok(Market {
            symbol: self.symbol.clone(),
            contract: self.contract.canonize(api)?,
            ltv_ratio: self.ltv_ratio
        })
    }
}

impl Humanize<Market<HumanAddr>> for Market<CanonicalAddr> {
    fn humanize (&self, api: &impl Api) -> StdResult<Market<HumanAddr>> {
        Ok(Market {
            symbol: self.symbol.clone(),
            contract: self.contract.humanize(api)?,
            ltv_ratio: self.ltv_ratio
        })
    }
}
