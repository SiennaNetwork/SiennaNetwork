// Modules re-export
pub use fadroma;
pub use composable_admin as admin;
pub use composable_snip20 as snip20_impl;

pub use token_pair::*;
pub use token_pair_amount::*;
pub use token_type::*;
pub use token_type_amount::*;
pub use exchange::*;
pub use display::*;

pub mod msg;

mod token_pair;
mod token_pair_amount;
mod token_type;
mod token_type_amount;
mod exchange;
mod display;

use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8
}
