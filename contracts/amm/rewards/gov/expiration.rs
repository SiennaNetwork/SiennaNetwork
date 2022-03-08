use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use core::fmt;

use crate::time_utils::Moment;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
/// Utility enum used for storing and checking expiration time
pub enum Expiration {
    /// Sets the expiration time to a unix timestamp
    AtTime(Moment),
}

impl fmt::Display for Expiration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expiration::AtTime(time) => write!(f, "Expiration time {}", time),
        }
    }
}

impl Expiration {
    pub fn is_expired(&self, timestamp: Moment) -> bool {
        match self {
            Expiration::AtTime(time) => timestamp >= *time,
        }
    }
}
