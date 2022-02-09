use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::governance::Governance;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteType {
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Vote {
    pub variant: VoteType,
    pub vote_power: Uint128,
    pub voter: CanonicalAddr,
}

impl Vote {
    pub const VOTE: &'static [u8] = b"/gov/vote/";
}
pub trait IVote<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn store(&self, core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()>;
    fn get(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Self>;
    fn build_prefix(core: &C, poll_id: u64) -> StdResult<Vec<u8>>;
    fn remove(core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()>;
}

impl<S, A, Q, C> IVote<S, A, Q, C> for Vote
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn store(&self, core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()> {
        let prefix = Self::build_prefix(core, poll_id)?;
        let key = core.canonize(address)?;
        core.set_ns(&prefix, key.as_slice(), self.clone())?;
        Ok(())
    }
    fn get(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Self> {
        let prefix = Self::build_prefix(core, poll_id)?;
        let key = core.canonize(address)?;
        let vote = core
            .get_ns::<Vote>(&prefix, key.as_slice())?
            .ok_or(StdError::generic_err(
                "can't find vote for user on that poll",
            ))?;
        Ok(vote)
    }

    fn build_prefix(_core: &C, poll_id: u64) -> StdResult<Vec<u8>> {
        let poll_id = poll_id.to_be_bytes().to_vec();

        //TODO: Cleaner way to handle this?
        let len = Self::VOTE.len() + poll_id.as_slice().len();

        let mut key = Vec::with_capacity(len);
        key.extend_from_slice(Self::VOTE);
        key.extend_from_slice(poll_id.as_slice());

        Ok(key)
    }
    fn remove(core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()> {
        //again, better way to handle concat?
        let mut prefix = Self::build_prefix(core, poll_id)?;
        let key = core.canonize(address)?;
        prefix.extend_from_slice(key.as_slice());

        core.storage_mut().remove(&prefix);

        Ok(())
    }
}
