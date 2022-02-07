
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RevealCommitteeMember {
    pub address: CanonicalAddr,
    pub approved: bool
}

impl RevealCommitteeMember {
 //   pub const ADDRESS: &'static [u8] = b"/gov/poll/committee/";
}