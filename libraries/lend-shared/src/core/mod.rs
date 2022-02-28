use fadroma::schemars;
use serde::{Serialize, Deserialize};

mod interest;
mod state;
mod auth;

pub use interest::*;
pub use state::MasterKey;
pub use auth::*;

#[derive(Serialize, Deserialize, schemars::JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
#[serde(deny_unknown_fields)]
pub struct Pagination {
    pub start: u64,
    pub limit: u8,
}
