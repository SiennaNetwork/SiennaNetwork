use fadroma::{Api, Querier, StdError, StdResult, Storage, Uint128};
use serde::{Deserialize, Serialize};

use super::governance::Governance;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PollResult {
    pub poll_id: u64,
    pub yes_votes: u64,
    pub no_votes: u64,
    pub amassed_voting_power: Uint128,
}
impl PollResult {
    pub const SELF: &'static [u8] = b"/gov/result";
}

pub trait IPollResult<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn new(core: &C, poll_id: u64) -> Self;
    fn store(&self, core: &mut C) -> StdResult<()>;
    fn get(core: &C, poll_id: u64) -> StdResult<Self>;
}

impl<S, A, Q, C> IPollResult<S, A, Q, C> for PollResult
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn store(&self, core: &mut C) -> StdResult<()> {
        core.set_ns(Self::SELF, &self.poll_id.to_be_bytes(), &self)
    }

    fn get(core: &C, poll_id: u64) -> StdResult<Self> {
        Ok(core
            .get_ns::<Self>(Self::SELF, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll result"))?)
    }
    fn new(_: &C, poll_id: u64) -> Self {
        Self {
            poll_id,
            amassed_voting_power: Uint128(0),
            no_votes: 0,
            yes_votes: 0,
        }
    }
}
