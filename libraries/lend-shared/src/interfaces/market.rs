use std::fmt::Display;

use fadroma::{
    admin,
    cosmwasm_std,
    derive_contract::*,
    schemars,
    schemars::JsonSchema,
    ContractLink, Decimal256, HandleResponse, HumanAddr, InitResponse,
    StdResult, Uint128, Uint256, Binary, QueryRequest, WasmQuery,
    Querier, Callback, to_binary
};

use serde::{Deserialize, Serialize};

use crate::interfaces::overseer::{
    AccountLiquidity,
    Market as EnteredMarket
};
use crate::core::{MasterKey, AuthMethod};

#[interface(component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: HumanAddr,
        prng_seed: Binary,
        key: MasterKey,
        // Underlying asset address
        underlying_asset: ContractLink<HumanAddr>,
        // Interest model contract address
        interest_model_contract: ContractLink<HumanAddr>,
        config: Config,
        callback: Callback<HumanAddr>
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
    fn borrow(amount: Uint256) -> StdResult<HandleResponse>;

    #[handle]
    fn transfer(
        recipient: HumanAddr,
        amount: Uint256
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn accrue_interest() -> StdResult<HandleResponse>;

    #[handle]
    fn seize(
        liquidator: HumanAddr,
        borrower: HumanAddr,
        amount: Uint256
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn reduce_reserves(
        amount: Uint128,
        to: Option<HumanAddr>
    ) -> StdResult<HandleResponse>;

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

    #[query]
    fn balance(
        address: HumanAddr,
        key: String
    ) -> StdResult<Uint128>;

    #[query]
    fn balance_underlying(
        method: MarketAuth,
        block: Option<u64>
    ) -> StdResult<Uint128>;

    #[query]
    fn balance_internal(
        address: HumanAddr,
        key: MasterKey
    ) -> StdResult<Uint128>;

    #[query]
    fn state(block: Option<u64>) -> StdResult<State>;

    #[query]
    fn borrow_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query]
    fn supply_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query]
    fn exchange_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query]
    fn account(
        method: MarketAuth,
        block: Option<u64>
    ) -> StdResult<AccountInfo>;

    #[query]
    fn borrowers(
        block: u64,
        start_after: Option<Binary>,
        limit: Option<u8>
    ) -> StdResult<Vec<Borrower>>;
}

pub type MarketAuth = AuthMethod<MarketPermissions>;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MarketPermissions {
    AccountInfo,
    Balance
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
    /// Initial exchange rate used when minting the first slTokens (used when totalSupply = 0)
    pub initial_exchange_rate: Decimal256,
    /// Fraction of interest currently set aside for reserves
    pub reserve_factor: Decimal256,
    /// Share of seized collateral that is added to reserves
    pub seize_factor: Decimal256
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Borrower {
    pub id: Binary,
    pub info: BorrowerInfo,
    pub liquidity: AccountLiquidity,
    pub markets: Vec<EnteredMarket<HumanAddr>>
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
        /// Repay someone else's debt.
        borrower: Option<Binary>
    },
    Liquidate {
        borrower: Binary,
        collateral: HumanAddr
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
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::ExchangeRate { block })?,
    }))
}

pub fn query_account(
    querier: &impl Querier,
    market: ContractLink<HumanAddr>,
    method: MarketAuth,
    block: Option<u64>
) -> StdResult<AccountInfo> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::Account {
            method,
            block
        })?
    }))
}

pub fn query_balance(
    querier: &impl Querier,
    market: ContractLink<HumanAddr>,
    key: MasterKey,
    address: HumanAddr
) -> StdResult<Uint128> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::BalanceInternal {
            key,
            address
        })?
    }))
}
