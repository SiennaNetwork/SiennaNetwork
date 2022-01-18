use std::fmt::Display;

use fadroma::{
    admin,
    auth::Permit,
    cosmwasm_std,
    derive_contract::*,
    schemars,
    schemars::JsonSchema,
    ContractLink, Decimal256, HandleResponse, HumanAddr, InitResponse,
    StdResult, Uint128, Uint256, Binary, QueryRequest, WasmQuery, StdError,
    Querier, to_binary
};

use serde::{Deserialize, Serialize};

use super::overseer::OverseerPermissions;

#[interface(component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        // Underlying asset address
        underlying_asset: ContractLink<HumanAddr>,
        // Overseer contract address
        overseer_contract: ContractLink<HumanAddr>,
        // Interest model contract address
        interest_model_contract: ContractLink<HumanAddr>,
        initial_exchange_rate: Decimal256,
        // Fraction of interest currently set aside for reserves
        reserve_factor: Decimal256,
    ) -> StdResult<InitResponse>;

    /// Snip20 receiver interface
    #[handle]
    fn receive(
        from: HumanAddr,
        msg: Option<Binary>,
        amount: Uint128
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn redeem_token(
        permit: Permit<OverseerPermissions>,
        burn_amount: Uint256
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn redeem_underlying(
        permit: Permit<OverseerPermissions>,
        receive_amount: Uint256
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn borrow(
        permit: Permit<OverseerPermissions>,
        amount: Uint256
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn transfer(
        recipient: HumanAddr,
        amount: Uint256
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn reduce_reserves(amount: Uint128) -> StdResult<HandleResponse>;

    #[query("config")]
    fn config() -> StdResult<ConfigResponse>;

    #[query("state")]
    fn state() -> StdResult<StateResponse>;

    #[query("borrow_rate_per_block")]
    fn borrow_rate() -> StdResult<Decimal256>;

    #[query("supply_rate_per_block")]
    fn supply_rate() -> StdResult<Decimal256>;

    #[query("exchange_rate")]
    fn exchange_rate() -> StdResult<Decimal256>;

    #[query("borrow_balance")]
    fn borrow_balance(id: Binary) -> StdResult<Decimal256>;

    #[query("account_snapshot")]
    fn account_snapshot(id: Binary) -> StdResult<AccountInfo>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    underlying_asset: ContractLink<HumanAddr>,
    overseer_contract: ContractLink<HumanAddr>,
    interest_model_contract: ContractLink<HumanAddr>,
    initial_exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct StateResponse {
    /// Block number that the interest was last accrued at
    accrual_block: u64,
    /// Accumulator of the total earned interest rate since the opening of the market
    borrow_index: Decimal256,
    /// Total amount of outstanding borrows of the underlying in this market
    total_borrows: Uint128,
    /// Total amount of reserves of the underlying held in this market
    total_reserves: Uint128,
    /// Total number of tokens in circulation
    total_supply: Uint128,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Borrower {
    pub id: Binary,
    pub info: BorrowerInfo
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, JsonSchema)]
pub struct BorrowerInfo {
    /// Total balance (with accrued interest), after applying the most recent balance-changing action
    pub principal: Uint256,
    /// Global borrowIndex as of the most recent balance-changing action
    pub interest_index: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountInfo {
    pub sl_token_balance: Uint256,
    pub borrow_balance: Uint256,
    pub exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiverCallbackMsg {
    /// Deposit underlying token
    Deposit,
    Repay {
        // Repay someone else's debt.
        borrower: Option<Binary>
    }
}

impl Display for BorrowerInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "total_balance: {}, global_index: {}", self.principal, self.interest_index)
    }
}

pub fn query_exchange_rate(
    querier: &impl Querier,
    market: ContractLink<HumanAddr>,
) -> StdResult<Decimal256> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::ExchangeRate {})?,
    }))?;
    match result {
        QueryResponse::ExchangeRate { exchange_rate } => Ok(exchange_rate),
        _ => Err(StdError::generic_err(
            "Expecting QueryResponse::ExchangeRate",
        )),
    }
}
