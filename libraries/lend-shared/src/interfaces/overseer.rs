use fadroma::{
    admin,
    auth,
    cosmwasm_std,
    cosmwasm_std::{
        to_binary, Api, Binary, CanonicalAddr, HandleResponse, HumanAddr, InitResponse, Querier,
        QueryRequest, StdError, StdResult, WasmQuery,
    },
    derive_contract::*,
    schemars, Canonize, ContractInstantiationInfo, ContractLink, Decimal256, Humanize, Uint256,
};

use serde::{Deserialize, Serialize};

use crate::interfaces::market::Config as MarketConfig;
use crate::core::{MasterKey, AuthMethod};

#[interface(
    component(path = "admin"),
    component(path = "auth")
)]
pub trait Overseer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        entropy: Binary,
        close_factor: Decimal256,
        // Liquidation incentive
        premium: Decimal256,
        market_contract: ContractInstantiationInfo,
        // Oracle instantiation info
        oracle_contract: ContractInstantiationInfo,
        // Price source for the oracle
        oracle_source: ContractLink<HumanAddr>,
    ) -> StdResult<InitResponse>;

    #[handle]
    fn register_oracle() -> StdResult<HandleResponse>;

    #[handle]
    fn whitelist(
        config: MarketInitConfig
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn register_market() -> StdResult<HandleResponse>;

    #[handle]
    fn enter(markets: Vec<HumanAddr>) -> StdResult<HandleResponse>;

    #[handle]
    fn exit(market_address: HumanAddr) -> StdResult<HandleResponse>;

    #[handle]
    fn set_ltv_ratio(
        market: HumanAddr,
        ltv_ratio: Decimal256
    ) -> StdResult<HandleResponse>;

    #[query]
    fn markets(pagination: Pagination) -> StdResult<Vec<Market<HumanAddr>>>;

    #[query]
    fn market(address: HumanAddr) -> StdResult<Market<HumanAddr>>;

    #[query]
    fn entered_markets(method: OverseerAuth) -> StdResult<Vec<Market<HumanAddr>>>;

    #[query]
    fn account_liquidity(
        method: OverseerAuth,
        market: Option<HumanAddr>,
        block: Option<u64>,
        redeem_amount: Uint256,
        borrow_amount: Uint256
    ) -> StdResult<AccountLiquidity>;

    #[query]
    fn can_transfer_internal(
        key: MasterKey,
        address: HumanAddr,
        market: HumanAddr,
        block: u64,
        amount: Uint256,
    ) -> StdResult<bool>;

    #[query]
    fn seize_amount(
        borrowed: HumanAddr,
        collateral: HumanAddr,
        repay_amount: Uint256
    ) -> StdResult<Uint256>;

    #[query]
    fn config() -> StdResult<Config>;
}

pub type OverseerAuth = AuthMethod<OverseerPermissions>;

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

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Market<A> {
    pub contract: ContractLink<A>,
    /// The symbol of the underlying asset.
    pub symbol: String,
    /// The percentage rate at which tokens can be borrowed given the size of the collateral.
    pub ltv_ratio: Decimal256,
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct MarketInitConfig {
    // Underlying asset address.
    pub underlying_asset: ContractLink<HumanAddr>,
    /// The percentage rate at which tokens can be borrowed given the size of the collateral.
    pub ltv_ratio: Decimal256,
    // Interest model contract address.
    pub interest_model_contract: ContractLink<HumanAddr>,
    pub config: MarketConfig,
    pub prng_seed: Binary
}

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
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
    key: MasterKey,
    address: HumanAddr,
    market: Option<HumanAddr>,
    block: Option<u64>,
    redeem_amount: Uint256,
    borrow_amount: Uint256,
) -> StdResult<AccountLiquidity> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::AccountLiquidity {
            method: OverseerAuth::Internal {
                key,
                address
            },
            market,
            block,
            redeem_amount,
            borrow_amount,
        })?,
    }))
}

pub fn query_can_transfer(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    key: MasterKey,
    address: HumanAddr,
    market: HumanAddr,
    block: u64,
    amount: Uint256,
) -> StdResult<bool> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::CanTransferInternal {
            key,
            address,
            market,
            block,
            amount,
        })?,
    }))
}

pub fn query_market(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    address: HumanAddr,
) -> StdResult<Market<HumanAddr>> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::Market { address })?,
    }))
}

pub fn query_config(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>
) -> StdResult<Config> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::Config { })?
    }))
}

pub fn query_seize_amount(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    borrowed: HumanAddr,
    collateral: HumanAddr,
    repay_amount: Uint256
) -> StdResult<Uint256> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::SeizeAmount {
            borrowed,
            collateral,
            repay_amount
        })?
    }))
}

pub fn query_entered_markets(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    key: MasterKey,
    address: HumanAddr
) -> StdResult<Vec<Market<HumanAddr>>> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::EnteredMarkets {
            method: AuthMethod::Internal {
                key,
                address
            }
        })?,
    }))
}
