use fadroma::{Api, Composable, HumanAddr, Querier, StdError, StdResult, Storage, ViewingKey};
use serde::{Deserialize, Serialize};

use crate::time_utils::Moment;

use super::poll::{IPoll, Poll};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    active_polls: Vec<u64>,
    created_polls: Vec<u64>
}

impl User {
    pub const ACTIVE_POLLS: &'static [u8] = b"/gov/user/polls";
    pub const VIEWING_KEY: &'static [u8] = b"gov/user/key";
    pub const CREATED_POLLS: &'static [u8] = b"gov/user/created_polls";
}

pub trait IUser<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Composable<S, A, Q>,
    Self: Sized,
{
    fn store(&self, core: &mut C, address: HumanAddr) -> StdResult<()>;
    fn active_polls(core: &C, address: HumanAddr, now: Moment) -> StdResult<Vec<u64>>;
    fn append_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()>;
    fn set_active_polls(core: &mut C, address: HumanAddr, polls: &Vec<u64>) -> StdResult<()>;
    fn remove_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()>;

    fn created_polls(core: &C, address: HumanAddr, now: Moment) -> StdResult<Vec<u64>>;
    fn append_created_poll(core: &mut C, address: HumanAddr, poll_id: u64, now: Moment) -> StdResult<()>; 
    fn get(core: &C, address: HumanAddr, now: Moment) -> StdResult<User>;
    fn set_viewing_key(core: &mut C, address: HumanAddr, key: &ViewingKey) -> StdResult<()>;
    fn viewing_key(core: &C, address: HumanAddr) -> StdResult<Option<ViewingKey>>;
    fn check_viewing_key(core: &C, address: HumanAddr, provided_vk: &ViewingKey) -> StdResult<()>;
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
        let created_polls= User::created_polls(core, address, now)?;
        Ok(Self { active_polls , created_polls})
    }

    fn active_polls(core: &C, address: HumanAddr, timestamp: Moment) -> StdResult<Vec<u64>> {
        let canonized_address = core.canonize(address.clone())?;
        let polls = core
            .get_ns::<Vec<u64>>(Self::ACTIVE_POLLS, canonized_address.as_slice())?
            .unwrap_or_default();
        Ok(filter_active_polls(core, polls, timestamp))
    }

    fn append_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()> {
        let mut active_polls = Self::active_polls(core, address.clone(), timestamp)?;
        active_polls.push(poll_id);
        Self::set_active_polls(core, address, &active_polls)?;
        Ok(())
    }
    fn store(&self, core: &mut C, address: HumanAddr) -> StdResult<()> {
        let canonized_address = core.canonize(address.clone())?;
        core.set_ns(Self::CREATED_POLLS, canonized_address.as_slice(), &self.created_polls)?;
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
    
    fn created_polls(core: &C, address: HumanAddr, timestamp: Moment) -> StdResult<Vec<u64>> {
        let canonized_adr = core.canonize(address.clone())?;
        let polls = core
            .get_ns::<Vec<u64>>(User::CREATED_POLLS, canonized_adr.as_slice())?
            .unwrap_or_default();
        Ok(filter_active_polls(core, polls, timestamp))
    }

    fn append_created_poll(core: &mut C, address: HumanAddr, poll_id: u64, now: Moment) -> StdResult<()> {
        let canonized_address = core.canonize(address.clone())?;
        let mut polls = User::created_polls(core, address, now)?;
        polls.push(poll_id);
        core.set_ns(User::CREATED_POLLS, canonized_address.as_slice(), polls)?;
        Ok(())
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
}

fn filter_active_polls<S, A, Q, C> (core: & C, polls: Vec<u64>, timestamp: Moment) -> Vec<u64> 
where
S: Storage,
A: Api,
Q: Querier,
C: Composable<S, A, Q>
{
    polls.iter()
    .copied()
    .filter(|id| {
        let expiration = Poll::expiration(core, *id).unwrap();
        !expiration.is_expired(timestamp)
    })
    .collect()
}