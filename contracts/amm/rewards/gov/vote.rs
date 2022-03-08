use amm_shared::Sender;
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteType {
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Vote {
    pub choice: VoteType,
    pub power: Uint128,
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
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn store(self, core: &mut C, sender: &Sender, poll_id: u64) -> StdResult<Self>;
    fn new(_: &C, choice: VoteType, voter: &Sender, vote_power: Uint128) -> StdResult<Self>;
    fn get(core: &C, sender: &Sender, poll_id: u64) -> StdResult<Self>;
    fn set(core: &mut C, sender: &Sender, poll_id: u64, vote: &Vote) -> StdResult<()>;
    fn increase(core: &mut C, sender: &Sender, poll_id: u64, amount: u128) -> StdResult<()>;
    fn remove(core: &mut C, sender: &Sender, poll_id: u64) -> StdResult<()>;
}

impl<S, A, Q, C> IVote<S, A, Q, C> for Vote
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn store(self, core: &mut C, sender: &Sender, poll_id: u64) -> StdResult<Self> {
        Vote::set(core, sender, poll_id, &self)?;
        Ok(self)
    }

    fn get(core: &C, sender: &Sender, poll_id: u64) -> StdResult<Self> {
        core.get_multi_ns::<Vote>(
            &[Self::VOTE, poll_id.to_be_bytes().as_slice()],
            sender.canonical.as_slice(),
        )?
        .ok_or_else(|| StdError::generic_err("Can't find vote for user on that poll"))
    }

    fn set(core: &mut C, sender: &Sender, poll_id: u64, vote: &Vote) -> StdResult<()> {
        core.set_multi_ns(
            &[Self::VOTE, poll_id.to_be_bytes().as_slice()],
            sender.canonical.as_slice(),
            vote,
        )?;
        Ok(())
    }

    fn increase(core: &mut C, sender: &Sender, poll_id: u64, amount: u128) -> StdResult<()> {
        let mut vote = Self::get(core, sender, poll_id)?;
        vote.power += Uint128(amount);
        Self::set(core, sender, poll_id, &vote)?;
        Ok(())
    }

    fn remove(core: &mut C, sender: &Sender, poll_id: u64) -> StdResult<()> {
        core.remove_multi_ns(
            &[Self::VOTE, poll_id.to_be_bytes().as_slice()],
            sender.canonical.as_slice(),
        )?;
        Ok(())
    }

    fn new(_: &C, choice: VoteType, voter: &Sender, vote_power: Uint128) -> StdResult<Self> {
        Ok(Self {
            power: vote_power,
            choice,
            voter: voter.canonical.clone(),
        })
    }
}
