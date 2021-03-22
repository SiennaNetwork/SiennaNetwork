use cosmwasm_std::{HumanAddr, CanonicalAddr, StdResult, Api};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Code hash and address of a contract.
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct ContractInfo {
    pub code_hash: String,
    pub address: HumanAddr,
}
/// Code hash and address of a contract.
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct ContractInfoStored {
    pub code_hash: String,
    pub address: CanonicalAddr,
}

/// Info used to instantiate a contract
#[derive(Serialize, Deserialize, JsonSchema, Clone, PartialEq, Debug)]
pub struct ContractInstantiationInfo {
    pub code_hash: String,
    pub id: u64
}

impl Default for ContractInfo {
    fn default() -> Self {
        ContractInfo {
            code_hash: "".into(),
            address: HumanAddr::default()
        }
    }
}

impl ContractInfo {
    pub fn to_stored(&self, api: &impl Api) -> StdResult<ContractInfoStored> {
        Ok(ContractInfoStored {
            code_hash: self.code_hash.clone(),
            address: api.canonical_address(&self.address)?
        })
    }
}

impl ContractInfoStored {
    pub fn to_normal(self, api: &impl Api) -> StdResult<ContractInfo> {
        Ok(ContractInfo {
            code_hash: self.code_hash,
            address: api.human_address(&self.address)?
        })
    }
}
