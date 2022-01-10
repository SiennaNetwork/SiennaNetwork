use fadroma::{
    Binary, HandleResponse, HumanAddr, InitResponse, StdResult, Uint128,
    ContractInstantiationInfo, ContractLink,
    derive_contract::*,
    schemars, schemars::JsonSchema,
    auth::Permit,
    admin,
    uint256::Uint256,
    cosmwasm_std
};

use serde::{Deserialize, Serialize};

use super::overseer::OverseerPermissions;

#[interface(component(path = "admin"))]
pub trait Custody {
    #[init]
    fn new(
        // owner address
        owner: Option<HumanAddr>,
        prng_seed: Binary,
        // underlying asset address
        underlying_asset: ContractLink<HumanAddr>,
        // overseer contract address
        overseer_contract: ContractLink<HumanAddr>,
        // market contract address
        market_contract: ContractLink<HumanAddr>,
        // liquidation contract address
        liquidation_contract: ContractLink<HumanAddr>,
        sl_token_info: ContractInstantiationInfo,
        unbonding_period: Option<u64>,
    ) -> StdResult<InitResponse>;

    /// Snip20 receiver interface
    #[handle]
    fn receive(from: HumanAddr, msg: Option<Binary>, amount: Uint128) -> StdResult<HandleResponse>;

    #[handle]
    fn register_sl_token() -> StdResult<HandleResponse>;

    #[handle]
    fn update_config(
        liquidation_contract: Option<ContractLink<HumanAddr>>,
        unbonding_period: Option<u64>,
    ) -> StdResult<HandleResponse>;

    /// Make specified amount of tokens unspendable
    #[handle]
    fn lock_collateral(borrower: Binary, amount: Uint256) -> StdResult<HandleResponse>;

    /// Make specified amount of collateral tokens spendable
    #[handle]
    fn unlock_collateral(borrower: Binary, amount: Uint256) -> StdResult<HandleResponse>;

    /// Liquidate collateral and send liquidated collateral to `to` address
    #[handle]
    fn liquidate_collateral(
        liquidator: HumanAddr,
        borrower: Binary,
        amount: Uint256,
    ) -> StdResult<HandleResponse>;

    #[query("config")]
    fn config() -> StdResult<ConfigResponse>;

    #[query("borrower")]
    fn borrower(permit: Permit<OverseerPermissions>) -> StdResult<BorrowerResponse>;

    #[query("borrowers")]
    fn borrowers(
        start_after: Option<HumanAddr>,
        limit: Option<u32>,
    ) -> StdResult<BorrowersResponse>;

    #[query("withdrawable_unbonded")]
    fn withdrawable_unbonded(
        permit: Permit<OverseerPermissions>,
        timestamp: u64,
    ) -> StdResult<WithdrawableUnbondedResponse>;
}

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CustodyPermission;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ReceiverCallbackMsg {
    /// Deposit collateral token
    DepositCollateral { permit: Permit<OverseerPermissions> },
    /// Withdraw spendable collateral token.
    /// If the amount is not given,
    /// return all spendable collateral
    /// User operation
    WithdrawCollateral { permit: Permit<OverseerPermissions> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub collateral_token: ContractLink<HumanAddr>,
    pub overseer_contract: ContractLink<HumanAddr>,
    pub market_contract: ContractLink<HumanAddr>,
    pub liquidation_contract: ContractLink<HumanAddr>,
    pub stable_token: ContractLink<HumanAddr>,
    pub basset_token: ContractLink<HumanAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowerResponse {
    pub balance: Uint256,
    pub spendable: Uint256,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct BorrowersResponse {
    pub borrowers: Vec<BorrowerResponse>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawableUnbondedResponse {
    pub withdrawable: Uint256,
}