use fadroma::Uint128;
use serde::{Serialize, Deserialize};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PollResult {
    poll_id: u64,
    yes_votes: u64,
    no_votes: u64,
    total_voting_power: Uint128,
}
