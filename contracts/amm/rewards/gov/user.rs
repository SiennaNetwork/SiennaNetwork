use amm_shared::Sender;
use fadroma::{Api, Composable, Querier, StdError, StdResult, Storage, Uint128, UsuallyOk};
use serde::{Deserialize, Serialize};

use crate::time_utils::Moment;

use super::{
    poll::{IPoll, Poll, UpdateResultReason},
    poll_result::{IPollResult, PollResult},
    vote::{IVote, Vote, VoteType},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub active_polls: Vec<u64>,
    created_polls: Vec<u64>,
}

impl User {
    pub const ACTIVE_POLLS: &'static [u8] = b"/gov/user/polls";
    pub const CREATED_POLLS: &'static [u8] = b"gov/user/created_polls";

    pub fn can_unstake(&self, balance: u128, threshold: u128, amount: u128) -> bool {
        if !self.active_polls.is_empty() {
            return false;
        }
        if !self.created_polls.is_empty() {
            return balance - amount > threshold;
        }
        true
    }
}

pub trait IUser<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn get(core: &C, sender: &Sender, now: Moment) -> StdResult<User>;
    fn store(&self, core: &mut C, sender: &Sender) -> UsuallyOk;

    fn active_polls(core: &C, sender: &Sender, now: Moment) -> StdResult<Vec<u64>>;
    fn set_active_polls(core: &mut C, address: &Sender, polls: &Vec<u64>) -> UsuallyOk;
    fn remove_active_poll(
        core: &mut C,
        sender: &Sender,
        poll_id: u64,
        timestamp: Moment,
    ) -> UsuallyOk;

    fn create_poll(core: &mut C, sender: &Sender, poll: &Poll, now: Moment) -> UsuallyOk;
    fn created_polls(core: &C, sender: &Sender, now: Moment) -> StdResult<Vec<u64>>;

    fn add_vote(
        core: &mut C,
        poll_id: u64,
        sender: &Sender,
        choice: VoteType,
        power: Uint128,
        now: Moment,
    ) -> UsuallyOk;
    fn change_choice(
        core: &mut C,
        poll_id: u64,
        sender: &Sender,
        choice: VoteType,
        now: Moment,
    ) -> UsuallyOk;
    fn increase_vote_power(
        core: &mut C,
        poll_id: u64,
        sender: &Sender,
        power_diff: Uint128,
        now: Moment,
    ) -> UsuallyOk;
    fn remove_vote(core: &mut C, poll_id: u64, sender: &Sender, now: Moment) -> UsuallyOk;
}

impl<S, A, Q, C> IUser<S, A, Q, C> for User
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn get(core: &C, sender: &Sender, now: Moment) -> StdResult<User> {
        let active_polls = User::active_polls(core, &sender, now)?;
        let created_polls = User::created_polls(core, &sender, now)?;
        Ok(Self {
            active_polls,
            created_polls,
        })
    }

    fn active_polls(core: &C, sender: &Sender, timestamp: Moment) -> StdResult<Vec<u64>> {
        let polls = core
            .get_ns::<Vec<u64>>(Self::ACTIVE_POLLS, sender.canonical.as_slice())?
            .unwrap_or_default();
        Ok(filter_active_polls(core, polls, timestamp))
    }

    fn store(&self, core: &mut C, sender: &Sender) -> UsuallyOk {
        core.set_ns(
            Self::CREATED_POLLS,
            sender.canonical.as_slice(),
            &self.created_polls,
        )?;
        User::set_active_polls(core, sender, &self.active_polls)?;
        Ok(())
    }

    /**
    Overwrites the saved active polls for given user
    */
    fn set_active_polls(core: &mut C, sender: &Sender, polls: &Vec<u64>) -> UsuallyOk {
        core.set_ns(Self::ACTIVE_POLLS, sender.canonical.as_slice(), polls)?;
        Ok(())
    }

    fn remove_active_poll(
        core: &mut C,
        sender: &Sender,
        poll_id: u64,
        timestamp: Moment,
    ) -> UsuallyOk {
        let mut active_polls = Self::active_polls(core, sender, timestamp)?;
        let position = active_polls.iter().position(|id| *id == poll_id).unwrap();
        active_polls.swap_remove(position);

        Self::set_active_polls(core, sender, &active_polls)?;
        Ok(())
    }

    fn create_poll(core: &mut C, sender: &Sender, poll: &Poll, now: Moment) -> UsuallyOk {
        poll.store(core)?;
        PollResult::new(core, poll.id).store(core)?;

        append_created_poll(core, &sender, poll.id, now)?;
        Ok(())
    }

    fn created_polls(core: &C, sender: &Sender, timestamp: Moment) -> StdResult<Vec<u64>> {
        let polls = core
            .get_ns::<Vec<u64>>(User::CREATED_POLLS, sender.canonical.as_slice())?
            .unwrap_or_default();
        Ok(filter_active_polls(core, polls, timestamp))
    }

    fn add_vote(
        core: &mut C,
        poll_id: u64,
        sender: &Sender,
        choice: VoteType,
        power: Uint128,
        now: Moment,
    ) -> UsuallyOk {
        if Vote::get(core, sender, poll_id).is_ok() {
            return Err(StdError::generic_err(
                "Already voted. Can't cast a vote for a second time. ",
            ));
        }
        Vote::new(core, choice, sender, power)?.store(core, sender, poll_id)?;

        append_active_poll(core, sender, poll_id, now)?;
        Poll::update_result(
            core,
            poll_id,
            now,
            UpdateResultReason::ChangeVotePower {
                choice,
                power_diff: power.u128() as i128,
            },
        )?;
        Ok(())
    }

    fn increase_vote_power(
        core: &mut C,
        poll_id: u64,
        sender: &Sender,
        power_diff: Uint128,
        now: Moment,
    ) -> UsuallyOk {
        Vote::increase(core, sender, poll_id, power_diff.u128()).unwrap();
        let vote = Vote::get(core, sender, poll_id)?;
        Poll::update_result(
            core,
            poll_id,
            now,
            UpdateResultReason::ChangeVotePower {
                choice: vote.choice,
                power_diff: power_diff.u128() as i128,
            },
        )
        .unwrap();
        Ok(())
    }

    fn change_choice(
        core: &mut C,
        poll_id: u64,
        sender: &Sender,
        choice: VoteType,
        now: Moment,
    ) -> UsuallyOk {
        let mut vote = Vote::get(core, sender, poll_id)?;

        if vote.choice == choice {
            return Err(StdError::generic_err(
                "Your vote is not changed. You tried to cast the same vote. ",
            ));
        };

        Poll::update_result(
            core,
            poll_id,
            now,
            UpdateResultReason::ChangeVoteChoice {
                choice,
                power: vote.power.u128(),
            },
        )
        .unwrap();

        vote.choice = choice;
        vote.store(core, sender, poll_id)?;

        Ok(())
    }

    fn remove_vote(core: &mut C, poll_id: u64, sender: &Sender, now: Moment) -> UsuallyOk {
        let vote = Vote::get(core, sender, poll_id)?;
        Vote::remove(core, sender, poll_id)?;
        User::remove_active_poll(core, sender, poll_id, now)?;
        Poll::update_result(
            core,
            poll_id,
            now,
            UpdateResultReason::ChangeVotePower {
                choice: vote.choice,
                power_diff: -(vote.power.u128() as i128),
            },
        )
        .unwrap();
        Ok(())
    }
}

fn filter_active_polls<S, A, Q, C>(core: &C, polls: Vec<u64>, timestamp: Moment) -> Vec<u64>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    polls
        .into_iter()
        .filter(|id| {
            let expiration = Poll::expiration(core, *id).unwrap();
            !expiration.is_expired(timestamp)
        })
        .collect()
}

fn append_active_poll<S, A, Q, C>(
    core: &mut C,
    sender: &Sender,
    poll_id: u64,
    now: Moment,
) -> UsuallyOk
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    let mut active_polls = User::active_polls(core, sender, now)?;
    active_polls.push(poll_id);
    User::set_active_polls(core, sender, &active_polls)?;
    Ok(())
}

fn append_created_poll<S, A, Q, C>(
    core: &mut C,
    sender: &Sender,
    poll_id: u64,
    now: Moment,
) -> UsuallyOk
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    let mut polls = User::created_polls(core, sender, now)?;
    polls.push(poll_id);
    core.set_ns(User::CREATED_POLLS, sender.canonical.as_slice(), polls)?;
    Ok(())
}
