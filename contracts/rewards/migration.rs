use crate::auth::Auth;
use fadroma::*;
use fadroma::messages;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationHandle {
    InitiateMigration {
        next_contract: ContractLink<HumanAddr>
    },
    MigrateAway {},
    MigrateFrom {
        prev_contract: ContractLink<HumanAddr>
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
            MigrationHandle::InitiateMigration { next_contract } =>
                self.handle_initiate_migration(env, next_contract),
            MigrationHandle::MigrateAway {} =>
                self.handle_migrate_away(env),
            MigrationHandle::MigrateFrom { prev_contract } =>
                self.handle_migrate_from(env, prev_contract)
        }
    }

    fn handle_initiate_migration (&mut self, env: Env, next_contract: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn handle_migrate_away (&mut self, env: Env) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn handle_migrate_from (&mut self, env: Env, prev_contract: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn query (&self, msg: MigrationQuery) -> StdResult<Binary> {
        Err(StdError::generic_err("not implemented"))
    }
}
