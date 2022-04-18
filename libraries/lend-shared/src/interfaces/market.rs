use fadroma::{
    admin, cosmwasm_std, derive_contract::*, killswitch, schemars, schemars::JsonSchema,
    snip20_impl::msg::QueryAnswer as Snip20Response, to_binary, Binary, Callback, ContractLink,
    Decimal256, HandleResponse, HumanAddr, InitResponse, Querier, QueryRequest, StdError,
    StdResult, Uint128, Uint256, WasmQuery,
};

use serde::{Deserialize, Serialize};

use crate::interfaces::overseer::{
    AccountLiquidity,
    Market as EnteredMarket
};
use crate::core::{MasterKey, AuthMethod, Pagination};

#[interface(
    component(path = "admin"),
    component(path = "killswitch")
)]
pub trait Market {
    #[init]
    fn new(
        admin: HumanAddr,
        prng_seed: Binary,
        entropy: Binary,
        key: MasterKey,
        // Underlying asset address
        underlying_asset: ContractLink<HumanAddr>,
        // Interest model contract address
        interest_model_contract: ContractLink<HumanAddr>,
        config: Config,
        callback: Callback<HumanAddr>,
    ) -> StdResult<InitResponse>;

    /// Snip20 receiver interface
    #[handle]
    fn receive(
        sender: HumanAddr,
        from: HumanAddr,
        msg: Option<Binary>,
        amount: Uint128,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn redeem_token(burn_amount: Uint256) -> StdResult<HandleResponse>;

    #[handle]
    fn redeem_underlying(receive_amount: Uint256) -> StdResult<HandleResponse>;

    #[handle]
    fn borrow(amount: Uint256) -> StdResult<HandleResponse>;

    #[handle]
    fn transfer(recipient: HumanAddr, amount: Uint256) -> StdResult<HandleResponse>;

    #[handle]
    fn send(
        recipient: HumanAddr,
        recipient_code_hash: Option<String>,
        amount: Uint256,
        msg: Option<Binary>,
        memo: Option<String>,
        padding: Option<String>,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn register_receive(
        code_hash: String, 
        padding: Option<String>
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn accrue_interest() -> StdResult<HandleResponse>;

    #[handle]
    fn seize(
        liquidator: HumanAddr,
        borrower: HumanAddr,
        amount: Uint256,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn update_config(
        interest_model: Option<ContractLink<HumanAddr>>,
        reserve_factor: Option<Decimal256>,
        borrow_cap: Option<Uint256>,
    ) -> StdResult<HandleResponse>;

    #[handle]
    fn reduce_reserves(amount: Uint128, to: Option<HumanAddr>) -> StdResult<HandleResponse>;

    #[handle]
    fn create_viewing_key(entropy: String, padding: Option<String>) -> StdResult<HandleResponse>;

    #[handle]
    fn set_viewing_key(key: String, padding: Option<String>) -> StdResult<HandleResponse>;

    #[query]
    fn token_info() -> StdResult<Snip20Response>;

    #[query]
    fn balance(address: HumanAddr, key: String) -> StdResult<Uint128>;

    #[query]
    fn balance_underlying(method: MarketAuth, block: Option<u64>) -> StdResult<Uint128>;

    #[query]
    fn state(block: Option<u64>) -> StdResult<State>;

    #[query]
    fn underlying_asset() -> StdResult<ContractLink<HumanAddr>>;

    #[query]
    fn borrow_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query]
    fn supply_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query]
    fn exchange_rate(block: Option<u64>) -> StdResult<Decimal256>;

    #[query]
    fn account(method: MarketAuth, block: Option<u64>) -> StdResult<AccountInfo>;

    #[query]
    fn id(method: MarketAuth) -> StdResult<Binary>;

    #[query]
    fn borrowers(
        block: u64,
        pagination: Pagination
    ) -> StdResult<BorrowersResponse>;
}

pub type MarketAuth = AuthMethod<MarketPermissions>;

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum MarketPermissions {
    AccountInfo,
    Balance,
    Id,
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
    pub config: Config,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Initial exchange rate used when minting the first slTokens (used when totalSupply = 0)
    pub initial_exchange_rate: Decimal256,
    /// Fraction of interest currently set aside for reserves
    pub reserve_factor: Decimal256,
    /// Share of seized collateral that is added to reserves
    pub seize_factor: Decimal256,
}

impl Config {
    pub fn set_reserve_factor(&mut self, new: Decimal256) -> StdResult<()> {
        Self::validate_reserve_factor(&new)?;

        self.reserve_factor = new;

        Ok(())
    }

    fn validate_reserve_factor(reserve_factor: &Decimal256) -> StdResult<()> {
        if *reserve_factor > Decimal256::one() {
            return Err(StdError::generic_err("Reserve factor must be lower than or equal to 1"));
        } else {
            Ok(())
        }
    }

    fn validate_initial_exchange_rate(rate: &Decimal256) -> StdResult<()> {
        if *rate == Decimal256::zero() {
            return Err(StdError::generic_err("Initial exchange rate must be greater than 0"));
        } else {
            Ok(())
        }
    }

    pub fn validate(&self) -> StdResult<()> {
        Self::validate_initial_exchange_rate(&self.initial_exchange_rate)?;
        Self::validate_reserve_factor(&self.reserve_factor)?;

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct BorrowersResponse {
    pub entries: Vec<Borrower>,
    /// The total number of entries stored by the contract.
    pub total: u64
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct Borrower {
    pub id: Binary,
    pub principal_balance: Uint256,
    pub actual_balance: Uint256,
    pub liquidity: AccountLiquidity,
    pub markets: Vec<EnteredMarket<HumanAddr>>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BorrowerInfo {
    /// Total balance (with accrued interest), after applying the most recent balance-changing action
    pub principal: Uint256,
    /// Global borrowIndex as of the most recent balance-changing action
    pub interest_index: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct AccountInfo {
    pub sl_token_balance: Uint256,
    pub borrow_balance: Uint256,
    pub exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub enum ReceiverCallbackMsg {
    /// Deposit underlying token
    Deposit,
    Repay {
        /// Repay someone else's debt.
        borrower: Option<Binary>,
    },
    Liquidate {
        borrower: Binary,
        collateral: HumanAddr,
    },
}

pub fn query_exchange_rate(
    querier: &impl Querier,
    market: ContractLink<HumanAddr>,
    block: Option<u64>,
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
    block: Option<u64>,
) -> StdResult<AccountInfo> {
    querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: market.address,
        callback_code_hash: market.code_hash,
        msg: to_binary(&QueryMsg::Account { method, block })?,
    }))
}
