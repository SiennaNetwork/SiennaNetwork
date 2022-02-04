
use fadroma::*;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

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
    fn build_key(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Vec<u8>>;
    fn get_all(core: &C, poll_id: u64) -> StdResult<Vec<Self>>;
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
        let key = Self::build_key(core, address, poll_id)?;

        core.set_ns(Self::VOTE, &key, self.clone())?;

        Ok(())
    }
    fn get_all<'a>(core: &C, poll_id: u64) -> StdResult<Vec<Self>> {
        let poll_id = poll_id.to_be_bytes().to_vec();
        let mut key = Vec::with_capacity(poll_id.as_slice().len() + Self::VOTE.len());
        key.extend_from_slice(Self::VOTE);
        key.extend_from_slice(poll_id.as_slice());

        let store = IterableStorage::<Vote>::new(&key);

        let votes = store.iter(core.storage())?.map(|vote| vote).collect();
        votes
    }
    fn get(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Self> {
        let key = Self::build_key(core, address, poll_id)?;

        let vote = core
            .get_ns::<Vote>(Self::VOTE, &key)?
            .ok_or(StdError::generic_err(
                "can't find vote for user on that poll",
            ))?;
        Ok(vote)
    }

    fn build_key(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Vec<u8>> {
        let address = core.canonize(address)?;
        let poll_id = poll_id.to_be_bytes().to_vec();

        //TODO: Cleaner way to handle this?
        let len = address.as_slice().len() + poll_id.as_slice().len();

        let mut key = Vec::with_capacity(len);
        key.extend_from_slice(poll_id.as_slice());
        key.extend_from_slice(address.as_slice());

        Ok(key)
    }
}
