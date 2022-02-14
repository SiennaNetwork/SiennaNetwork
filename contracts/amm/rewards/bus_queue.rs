use fadroma::{StdResult, Storage, Api, Querier, Composable};

pub trait BusQueue<S: Storage, A: Api, Q: Querier> : Composable<S, A, Q> {
    fn init_bindings(&mut self);
    fn bind<T>(&mut self, msg: &str, func: fn() -> T) where T: Default;
    fn broadcast<T>(msg: &str) -> StdResult<T> where T: Default;
}

