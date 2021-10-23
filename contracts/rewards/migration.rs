pub use fadroma::scrt::{
    contract::{message, messages},
    cosmwasm_std::*,
}

use crate::core::Composable;

messages!(MigrationHandleMsg {
    MigrateTo   { contract: ContractLink<HumanAddr> },
    MigrateFrom { contract: ContractLink<HumanAddr> }
});

pub trait Migration<S, A, Q>: Composable<S, A, Q> {
    fn handle (&mut self, env: &Env, msg: &Handle) -> StdResult<Option<HandleResponse>>;
    fn query  (&self, msg: &Query) -> StdResult<Option<Binary>>;
}
