use crate::asset::Asset;
use crate::state::Pair;
use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use cosmwasm_utils::ContractInfo;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InitMsg {
    pub sscrt_addr: HumanAddr,
    pub pairs: Option<Vec<Pair>>,
    pub cashback: ContractInfo,
}

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    ReceiveSwapData {
        asset_in: Asset,
        asset_out: Asset,
        account: HumanAddr,
    },

    // Admin
    AddPairs {
        pairs: Vec<Pair>,
    },
    RemovePairs {
        pairs: Vec<Pair>,
    },
    SetAdmin {
        address: HumanAddr,
    },
    SetDataSender {
        address: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    IsSupported { pair: Pair },
    Cashback {},
    Admin {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryAnswer {
    IsSupported { is_supported: bool },
    Cashback { address: HumanAddr },
    Admin { address: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}
