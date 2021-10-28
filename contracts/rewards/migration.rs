use crate::auth::Auth;
use fadroma::*;
use fadroma::messages;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationHandle {
    InitiateMigration {
        next_contract: ContractLink<HumanAddr>
    },
    MigrationData {},
    MigrateFrom {
        contract: ContractLink<HumanAddr>
    },
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationQuery {}

pub trait Migration<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    fn handle (&mut self, env: Env, msg: MigrationHandle) -> StdResult<HandleResponse> {
        match msg {
            MigrationHandle::MigrateTo { contract } =>
                self.handle_migrate_to(env, contract),
            MigrationHandle::MigrateFrom { contract } =>
                self.handle_migrate_from(env, contract)
        }
    }

    fn handle_migrate_to (&mut self, env: Env, contract: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn handle_migrate_from (&mut self, env: Env, contract: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn query (&self, msg: MigrationQuery) -> StdResult<Binary> {
        Err(StdError::generic_err("not implemented"))
    }
}
