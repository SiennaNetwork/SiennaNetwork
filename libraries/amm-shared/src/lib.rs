use fadroma::{Api, CanonicalAddr, Canonize, HumanAddr, StdResult};
// Modules re-export
pub use fadroma;
pub use fadroma_snip20_impl as snip20_impl;

pub use exchange::*;
pub use token_pair::*;
pub use token_pair_amount::*;
pub use token_type::*;
pub use token_type_amount::*;

pub mod msg;

#[cfg(not(target_arch = "wasm32"))]
// This is instead of declaring it as a testing package due to limit of re-exporting testing packages
pub mod querier;

mod display;
mod exchange;
mod token_pair;
mod token_pair_amount;
mod token_type;
mod token_type_amount;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}

pub struct Sender {
    pub human: HumanAddr,
    pub canonical: CanonicalAddr,
}
impl Sender {
    pub fn from_human(human: &HumanAddr, api: &impl Api) -> StdResult<Self> {
        let canonical = api.canonical_address(human)?;
        Ok(Self {
            human: human.clone(),
            canonical,
        })
    }
}
