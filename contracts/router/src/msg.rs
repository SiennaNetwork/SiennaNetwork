use std::collections::VecDeque;

use amm_shared::{
    fadroma::scrt::{Binary, HumanAddr, Uint128},
    TokenType,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Asset {
    pub info: AssetInfo,
    pub amount: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AssetInfo {
    CustomToken {
        contract_addr: HumanAddr,
        token_code_hash: String,
        viewing_key: String,
    },
    NativeToken {
        denom: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub register_tokens: Option<Vec<TokenType<HumanAddr>>>,
    pub owner: Option<HumanAddr>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Hop {
    pub from_token: TokenType<HumanAddr>,
    pub pair_address: HumanAddr,
    pub pair_code_hash: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Route {
    pub hops: VecDeque<Hop>,
    pub expected_return: Option<Uint128>,
    pub to: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Receive {
        from: HumanAddr,
        msg: Option<Binary>,
        amount: Uint128,
    },
    FinalizeRoute {},
    RegisterTokens {
        tokens: Vec<TokenType<HumanAddr>>,
    },
    RecoverFunds {
        token: TokenType<HumanAddr>,
        amount: Uint128,
        to: HumanAddr,
        snip20_send_msg: Option<Binary>,
    },
    UpdateSettings {
        new_owner: Option<HumanAddr>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    SupportedTokens {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Snip20Swap {
    Swap {
        expected_return: Option<Uint128>,
        to: Option<HumanAddr>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NativeSwap {
    Swap {
        offer_asset: Asset,
        expected_return: Option<Uint128>,
        to: Option<HumanAddr>,
    },
}
