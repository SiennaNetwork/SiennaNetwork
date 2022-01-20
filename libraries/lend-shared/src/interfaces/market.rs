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

use crate::interfaces::overseer::OverseerPermissions;
use crate::core::MasterKey;

#[interface(component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        key: MasterKey,
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
        sender: HumanAddr,
        from: HumanAddr,
        msg: Option<Binary>,
        amount: Uint128
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn redeem_token(burn_amount: Uint256) -> StdResult<HandleResponse>;

    #[handle]
    fn redeem_underlying(receive_amount: Uint256) -> StdResult<HandleResponse>;

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

    #[handle]
    fn create_viewing_key(
        entropy: String,
        padding: Option<String>
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn set_viewing_key(
        key: String,
        padding: Option<String>
    ) -> StdResult<HandleResponse>;

    #[query("amount")]
    fn balance(
        address: HumanAddr,
        key: String
    ) -> StdResult<Uint128>;

    #[query("amount")]
    fn balance_underlying(
        address: HumanAddr,
        key: String,
        block: Option<u64>
    ) -> StdResult<Uint128>;

    #[query("amount")]
    fn balance_internal(
        address: HumanAddr,
        key: MasterKey
    ) -> StdResult<Uint128>;

    #[query("state")]
    fn state(block: Option<u64>) -> StdResult<State>;

    #[query("borrow_rate_per_block")]
    fn borrow_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query("supply_rate_per_block")]
    fn supply_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query("exchange_rate")]
    fn exchange_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query("account")]
    fn account(id: Binary, block: Option<u64>) -> StdResult<AccountInfo>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State {
    /// Block number that the interest was last accrued at
    pub accrual_block: u64,
    /// Accumulator of the total earned interest rate since the opening of the market
    pub borrow_index: Decimal256,
    /// Total amount of outstanding borrows of the underlying in this market
    pub total_borrows: Uint256,
    /// Total amount of reserves of the underlying held in this market
    pub total_reserves: Uint256,
    /// Total number of tokens in circulation
    pub total_supply: Uint256,
    /// The amount of the underlying token that the market has.
    pub underlying_balance: Uint128,
    /// Values in the contract that rarely change.
    pub config: Config
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    // Initial exchange rate used when minting the first slTokens (used when totalSupply = 0)
    pub initial_exchange_rate: Decimal256,
    // Fraction of interest currently set aside for reserves
    pub reserve_factor: Decimal256
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
    block: Option<u64>
) -> StdResult<Decimal256> {
    let result = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::ExchangeRate { block })?,
    }))?;
    match result {
        QueryResponse::ExchangeRate { exchange_rate } => Ok(exchange_rate),
        _ => Err(StdError::generic_err(
            "Expecting QueryResponse::ExchangeRate",
        )),
    }
}

pub fn query_account(
    querier: &impl Querier,
    market: ContractLink<HumanAddr>,
    id: Binary,
    block: Option<u64>
) -> StdResult<AccountInfo> {
    let result: QueryResponse = querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::Account {
            id,
            block
        })?
    }))?;

    match result {
        QueryResponse::Account { account } => {
            Ok(account)
        },
        _ => Err(StdError::generic_err("Expected QueryResponse::Account"))
    }
}
