use crate::{Auth, *};
use fadroma::*;

pub trait KeplrCompat<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q> + Auth<S, A, Q> {
    fn token_info(&self) -> StdResult<Response>;
    fn balance(&self, address: HumanAddr, key: ViewingKey) -> StdResult<Response>;
}
