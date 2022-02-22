use fadroma::{
    Api, Composable, HumanAddr, Querier, StdError, StdResult, Storage, Uint128, ViewingKey,
};
use serde::{Deserialize, Serialize};

use crate::time_utils::Moment;

use super::{
    poll::{IPoll, Poll, UpdateResultReason},
    vote::{IVote, Vote, VoteType},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    pub active_polls: Vec<u64>,
    created_polls: Vec<u64>,
}

impl User {
    pub const ACTIVE_POLLS: &'static [u8] = b"/gov/user/polls";
    pub const VIEWING_KEY: &'static [u8] = b"gov/user/key";
    pub const CREATED_POLLS: &'static [u8] = b"gov/user/created_polls";

    pub fn can_unstake(&self, balance: u128, threshold: u128, amount: u128) -> bool {
        if self.active_polls.len() > 0 {
            return false;
        }
        if self.created_polls.len() > 0 {
            return balance - amount > threshold;
        }
        return true;
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
    fn get(core: &C, address: HumanAddr, now: Moment) -> StdResult<User>;
    fn store(&self, core: &mut C, address: HumanAddr) -> StdResult<()>;

    fn active_polls(core: &C, address: HumanAddr, now: Moment) -> StdResult<Vec<u64>>;
    fn set_active_polls(core: &mut C, address: HumanAddr, polls: &Vec<u64>) -> StdResult<()>;
    fn remove_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()>;

    fn create_poll(core: &mut C, sender: HumanAddr, poll: &Poll, now: Moment) -> StdResult<()>;
    fn created_polls(core: &C, address: HumanAddr, now: Moment) -> StdResult<Vec<u64>>;

    fn set_viewing_key(core: &mut C, address: HumanAddr, key: &ViewingKey) -> StdResult<()>;
    fn viewing_key(core: &C, address: HumanAddr) -> StdResult<Option<ViewingKey>>;
    fn check_viewing_key(core: &C, address: HumanAddr, provided_vk: &ViewingKey) -> StdResult<()>;
    fn add_vote(
        core: &mut C,
        poll_id: u64,
        sender: HumanAddr,
        choice: VoteType,
        power: Uint128,
        now: Moment,
    ) -> StdResult<()>;
    fn change_choice(
        core: &mut C,
        poll_id: u64,
        sender: HumanAddr,
        choice: VoteType,
        now: Moment,
    ) -> StdResult<()>;
    fn increase_vote_power(
        core: &mut C,
        poll_id: u64,
        sender: HumanAddr,
        power_diff: Uint128,
        now: Moment,
    ) -> StdResult<()>;
    fn remove_vote(core: &mut C, poll_id: u64, sender: HumanAddr, now: Moment) -> StdResult<()>;
}

impl<S, A, Q, C> IUser<S, A, Q, C> for User
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    fn get(core: &C, address: HumanAddr, now: Moment) -> StdResult<User> {
        let active_polls = User::active_polls(core, address.clone(), now)?;
        let created_polls = User::created_polls(core, address, now)?;
        Ok(Self {
            active_polls,
            created_polls,
        })
    }

    fn active_polls(core: &C, address: HumanAddr, timestamp: Moment) -> StdResult<Vec<u64>> {
        let canonized_address = core.canonize(address.clone())?;
        let polls = core
            .get_ns::<Vec<u64>>(Self::ACTIVE_POLLS, canonized_address.as_slice())?
            .unwrap_or_default();
        Ok(filter_active_polls(core, polls, timestamp))
    }

    fn store(&self, core: &mut C, address: HumanAddr) -> StdResult<()> {
        let canonized_address = core.canonize(address.clone())?;
        core.set_ns(
            Self::CREATED_POLLS,
            canonized_address.as_slice(),
            &self.created_polls,
        )?;
        User::set_active_polls(core, address.clone(), &self.active_polls)?;
        Ok(())
    }

    /**
    Overwrites the saved active polls for given user
    */
    fn set_active_polls(core: &mut C, address: HumanAddr, polls: &Vec<u64>) -> StdResult<()> {
        let address = core.canonize(address)?;
        core.set_ns(Self::ACTIVE_POLLS, address.as_slice(), polls)?;

        Ok(())
    }

    fn remove_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()> {
        let active_polls = Self::active_polls(core, address.clone(), timestamp)?;
        let active_polls = active_polls
            .into_iter()
            .filter(|id| *id != poll_id)
            .collect();
        Self::set_active_polls(core, address, &active_polls)?;
        Ok(())
    }

    fn create_poll(core: &mut C, sender: HumanAddr, poll: &Poll, now: Moment) -> StdResult<()> {
        poll.store(core)?;
        append_created_poll(core, sender, poll.id, now)?;
        Ok(())
    }

    fn created_polls(core: &C, address: HumanAddr, timestamp: Moment) -> StdResult<Vec<u64>> {
        let canonized_adr = core.canonize(address.clone())?;
        let polls = core
            .get_ns::<Vec<u64>>(User::CREATED_POLLS, canonized_adr.as_slice())?
            .unwrap_or_default();
        Ok(filter_active_polls(core, polls, timestamp))
    }

    fn set_viewing_key(core: &mut C, address: HumanAddr, key: &ViewingKey) -> StdResult<()> {
        let id = core.canonize(address)?;
        core.set_ns(Self::VIEWING_KEY, id.as_slice(), key)?;
        Ok(())
    }
    fn viewing_key(core: &C, address: HumanAddr) -> StdResult<Option<ViewingKey>> {
        let id = core.canonize(address)?;
        core.get_ns(Self::VIEWING_KEY, id.as_slice())
    }

    fn check_viewing_key(core: &C, address: HumanAddr, provided_vk: &ViewingKey) -> StdResult<()> {
        let stored_vk = Self::viewing_key(core, address)?;
        if let Some(ref key) = stored_vk {
            if provided_vk.check_viewing_key(&key.to_hashed()) {
                Ok(())
            } else {
                Err(StdError::unauthorized())
            }
        } else {
            Err(StdError::unauthorized())
        }
    }

    fn add_vote(
        core: &mut C,
        poll_id: u64,
        sender: HumanAddr,
        choice: VoteType,
        power: Uint128,
        now: Moment,
    ) -> StdResult<()> {
        if let Ok(_) = Vote::get(core, sender.clone(), poll_id) {
            return Err(StdError::generic_err(
                "Already voted. Can't cast a vote for a second time. ",
            ));
        }
        Vote::new(core, choice.clone(), sender.clone(), power)?.store(
            core,
            sender.clone(),
            poll_id,
        )?;

        append_active_poll(core, sender.clone(), poll_id, now)?;
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
        sender: HumanAddr,
        power_diff: Uint128,
        now: Moment,
    ) -> StdResult<()> {
        Vote::increase(core, sender.clone(), poll_id, power_diff.u128())
            .expect("Failed to increase vote");
        let vote = Vote::get(core, sender.clone(), poll_id)?;
        Poll::update_result(
            core,
            poll_id,
            now,
            UpdateResultReason::ChangeVotePower {
                choice: vote.choice,
                power_diff: power_diff.u128() as i128,
            },
        )
        .expect("Failed to update poll results");
        Ok(())
    }

    fn change_choice(
        core: &mut C,
        poll_id: u64,
        sender: HumanAddr,
        choice: VoteType,
        now: Moment,
    ) -> StdResult<()> {
        let mut vote = Vote::get(core, sender.clone(), poll_id)?;
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
        .expect("Failed to update poll results");

        vote.choice = choice;
        vote.store(core, sender.clone(), poll_id)?;

        Ok(())
    }

    fn remove_vote(core: &mut C, poll_id: u64, sender: HumanAddr, now: Moment) -> StdResult<()> {
        let vote = Vote::get(core, sender.clone(), poll_id)?;
        Vote::remove(core, sender.clone(), poll_id)?;
        User::remove_active_poll(core, sender.clone(), poll_id, now)?;
        Poll::update_result(
            core,
            poll_id,
            now,
            UpdateResultReason::ChangeVotePower {
                choice: vote.choice,
                power_diff: -(vote.power.u128() as i128),
            },
        )
        .expect("Failed to update poll results");
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
        .iter()
        .copied()
        .filter(|id| {
            let expiration = Poll::expiration(core, *id).unwrap();
            !expiration.is_expired(timestamp)
        })
        .collect()
}

fn append_active_poll<S, A, Q, C>(
    core: &mut C,
    address: HumanAddr,
    poll_id: u64,
    now: Moment,
) -> StdResult<()>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    let mut active_polls = User::active_polls(core, address.clone(), now)?;
    active_polls.push(poll_id);
    User::set_active_polls(core, address, &active_polls)?;
    Ok(())
}

fn append_created_poll<S, A, Q, C>(
    core: &mut C,
    address: HumanAddr,
    poll_id: u64,
    now: Moment,
) -> StdResult<()>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
{
    let canonized_address = core.canonize(address.clone())?;
    let mut polls = User::created_polls(core, address, now)?;
    polls.push(poll_id);
    core.set_ns(User::CREATED_POLLS, canonized_address.as_slice(), polls)?;
    Ok(())
}
