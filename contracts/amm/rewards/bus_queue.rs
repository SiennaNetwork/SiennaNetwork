use fadroma::{StdResult, Storage, Api, Querier, Composable};

pub trait BusQueue<S: Storage, A: Api, Q: Querier> : Composable<S, A, Q> {
    fn init_bindings(&mut self);
    fn bind<Core, Fin, Fout: Default>(&mut self, msg: &str, func: fn(Core, Fin)-> Fout);
    fn broadcast<Fin, Fout: Default>(&self, msg: &str, arg: Fin) -> StdResult<Fout>;
}

