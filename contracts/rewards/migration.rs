//! This module supports two methods of migration.
//!
//! * One is by automatically emitting the messages from both contract
//!   that a manual migration would involve.
//!
//! * The other is by emitting a snapshot from the sender contract
//!   that is imported by the receiver contract.
//!
//! Sienna Rewards currently uses the former method, so as not to
//! reimplement pieces of the liquidity accumulation logic twice.

use crate::auth::Auth;
use fadroma::*;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum EmigrationHandle {
    /// Allow another contract to receive data from this contract
    EnableMigrationTo(ContractLink<HumanAddr>),
    /// Disallow another contract to receive data from this contract
    DisableMigrationTo(ContractLink<HumanAddr>),
    /// Export migration data to another contract. Must be called by a contract
    /// migration to which was enabled via `EnableMigrationTo`, and pass an address
    /// for which the migration is to be performed.
    ExportState(HumanAddr),
}

pub trait Emigration<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    const CAN_MIGRATE_TO: &'static [u8] = b"/migration/prev";

    fn handle (&mut self, env: Env, msg: EmigrationHandle) -> StdResult<HandleResponse> {
        match msg {
            EmigrationHandle::ExportState(migrant) =>
                self.handle_export_state(&env, &migrant),
            _ => {
                Auth::assert_admin(self, &env)?;
                match msg {
                    EmigrationHandle::EnableMigrationTo(contract) =>
                        self.handle_enable_migration_to(env, contract),
                    EmigrationHandle::DisableMigrationTo(contract) =>
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

    fn can_export_state (&mut self, env: &Env, migrant: &HumanAddr)
        -> StdResult<ContractLink<HumanAddr>>
    {
        // The ExportState transaction cannot be called manually by the user;
        // it must be called by the contract which is receiving the migration
        if &env.message.sender == migrant {
            return Err(StdError::generic_err("This handler must be called as part of a transaction"))
        }
        // If migration to the caller contract is enabled,
        // its code hash should be available in storage
        let id = self.canonize(env.message.sender.clone())?;
        let receiver_link: Option<ContractLink<HumanAddr>> =
            self.get_ns(Self::CAN_MIGRATE_TO, id.as_slice())?;
        if let Some(receiver_link) = receiver_link {
            Ok(receiver_link)
        } else {
            Err(StdError::generic_err("Migration to this target is not enabled."))
        }
    }

    /// Override this to emit the corresponding messages, if migrating via transactions.
    /// Make sure to keep can_export_state call in the override.
    fn handle_export_state (&mut self, env: &Env, migrant: &HumanAddr) -> StdResult<HandleResponse> {
        let receiver = self.can_export_state(env, migrant)?;
        let response = HandleResponse::default();
        if let Some(snapshot) = self.export_state(env, migrant)? {
            let msg = ImmigrationHandle::ReceiveMigration(snapshot);
            response.msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      receiver.address,
                callback_code_hash: receiver.code_hash,
                send:               vec![],
                msg: to_binary(&msg)?,
            }))
        } else {
            Ok(response)
        }
    }

    /// Override this to return a serialized version of your migration snapshot object
    /// if you are migrating via snapshots.
    fn export_state (&mut self, _env: &Env, _migrant: &HumanAddr) -> StdResult<Option<Binary>> {
        Ok(None)
    }
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum ImmigrationHandle {
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

pub trait Immigration<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q>
    + Auth<S, A, Q>
{
    const CAN_MIGRATE_FROM: &'static [u8] = b"/migration/next";

    fn handle (&mut self, env: Env, msg: ImmigrationHandle) -> StdResult<HandleResponse> {
        match msg {
            ImmigrationHandle::RequestMigration(last_contract) =>
                self.handle_request_migration(env, last_contract),
            ImmigrationHandle::ReceiveMigration(data) =>
                self.handle_receive_migration(env, data),
            _ => {
                match msg {
                    ImmigrationHandle::EnableMigrationFrom(contract) =>
                        self.handle_enable_migration_from(env, contract),
                    ImmigrationHandle::DisableMigrationFrom(contract) =>
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

    fn handle_request_migration (&mut self, env: Env, prev: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        HandleResponse::default()
            .msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      prev.address,
                callback_code_hash: prev.code_hash,
                send:               vec![],
                msg: to_binary(&EmigrationHandle::ExportState(env.message.sender))?,
            }))
    }

    /// Override this to emit the corresponding messages, if migrating via transactions
    fn handle_receive_migration (&mut self, env: Env, data: Binary) -> StdResult<HandleResponse> {
        self.import_state(env, data)?;
        Ok(HandleResponse::default())
    }

    /// Override this to import a snapshot, if migrating via snapshots
    fn import_state (&mut self, env: Env, data: Binary) -> StdResult<()> {
        unimplemented!()
    }
}
