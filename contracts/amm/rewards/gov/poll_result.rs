use std::convert::TryInto;

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

    
    pub fn append_votes(&mut self, amount: i128, choice: VoteType) -> StdResult<()>{

        let try_add = |vote: &mut u128, amount:i128| {
            if amount > 0 {
                *vote += amount as u128;
            } else {
                vote
                    .checked_sub(amount.abs() as u128)
                    .ok_or(StdError::generic_err(format!("Not enough voting power available")))
                    .unwrap();
            }
         };

        if let VoteType::Yes = choice {
            try_add(&mut self.yes_votes, amount);
        } else {
            try_add(&mut self.no_votes, amount);
        }
        Ok(())
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

    fn set_vote_power(
        &mut self,
        core: &mut C,
        power: Uint128,
        sender: HumanAddr,
    ) -> StdResult<&mut Self>;

    fn change_choice(
        &mut self,
        core: &mut C,
        choice: VoteType,
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

    fn set_vote_power(
        &mut self,
        core: &mut C,
        power: Uint128,
        sender: HumanAddr,
    ) -> StdResult<&mut Self> {
        let mut vote = Vote::get(core, sender.clone(), self.poll_id)?;
        let power_diff = power.u128() - vote.vote_power;
        self.append_votes(power_diff.try_into().unwrap(), vote.variant.clone())?;

        vote.vote_power = power.u128();
        vote.store(core, sender.clone(), self.poll_id)?;

        Ok(self)
    }

    fn change_choice(
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

        let voting_power: i128 = vote.vote_power.try_into().unwrap();
        if let VoteType::Yes = variant {
            self.append_votes(voting_power, VoteType::Yes)?;
            self.append_votes(- voting_power, VoteType::No)?;
        } else {
            self.append_votes(voting_power, VoteType::Yes)?;
            self.append_votes(- voting_power, VoteType::No)?;
        }

        vote.variant = variant;
        vote.store(core, sender.clone(), self.poll_id)?;
        Ok(self)
    }

    fn remove_vote(&mut self, core: &mut C, sender: HumanAddr) -> StdResult<&mut Self> {
        let vote = Vote::get(core, sender.clone(), self.poll_id)?;
        let vote_power: i128 = vote.vote_power.try_into().unwrap();
        self.append_votes(- vote_power, vote.variant)?;
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

        self.append_votes(power.u128().try_into().unwrap(), variant)?;

        Ok(self)
    }
}
