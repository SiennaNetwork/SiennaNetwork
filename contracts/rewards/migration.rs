use crate::auth::Auth;
use fadroma::*;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationExportHandle {
    /// Allow another contract to receive data from this contract
    EnableMigrationTo(ContractLink<HumanAddr>),
    /// Disallow another contract to receive data from this contract
    DisableMigrationTo(ContractLink<HumanAddr>),
    /// Export migration data to another contract. Must be called by a contract
    /// migration to which was enabled via `EnableMigrationTo`, and pass an address
    /// for which the migration is to be performed.
    ExportState(HumanAddr),
}

pub trait MigrationExport<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    const CAN_MIGRATE_TO: &'static [u8] = b"/migration/prev";

    fn handle (&mut self, env: Env, msg: MigrationExportHandle) -> StdResult<HandleResponse> {
        match msg {
            MigrationExportHandle::ExportState(initiator) =>
                self.handle_export_state(env, initiator),
            _ => {
                Auth::assert_admin(self, &env)?;
                match msg {
                    MigrationExportHandle::EnableMigrationTo(contract) =>
                        self.handle_enable_migration_to(env, contract),
                    MigrationExportHandle::DisableMigrationTo(contract) =>
                        self.handle_disable_migration_to(env, contract),
                    _ => unreachable!()
                }
            }
        }
    }

    fn handle_enable_migration_to (&mut self, env: Env, next: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        Auth::assert_admin(self, &env)?;
        let id = self.canonize(next.address.clone())?;
        self.set_ns(Self::CAN_MIGRATE_TO, id.as_slice(), Some(next))?;
        Ok(HandleResponse::default())
    }

    fn handle_disable_migration_to (&mut self, env: Env, next: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        Auth::assert_admin(self, &env)?;
        let id = self.canonize(next.address)?;
        self.set_ns::<Option<ContractLink<HumanAddr>>>(Self::CAN_MIGRATE_TO, id.as_slice(), None)?;
        Ok(HandleResponse::default())
    }

    fn handle_export_state (&mut self, env: Env, initiator: HumanAddr) -> StdResult<HandleResponse> {
        // This makes no sense to be called manually by the user;
        // it must be called by the contract which is receiving the migration
        let contract_addr = env.message.sender;
        if contract_addr == initiator {
            return Err(StdError::generic_err("This handler must be called as part of a transaction"))
        }
        // If migration to the caller contract is enabled,
        // its code hash should be available in storage
        let id = self.canonize(contract_addr.clone())?;
        let link: Option<ContractLink<HumanAddr>> = self.get_ns(Self::CAN_MIGRATE_TO, id.as_slice())?;
        if let Some(link) = link {
            Ok(HandleResponse::default().msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      link.address,
                callback_code_hash: link.code_hash,
                send:               vec![],
                msg: to_binary(&MigrationExportHandle::ExportState(contract_addr))?,
            }))?)
        } else {
            return Err(StdError::generic_err("Migration to this contract is not enabled."))
        }
    }

    /// Implement this to return a serialized version of your migration snapshot object.
    fn export_state (&mut self, env: Env, initiator: HumanAddr) -> StdResult<Binary>;
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum MigrationImportHandle {
    /// Allow this contract to receive data from another contract
    EnableMigrationFrom(ContractLink<HumanAddr>),
    /// Disallow this contract to receive data from another contract
    DisableMigrationFrom(ContractLink<HumanAddr>),
    /// Request migration data from another contract. Called by the user to initiate a migration.
    RequestMigration(ContractLink<HumanAddr>),
    /// Callback containing migration data. Must be called by a contract
    /// migration from which was enabled via `EnableMigrationFrom`.
    ReceiveMigration(Binary),
}


pub trait MigrationImport<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    const CAN_MIGRATE_FROM: &'static [u8] = b"/migration/next";

    fn handle (&mut self, env: Env, msg: MigrationImportHandle) -> StdResult<HandleResponse> {
        match msg {
            MigrationImportHandle::RequestMigration(last_contract) =>
                self.handle_request_migration(env, last_contract),
            MigrationImportHandle::ReceiveMigration(data) =>
                self.handle_receive_migration(env, data),
            _ => {
                match msg {
                    MigrationImportHandle::EnableMigrationFrom(contract) =>
                        self.handle_enable_migration_from(env, contract),
                    MigrationImportHandle::DisableMigrationFrom(contract) =>
                        self.handle_disable_migration_from(env, contract),
                    _ => unreachable!()
                }
            }
        }
    }

    fn handle_enable_migration_from (&mut self, env: Env, last: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        Auth::assert_admin(self, &env)?;
        let id = self.canonize(last.address.clone())?;
        self.set_ns(Self::CAN_MIGRATE_FROM, id.as_slice(), Some(last))?;
        Ok(HandleResponse::default())
    }

    fn handle_disable_migration_from (&mut self, env: Env, last: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        Auth::assert_admin(self, &env)?;
        let id = self.canonize(last.address)?;
        self.set_ns::<Option<ContractLink<HumanAddr>>>(Self::CAN_MIGRATE_FROM, id.as_slice(), None)?;
        Ok(HandleResponse::default())
    }

    fn handle_request_migration (&mut self, env: Env, prev: ContractLink<HumanAddr>) -> StdResult<HandleResponse> {
        HandleResponse::default()
            .msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      prev.address,
                callback_code_hash: prev.code_hash,
                send:               vec![],
                msg: to_binary(&MigrationExportHandle::ExportState(env.message.sender))?,
            }))
    }

    fn handle_receive_migration (&mut self, env: Env, data: Binary) -> StdResult<HandleResponse> {
        self.import_state(env, data)?;
        Ok(HandleResponse::default())
    }

    /// Implement this to deserialize and store a migration snapshot
    fn import_state (&mut self, env: Env, data: Binary) -> StdResult<()>;
}
