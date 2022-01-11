use fadroma::{
    admin, auth::Permit, cosmwasm_std, derive_contract::*, schemars, schemars::JsonSchema, Binary,
    ContractInstantiationInfo, ContractLink, Decimal256, HandleResponse, HumanAddr, InitResponse,
    StdResult, Uint128, Uint256,
};

use serde::{Deserialize, Serialize};

use super::overseer::OverseerPermissions;

pub const VIEWING_KEY: &str = "SiennaLend"; // TODO: Should this be public?
pub const MAX_RESERVE_FACTOR: Decimal256 = Decimal256::one();
#[interface(component(path = "admin"))]
pub trait Market {
    #[init]
    fn new(
        admin: Option<HumanAddr>,
        prng_seed: Binary,
        // Underlying asset address
        underlying_asset: ContractLink<HumanAddr>,
        // SiennaLend token info
        sl_token_info: ContractInstantiationInfo,
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
    fn receive(from: HumanAddr, msg: Option<Binary>, amount: Uint128) -> StdResult<HandleResponse>;

    #[handle]
    fn register_sl_token() -> StdResult<HandleResponse>;

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

    #[query("borrower")]
    fn borrower(id: Binary) -> StdResult<BorrowerInfoResponse>;

    #[query("borrow_rate_per_block")]
    fn borrow_rate() -> StdResult<Decimal256>;

    #[query("supply_rate_per_block")]
    fn supply_rate() -> StdResult<Decimal256>;

    #[query("exchange_rate")]
    fn exchange_rate() -> StdResult<Decimal256>;

    #[query("borrow_balance")]
    fn borrow_balance(id: Binary) -> StdResult<Decimal256>;

    #[query("account_snapshot")]
    fn account_snapshot(id: Binary) -> StdResult<AccountSnapshotResponse>;
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    underlying_asset: ContractLink<HumanAddr>,
    sl_token: ContractLink<HumanAddr>,
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
    total_borrows: Decimal256,
    /// Total amount of reserves of the underlying held in this market
    total_reserves: Decimal256,
    /// Total number of tokens in circulation
    total_supply: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct BorrowerInfoResponse {
    /// Total balance (with accrued interest), after applying the most recent balance-changing action
    principal: Uint256,
    /// Global borrowIndex as of the most recent balance-changing action
    interest_index: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountSnapshotResponse {
    pub sl_token_balance: Uint128,
    pub borrow_balance: Uint128,
    pub exchange_rate: Decimal256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiverCallbackMsg {
    /// Deposit underlying token
    DepositUnderlying { permit: Permit<OverseerPermissions> },
    /// Withdraw spendable underlying token.
    /// If the amount is not given,
    /// return all spendable underlying
    /// User operation
    WithdrawUnderlying { permit: Permit<OverseerPermissions> },
}
