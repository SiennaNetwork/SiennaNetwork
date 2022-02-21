use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteType {
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Vote {
    pub variant: VoteType,
    pub vote_power: u128,
    pub voter: CanonicalAddr,
}

impl Vote {
    pub const VOTE: &'static [u8] = b"/gov/vote/";


    fn build_prefix(poll_id: u64) -> StdResult<Vec<u8>> {
        let poll_id = poll_id.to_be_bytes().to_vec();

        //TODO: Cleaner way to handle this?
        let len = Self::VOTE.len() + poll_id.as_slice().len();

        let mut key = Vec::with_capacity(len);
        key.extend_from_slice(Self::VOTE);
        key.extend_from_slice(poll_id.as_slice());

        Ok(key)
    }

}
pub trait IVote<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn store(self, core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<Self>;
    fn new(core: &C, variant: VoteType, voter: HumanAddr, vote_power: Uint128) -> StdResult<Self>;
    fn get(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Self>;
    fn set(core: &mut C, address: HumanAddr, poll_id: u64, vote: &Vote) -> StdResult<()>;
    fn increase(core: &mut C, address: HumanAddr, poll_id: u64, amount: u128) -> StdResult<()>;
    fn remove(core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()>;
}

impl<S, A, Q, C> IVote<S, A, Q, C> for Vote
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn store(self, core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<Self> {
        Vote::set(core, address, poll_id, &self)?;
        Ok(self)
    }

    fn get(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Self> {
        let prefix = Self::build_prefix(poll_id)?;
        let key = core.canonize(address)?;
        let vote = core
            .get_ns::<Vote>(&prefix, key.as_slice())?
            .ok_or(StdError::generic_err(
                "can't find vote for user on that poll",
            ))?;
        Ok(vote)
    }

    fn set(core: &mut C, address: HumanAddr, poll_id: u64, vote: &Vote) -> StdResult<()> {
        let prefix = Self::build_prefix(poll_id)?;
        let key = core.canonize(address)?;
        core.set_ns(&prefix, key.as_slice(), vote)?;
        Ok(())
    }

    fn increase(core: &mut C, address: HumanAddr, poll_id: u64, amount: u128) -> StdResult<()> {
        let mut vote = Self::get(core, address.clone(), poll_id)?;
        vote.vote_power += amount;
        Self::set(core, address, poll_id, &vote)?;
        Ok(())
    }

    fn remove(core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()> {
        //again, better way to handle concat?
        let mut prefix = Self::build_prefix(poll_id)?;
        let key = core.canonize(address)?;
        prefix.extend_from_slice(key.as_slice());

        core.storage_mut().remove(&prefix);

        Ok(())
    }

    fn new(core: &C, variant: VoteType, voter: HumanAddr, vote_power: Uint128) -> StdResult<Self> {
        let voter = core.canonize(voter)?;
        Ok(Self {
            vote_power: vote_power.u128(),
            variant,
            voter,
        })
    }
}

