use crate::auth::Auth;
use fadroma::*;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationHandle {
    EnableMigration(ContractLink<HumanAddr>),
    DisableMigration,
    ExportState(HumanAddr),
    ImportState(ContractLink<HumanAddr>),
    ReceiveState(Binary),
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationQuery {
    MigrationStatus { next_contract: Option<ContractLink<HumanAddr>> }
}

pub const NEXT_VERSION: &[u8] = b"/migration/next";
pub const LAST_VERSION: &[u8] = b"/migration/prev";

pub trait Migration<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    fn handle (&mut self, env: Env, msg: MigrationHandle) -> StdResult<HandleResponse> {
        match msg {
            MigrationHandle::EnableMigration(next_contract) =>
                self.handle_enable_migration(env, next_contract),
            MigrationHandle::DisableMigration =>
                self.handle_disable_migration(env),
            MigrationHandle::ExportState(initiator) =>
                self.handle_export_state(env, initiator),
            MigrationHandle::ImportState(last_contract) =>
                self.handle_import_state(env, last_contract),
            MigrationHandle::ReceiveState(data) =>
                self.handle_receive_state(env, data)
        }
    }

    fn handle_enable_migration (&mut self, _env: Env, next: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        self.set(NEXT_VERSION, Some(next))?;
        Ok(HandleResponse::default())
    }

    fn handle_disable_migration (&mut self, _env: Env) -> StdResult<HandleResponse> {
        self.set::<Option<ContractLink<HumanAddr>>>(NEXT_VERSION, None)?;
        Ok(HandleResponse::default())
    }

    fn handle_export_state (&mut self, _env: Env, _initiator: HumanAddr) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn export_state (&mut self, env: Env, initiator: HumanAddr) -> StdResult<Binary>;

    fn handle_import_state (&mut self, env: Env, prev: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        HandleResponse::default()
            .msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      prev.address,
                callback_code_hash: prev.code_hash,
                send:               vec![],
                msg: to_binary(&MigrationHandle::ExportState(env.message.sender))?,
            }))
    }

    fn import_state (&mut self, env: Env, data: Binary) -> StdResult<()>;

    fn handle_receive_state (&mut self, _env: Env, _data: Binary) -> StdResult<HandleResponse> {
        Ok(HandleResponse::default())
    }

    fn query (&self, _msg: MigrationQuery) -> StdResult<Binary> {
        Err(StdError::generic_err("not implemented"))
    }
}
