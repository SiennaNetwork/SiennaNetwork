// Modules re-export
pub use composable_admin as admin;
pub use composable_auth as auth;
pub use composable_snip20 as snip20_impl;
pub use fadroma;

pub use token_pair::*;
pub use token_pair_amount::*;
pub use token_type::*;
pub use token_type_amount::*;
pub mod exchange;

pub mod msg;

#[cfg(not(target_arch = "wasm32"))]
// This is instead of declaring it as a testing package due to limit of re-exporting testing packages
pub mod querier;

mod display;
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
