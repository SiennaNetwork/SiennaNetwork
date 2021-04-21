use cosmwasm_std::HumanAddr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const KEY_SSCRT: &[u8] = b"sscrt";
pub const KEY_CSHBK: &[u8] = b"cshbk";
pub const KEY_ADMIN: &[u8] = b"admin";
pub const KEY_DATA_SENDER: &[u8] = b"datasender";
pub const PREFIX_PAIRED_TOKENS: &[u8] = b"pairedtokens";

#[derive(Serialize, Deserialize, Clone, Default, Debug, PartialEq, Eq, JsonSchema)]
pub struct Pair {
    pub asset_0: HumanAddr,
    pub asset_1: HumanAddr,
}
