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

use crate::{auth::Auth, errors};
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
            EmigrationHandle::ExportState(migrant) => {
                let next_contract = self.can_export_state(&env, &migrant)?;
                self.handle_export_state(env, next_contract, migrant)
            }
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

    /// Allow another contract to receive migrations from this contract
    fn handle_enable_migration_to (&mut self, env: Env, next: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        Auth::assert_admin(self, &env)?;
        let id = self.canonize(next.address.clone())?;
        self.set_ns(Self::CAN_MIGRATE_TO, id.as_slice(), Some(next))?;
        Ok(HandleResponse::default())
    }

    /// Stop allowing another contract from receiving migrations from this contract
    fn handle_disable_migration_to (&mut self, env: Env, next: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        Auth::assert_admin(self, &env)?;
        let id = self.canonize(next.address)?;
        self.set_ns::<Option<ContractLink<HumanAddr>>>(Self::CAN_MIGRATE_TO, id.as_slice(), None)?;
        Ok(HandleResponse::default())
    }

    /// Implement this to emit the corresponding messages, if migrating via transactions.
    fn handle_export_state (
        &mut self,
        _env:           Env,
        _next_contract: ContractLink<HumanAddr>,
        _migrant:       HumanAddr
    ) -> StdResult<HandleResponse> {
        unimplemented!()
    }/* {
        let response = HandleResponse::default();
        if let Some(snapshot) = self.export_state(env, migrant)? {
            let msg = self.wrap_receive_msg(ImmigrationHandle::ReceiveMigration(snapshot))?;
            response.msg(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr:      receiver.address,
                callback_code_hash: receiver.code_hash,
                send: vec![],
                msg,
            }))
        } else {
            Ok(response)
        }
    }*/

    /// Check if the `ExportState` call is allowed.
    fn can_export_state (&mut self, env: &Env, migrant: &HumanAddr)
        -> StdResult<ContractLink<HumanAddr>>
    {
        // The ExportState transaction is not meat to be called manually by the user;
        // it must be called by the contract which is receiving the migration
        if &env.message.sender == migrant {
            return errors::export_state_miscalled()
        }
        // If migration to the caller contract is enabled,
        // its code hash should be available in storage
        let id = self.canonize(env.message.sender.clone())?;
        let next_contract: Option<ContractLink<HumanAddr>> =
            self.get_ns(Self::CAN_MIGRATE_TO, id.as_slice())?;
        if let Some(next_contract) = next_contract {
            if next_contract.address == env.message.sender {
                Ok(next_contract)
            } else {
                errors::immigration_disallowed()
            }
        } else {
            errors::immigration_disallowed()
        }
    }

    /// Implement this to wrap ImmigrationHandle in contract's root Handle type,
    /// so that `{"receive_migration":...}` can become `{"immigration":{"receive_migration":...}`
    fn wrap_receive_msg (&self, msg: ImmigrationHandle) -> StdResult<Binary>;

    /// If using snapshots:
    /// Generate a serialized snapshot of the migrant's data here.
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
            ImmigrationHandle::RequestMigration(contract) =>
                self.handle_request_migration(env, contract),
            ImmigrationHandle::ReceiveMigration(data) =>
                self.handle_receive_migration(env, data),
            _ => {
                Auth::assert_admin(self, &env)?;
                match msg {
                    ImmigrationHandle::EnableMigrationFrom(contract) =>
                        self.allow_immigration_from(contract),
                    ImmigrationHandle::DisableMigrationFrom(contract) =>
                        self.disallow_immigration_from(contract),
                    _ => unreachable!()
                }
            }
        }
    }

    fn allow_immigration_from (&mut self, prev: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        let id = self.canonize(prev.address.clone())?;
        self.set_ns(Self::CAN_MIGRATE_FROM, id.as_slice(), Some(prev))?;
        Ok(HandleResponse::default())
    }

    fn disallow_immigration_from (&mut self, prev: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        let id = self.canonize(prev.address)?;
        self.set_ns::<Option<ContractLink<HumanAddr>>>(Self::CAN_MIGRATE_FROM, id.as_slice(), None)?;
        Ok(HandleResponse::default())
    }

    /// This is where the migration begins.
    /// 1. User calls this method with the address of the previous contract.
    /// 2. This contract checks if migration from the previous contract is allowed.
    /// 3. This contract sends a message to the previous contract,
    ///    requesting the user's state to be exported.
    fn handle_request_migration (&mut self, env: Env, prev: ContractLink<HumanAddr>)
        -> StdResult<HandleResponse>
    {
        self.can_immigrate_from(&prev)?;
        HandleResponse::default().msg(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr:      prev.address,
            callback_code_hash: prev.code_hash,
            send:               vec![],
            msg: self.wrap_export_msg(EmigrationHandle::ExportState(env.message.sender))?,
        }))
    }

    fn can_immigrate_from (&mut self, prev: &ContractLink<HumanAddr>) -> StdResult<()> {
        let id = self.canonize(prev.address.clone())?;
        let sender_link: Option<ContractLink<HumanAddr>> =
            self.get_ns(Self::CAN_MIGRATE_FROM, id.as_slice())?;
        if let Some(sender_link) = sender_link {
            if sender_link.address == prev.address {
                Ok(())
            } else {
                errors::emigration_disallowed()
            }
        } else {
            errors::emigration_disallowed()
        }
    }

    /// Implement this to wrap EmigrationHandle in contract's root Handle type.
    /// That way, `{"export_state":...}` can become `{"emigration":{"export_state":...}`,
    /// where "emigration" is the variant from the contract's root HandleMsg enum.
    fn wrap_export_msg (&self, msg: EmigrationHandle) -> StdResult<Binary>;

    /// If using transactions:
    /// Emit messages that finalize the migration here.
    fn handle_receive_migration (&mut self, env: Env, data: Binary) -> StdResult<HandleResponse> {
        self.import_state(env, data)?;
        Ok(HandleResponse::default())
    }

    /// If using snapshots:
    /// Deserialize snapshot and save data here.
    fn import_state (&mut self, _env: Env, _data: Binary) -> StdResult<()> {
        Ok(())
    }
}
