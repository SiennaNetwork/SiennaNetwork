use fadroma::{
    admin, cosmwasm_std,
    cosmwasm_std::{
        to_binary, Api, Binary, CanonicalAddr, HandleResponse, HumanAddr, InitResponse, Querier,
        QueryRequest, StdError, StdResult, WasmQuery,
    },
    derive_contract::*,
    permit::Permit,
    schemars, Canonize, ContractInstantiationInfo, ContractLink, Decimal256, Humanize, Uint256,
};

use serde::{Deserialize, Serialize};

use crate::core::MasterKey;

#[interface(component(path = "admin"))]
pub trait Overseer {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        entropy: Binary,
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

    #[query("market")]
    fn market(address: HumanAddr) -> StdResult<Market<HumanAddr>>;

    #[query("entered_markets")]
    fn entered_markets(permit: Permit<OverseerPermissions>) -> StdResult<Vec<Market<HumanAddr>>>;

    #[query("liquidity")]
    fn account_liquidity(
        permit: Permit<OverseerPermissions>,
        market: Option<HumanAddr>,
        block: Option<u64>,
        redeem_amount: Uint256,
        borrow_amount: Uint256
    ) -> StdResult<AccountLiquidity>;

    #[query("liquidity")]
    fn account_liquidity_internal(
        key: MasterKey,
        address: HumanAddr,
        market: Option<HumanAddr>,
        block: Option<u64>,
        redeem_amount: Uint256,
        borrow_amount: Uint256
    ) -> StdResult<AccountLiquidity>;

    #[query("can_transfer")]
    fn can_transfer_internal(
        key: MasterKey,
        address: HumanAddr,
        market: HumanAddr,
        block: u64,
        amount: Uint256
    ) -> StdResult<bool>;

    #[query("amount")]
    fn seize_amount(
        borrowed: HumanAddr,
        collateral: HumanAddr,
        repay_amount: Uint256
    ) -> StdResult<Uint256>;

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
    key: MasterKey,
    address: HumanAddr,
    market: Option<HumanAddr>,
    block: Option<u64>,
    redeem_amount: Uint256,
    borrow_amount: Uint256,
) -> StdResult<AccountLiquidity> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::AccountLiquidityInternal {
            key,
            address,
            market,
            block,
            redeem_amount,
            borrow_amount,
        })?,
    }))?;

    match result {
        QueryResponse::AccountLiquidityInternal { liquidity } => Ok(liquidity),
        _ => Err(StdError::generic_err("Expecting QueryResponse::AccountLiquidityInternal"))
    }
}

pub fn query_id(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    permit: Permit<OverseerPermissions>,
) -> StdResult<Binary> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::Id { permit })?
    }))?;

    match result {
        QueryResponse::Id { id } => Ok(id),
        _ => Err(StdError::generic_err("Expecting QueryResponse::Id"))
    }
}

pub fn query_can_transfer(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    key: MasterKey,
    address: HumanAddr,
    market: HumanAddr,
    block: u64,
    amount: Uint256
) -> StdResult<bool> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::CanTransferInternal {
            key,
            address,
            market,
            block,
            amount
        })?
    }))?;

    match result {
        QueryResponse::CanTransferInternal { can_transfer } => Ok(can_transfer),
        _ => Err(StdError::generic_err("QueryResponse::CanTransferInternal"))
    }
}

pub fn query_market(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    address: HumanAddr,
) -> StdResult<Market<HumanAddr>> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::Market {
            address
        })?
    }))?;

    match result {
        QueryResponse::Market { market } => Ok(market),
        _ => Err(StdError::generic_err("QueryResponse::Market"))
    }
}

pub fn query_config(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>
) -> StdResult<Config> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::Config { })?
    }))?;

    match result {
        QueryResponse::Config { config } => Ok(config),
        _ => Err(StdError::generic_err("QueryResponse::Config"))
    }
}

pub fn query_seize_amount(
    querier: &impl Querier,
    overseer: ContractLink<HumanAddr>,
    borrowed: HumanAddr,
    collateral: HumanAddr,
    repay_amount: Uint256
) -> StdResult<Uint256> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: overseer.address,
        callback_code_hash: overseer.code_hash,
        msg: to_binary(&QueryMsg::SeizeAmount {
            borrowed,
            collateral,
            repay_amount
        })?
    }))?;

    match result {
        QueryResponse::SeizeAmount { amount } => Ok(amount),
        _ => Err(StdError::generic_err("QueryResponse::SeizeAmount"))
    }
}
