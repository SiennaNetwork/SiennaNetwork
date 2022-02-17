use fadroma::{Api, Composable, HumanAddr, Querier, StdError, StdResult, Storage, ViewingKey};
use serde::{Deserialize, Serialize};

use crate::time_utils::Moment;

use super::poll::{IPoll, Poll};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct User {
    active_polls: Vec<u64>,
}

impl User {
    pub const ACTIVE_POLLS: &'static [u8] = b"/gov/user/polls";
    pub const VIEWING_KEY: &'static [u8] = b"gov/user/key";
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
    fn get_active_polls(core: &C, address: HumanAddr, timestamp: Moment) -> StdResult<Vec<u64>>;
    fn append_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()>;
    fn set_active_polls(core: &mut C, address: HumanAddr, polls: Vec<u64>) -> StdResult<()>;
    fn remove_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()>;

    fn get(core: &C, address: HumanAddr) -> StdResult<User>;
    fn set_vk(core: &mut C, address: HumanAddr, key: &ViewingKey) -> StdResult<()>;
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
    fn get_active_polls(core: &C, address: HumanAddr, timestamp: Moment) -> StdResult<Vec<u64>> {
        let user = Self::get(core, address)?;

        let filtered: Vec<u64> = user
            .active_polls
            .iter()
            .map(|id| *id)
            .filter(|id| {
                //no invalid id's should end up in the vector, this cannot fail, if it does
                //an appropriate message is returned from the expiration function
                let expiration = Poll::expiration(core, *id).unwrap();

                !expiration.is_expired(timestamp)
            })
            .collect();

        Ok(filtered)
    }

    /**
    Adds the id to the saved vector
    */
    fn append_active_poll(
        core: &mut C,
        address: HumanAddr,
        poll_id: u64,
        timestamp: Moment,
    ) -> StdResult<()> {
        let mut active_polls = Self::get_active_polls(core, address.clone(), timestamp)?;
        active_polls.push(poll_id);
        Self::set_active_polls(core, address, active_polls)?;
        Ok(())
    }
    fn store(&self, core: &mut C, address: HumanAddr) -> StdResult<()> {
        let address = core.canonize(address)?;

        //for now only active polls are saved
        core.set_ns(Self::ACTIVE_POLLS, address.as_slice(), &self.active_polls)?;

        Ok(())
    }
    fn get(core: &C, address: HumanAddr) -> StdResult<User> {
        let address = core.canonize(address)?;

        let active_polls = core
            .get_ns::<Vec<u64>>(Self::ACTIVE_POLLS, address.as_slice())?
            .unwrap_or_default();

        Ok(Self { active_polls })
    }

    /**
    Overwrites the saved active polls for given user
    */
    fn set_active_polls(core: &mut C, address: HumanAddr, polls: Vec<u64>) -> StdResult<()> {
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
        let active_polls = Self::get_active_polls(core, address.clone(), timestamp)?;
        let active_polls = active_polls
            .iter()
            .map(|id| *id)
            .filter(|id| *id != poll_id)
            .collect();
        Self::set_active_polls(core, address, active_polls)?;
        Ok(())
    }

    fn set_vk(core: &mut C, address: HumanAddr, key: &ViewingKey) -> StdResult<()> {
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
