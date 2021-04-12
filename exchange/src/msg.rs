use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, HumanAddr, Decimal};
use shared::{TokenPair, TokenPairAmount, TokenTypeAmount};
use cosmwasm_utils::ContractInfo;

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    AddLiquidity {
        deposit: TokenPairAmount,
        /// The amount the price moves in a trading pair between when a transaction is submitted and when it is executed.
        /// Transactions that exceed this threshold will be rejected.
        slippage_tolerance: Option<Decimal>
    },
    RemoveLiquidity {
        /// The amount of LP tokens burned.
        amount: Uint128,
        /// The account to refund the tokens to.
        recipient: HumanAddr
    },
    Swap {
        /// The token type to swap from.
        offer: TokenTypeAmount,
        expected_return: Option<Uint128>,
    },
    /// Sent by the LP token contract so that we can record its address.
    OnLpTokenInit
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    PairInfo,
    FactoryInfo,
    Pool,
    SwapSimulation {
        /// The token type to swap from.
        offer: TokenTypeAmount
    }
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgResponse {
    PairInfo(TokenPair),
    FactoryInfo(ContractInfo),
    Pool(TokenPairAmount)
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct SwapSimulationResponse {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
    pub commission_amount: Uint128,
}
