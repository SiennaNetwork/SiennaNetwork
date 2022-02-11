use fadroma::{Api, HumanAddr, Querier, StdError, StdResult, Storage, Uint128};
use serde::{Deserialize, Serialize};

use super::{
    governance::Governance,
    vote::{IVote, Vote, VoteType},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PollResult {
    pub poll_id: u64,
    pub yes_votes: u128,
    pub no_votes: u128,
}
impl PollResult {
    pub const SELF: &'static [u8] = b"/gov/result";

    pub fn decrement_yes(&mut self, amount: u128) -> StdResult<()> {
        self.yes_votes.checked_sub(amount).ok_or_else(|| {
            StdError::generic_err(format!(
                "Can't remove more voting power than it's available. Overflow",
            ))
        })?;
        Ok(())
    }
    pub fn decrement_no(&mut self, amount: u128) -> StdResult<()> {
        self.no_votes.checked_sub(amount).ok_or_else(|| {
            StdError::generic_err(format!(
                "Can't remove more voting power than it's available. Overflow",
            ))
        })?;
        Ok(())
    }
    pub fn increment_yes(&mut self, amount: u128) {
        self.yes_votes += amount;
    }
    pub fn increment_no(&mut self, amount: u128) {
        self.no_votes += amount;
    }
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

    fn change_vote_power(
        &mut self,
        core: &mut C,
        power: Uint128,
        sender: HumanAddr,
    ) -> StdResult<&mut Self>;
    fn change_vote_variant(
        &mut self,
        core: &mut C,
        variant: VoteType,
        sender: HumanAddr,
    ) -> StdResult<&mut Self>;
    fn remove_vote(&mut self, core: &mut C, sender: HumanAddr) -> StdResult<&mut Self>;
    fn add_vote(
        &mut self,
        core: &mut C,
        variant: VoteType,
        power: Uint128,
        sender: HumanAddr,
    ) -> StdResult<&mut Self>;
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
            no_votes: 0,
            yes_votes: 0,
        }
    }

    fn change_vote_power(
        &mut self,
        core: &mut C,
        power: Uint128,
        sender: HumanAddr,
    ) -> StdResult<&mut Self> {
        let mut vote = Vote::get(core, sender.clone(), self.poll_id)?;
        if let VoteType::Yes = vote.variant {
            self.decrement_yes(vote.vote_power)?;
            self.increment_yes(power.u128());
        } else {
            self.decrement_no(vote.vote_power)?;
            self.increment_no(power.u128());
        }
        vote.vote_power = power.u128();
        vote.store(core, sender, self.poll_id)?;

        Ok(self)
    }

    fn change_vote_variant(
        &mut self,
        core: &mut C,
        variant: VoteType,
        sender: HumanAddr,
    ) -> StdResult<&mut Self> {
        let mut vote = Vote::get(core, sender, self.poll_id)?;
        if vote.variant == variant {
            return Err(StdError::generic_err(
                "Your vote is not changed. You tried to cast the same vote. ",
            ));
        };

        if let VoteType::Yes = variant {
            self.increment_yes(vote.vote_power);
            self.decrement_no(vote.vote_power)?;
        } else {
            self.decrement_yes(vote.vote_power)?;
            self.increment_no(vote.vote_power);
        }

        vote.variant = variant;
        vote.store(core, sender, self.poll_id)?;
        Ok(self)
    }

    fn remove_vote(&mut self, core: &mut C, sender: HumanAddr) -> StdResult<&mut Self> {
        let vote = Vote::get(core, sender.clone(), self.poll_id)?;
        match vote.variant {
            VoteType::No => self.decrement_no(vote.vote_power)?,
            VoteType::Yes => self.decrement_yes(vote.vote_power)?,
        };
        Vote::remove(core, sender, self.poll_id)?;

        Ok(self)
    }

    fn add_vote(
        &mut self,
        core: &mut C,
        variant: VoteType,
        power: Uint128,
        sender: HumanAddr,
    ) -> StdResult<&mut Self> {
        if let Ok(_) = Vote::get(core, sender.clone(), self.poll_id) {
            return Err(StdError::generic_err(
                "Already voted. Can't cast a vote for a second time. ",
            ));
        }
        Vote::new(core, variant.clone(), sender.clone(), power)?.store(
            core,
            sender,
            self.poll_id,
        )?;

        if let VoteType::Yes = variant {
            self.increment_yes(power.u128());
        } else {
            self.increment_no(power.u128());
        };

        Ok(self)
    }
}
