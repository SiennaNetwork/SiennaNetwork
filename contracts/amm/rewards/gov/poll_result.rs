use fadroma::{Api, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use serde::{Deserialize, Serialize};

use super::{
    governance::Governance,
    vote::{IVote, Vote, VoteType},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PollResult {
    pub poll_id: u64,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
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
    fn add_vote(
        &mut self,
        core: &mut C,
        sender: HumanAddr,
        variant: VoteType,
        vote_power: Uint128,
    ) -> StdResult<&mut Self>;
    fn update_vote(
        &mut self,
        core: &mut C,
        variant: VoteType,
        sender: HumanAddr,
    ) -> StdResult<&mut Self>;

    fn remove_vote(&mut self, core: &mut C, sender: HumanAddr) -> StdResult<()>;
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
            no_votes: Uint128(0),
            yes_votes: Uint128(0),
        }
    }

    fn add_vote(
        &mut self,
        core: &mut C,
        sender: HumanAddr,
        variant: VoteType,
        vote_power: Uint128,
    ) -> StdResult<&mut Self> {
        let vote = Vote::new(core, variant, sender.clone(), vote_power)?;
        match &vote.variant {
            VoteType::Yes => self.yes_votes += vote.vote_power,
            VoteType::No => self.no_votes += vote.vote_power,
        };
        vote.store(core, sender, self.poll_id)?;
        Ok(self)
    }
    fn update_vote(
        &mut self,
        core: &mut C,
        variant: VoteType,
        sender: HumanAddr,
    ) -> StdResult<&mut Self> {
        let mut vote = Vote::get(core, sender.clone(), self.poll_id)?;
        if vote.variant == variant {
            return Err(StdError::generic_err(
                "Your vote is not changed. You tried to cast the same vote. ",
            ));
        };

        match variant {
            VoteType::Yes => {
                self.yes_votes += vote.vote_power;
                self.no_votes = self
                    .no_votes
                    .u128()
                    .checked_sub(vote.vote_power.u128())
                    .unwrap()
                    .into();
            }
            VoteType::No => {
                self.yes_votes = self
                    .yes_votes
                    .u128()
                    .checked_sub(vote.vote_power.u128())
                    .unwrap()
                    .into();
                self.no_votes += vote.vote_power;
            }
        };

        vote.variant = variant;
        vote.store(core, sender, self.poll_id)?;

        Ok(self)
    }

    fn remove_vote(&mut self, core: &mut C, sender: HumanAddr) -> StdResult<()> {
        let vote = Vote::get(core, sender.clone(), self.poll_id)?;

        match vote.variant {
            VoteType::Yes => {
                self.yes_votes = self
                    .yes_votes
                    .u128()
                    .checked_sub(vote.vote_power.u128())
                    .unwrap()
                    .into();
            }
            VoteType::No => {
                self.no_votes = self
                    .no_votes
                    .u128()
                    .checked_sub(vote.vote_power.u128())
                    .unwrap()
                    .into();
            }
        };

        Vote::remove(core, sender, self.poll_id)?;

        self.store(core)?;

        Ok(())
    }
}
