use fadroma::{Api, HumanAddr, Querier, StdResult, Storage};

use super::governance::Governance;

pub struct User(Vec<u64>);

impl User {
    pub const ACTIVE_POLLS: &'static [u8] = b"/gov/user/polls";
}

pub trait IUser<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn get_active_polls(core: &C, address: HumanAddr) -> StdResult<Vec<u64>>;
}

impl<S, A, Q, C> IUser<S, A, Q, C> for User
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn get_active_polls(_core: &C, _address: HumanAddr) -> StdResult<Vec<u64>> {
        Ok(vec![])
    }
}
