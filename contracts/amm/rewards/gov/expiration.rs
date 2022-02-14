
use fadroma::*;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use core::fmt;

use crate::time_utils::Moment;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Expiration {
    AtTime(u64),
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
