use cosmwasm_std::{StdError, HumanAddr};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum ContractStatusLevel {
    Normal,
    Paused,
    Migration,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct ContractStatus {
    level:       ContractStatusLevel,
    reason:      String,
    new_address: Option<HumanAddr>
}

macro_rules! migration_message {
    (paused: $reason:expr) => { format!(
         "This contract has been paused. Reason: {}",
         &$reason
    ) };
    (migration: $reason:expr, $new_address:expr) => { format!(
         "This contract is being migrated to {}, please use that address instead. Reason: {}",
         &$new_address,
         &$reason
    ) };
}

pub fn is_operational (status: &ContractStatus) -> StatefulResult<()> {
    let ContractStatus { level, reason, new_address } = status;
    match level {
        ContractStatusLevel::Normal => Ok(((), None)),
        ContractStatusLevel::Paused => Err(StatefulError((StdError::GenericErr {
            backtrace: None,
            msg: migration_message!(paused: reason)
        }, None))),
        ContractStatusLevel::Migration => Err(StatefulError((StdError::GenericErr {
            backtrace: None,
            msg: migration_message!(migration: reason, new_address)
        }, None))),
    }
}

pub fn can_set_status (
    status: &ContractStatus,
    new_status_level: ContractStatusLevel
) -> StatefulResult<()> {
    let ContractStatus { level, reason, new_address } = status;
    match level {
        ContractStatusLevel::Normal => Ok(((), None)),
        ContractStatusLevel::Paused => Ok(((), None)),
        ContractStatusLevel::Migration => match new_status_level {
            // if already migrating, allow message and new_address to be updated
            ContractStatusLevel::Migration => Ok(((), None)),
            // but prevent reverting from migration status
            _ => Err(StatefulError((StdError::GenericErr {
                backtrace: None,
                msg: migration_message!(migration: reason, new_address);
            }, None)))
        }
    }
}
