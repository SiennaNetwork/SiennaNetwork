pub use fadroma::*;
pub use fadroma::messages;

use crate::core::Composable;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationHandle {
    MigrateTo   { contract: ContractLink<HumanAddr> },
    MigrateFrom { contract: ContractLink<HumanAddr> },
    ReleaseSnip20 {
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    },
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationQuery {}

pub trait Migration<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q> {
    fn handle (&mut self, env: Env, msg: MigrationHandle) -> StdResult<HandleResponse> {
        Err(StdError::generic_err("not implemented"))
    }
    fn query  (&self, msg: MigrationQuery) -> StdResult<Binary> {
        Err(StdError::generic_err("not implemented"))
    }
}
