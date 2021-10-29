use crate::auth::Auth;
use fadroma::*;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationHandle {
    StartMigration { next_contract: ContractLink<HumanAddr> },
    StopMigration  {},
    ExportState    { initiator: HumanAddr },
    ImportState    { prev_contract: ContractLink<HumanAddr> },
    ReceiveState   { data: Binary },
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationQuery {
    MigrationStatus { next_contract: Option<ContractLink<HumanAddr>> }
}

pub const NEXT: &[u8] = b"/migration/next";
pub const PREV: &[u8] = b"/migration/prev";

pub trait Migration<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    fn handle (&mut self, env: Env, msg: MigrationHandle) -> StdResult<HandleResponse> {
        match msg {
            MigrationHandle::StartMigration { next_contract } =>
                self.handle_start_migration(env, next_contract),
            MigrationHandle::StopMigration {} =>
                self.handle_stop_migration(env),
            MigrationHandle::ExportState { initiator } =>
                self.handle_export_state(env, initiator),
            MigrationHandle::ImportState { prev_contract } =>
                self.handle_import_state(env, prev_contract),
            MigrationHandle::ReceiveState { data } =>
                self.handle_receive_state(env, data)
        }
    }

    fn handle_start_migration (&mut self, env: Env, next: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        self.set(NEXT, Some(next))?;
        Ok(HandleResponse::default())
    }

    fn handle_stop_migration (&mut self, env: Env) -> StdResult<HandleResponse> {
        self.set::<Option<ContractLink<HumanAddr>>>(NEXT, None)?;
        Ok(HandleResponse::default())
    }

    fn handle_export_state (&mut self, env: Env, initiator: HumanAddr) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn handle_import_state (&mut self, env: Env, prev: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        HandleResponse::default()
            .msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      prev.address,
                callback_code_hash: prev.code_hash,
                send:               vec![],
                msg: to_binary(&MigrationHandle::ExportState { initiator: env.message.sender })?,
            }))
    }

    fn handle_receive_state (&mut self, env: Env, data: Binary) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn query (&self, msg: MigrationQuery) -> StdResult<Binary> {
        Err(StdError::generic_err("not implemented"))
    }
}
