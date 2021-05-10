use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult, Uint128};
use cosmwasm_utils::{ContractInfo, ContractInfoStored};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Represents a pair that is eligible for rewards.
pub struct RewardPool {
    pub lp_token: ContractInfo,
    /// The reward amount allocated to this pool.
    pub share: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
pub struct Account {
    pub address: HumanAddr,
    pub locked_amount: Uint128,
    pub last_claimed: u64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct RewardPoolStored {
    pub lp_token: ContractInfoStored,
    pub share: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct AccountStored {
    pub address: CanonicalAddr,
    pub locked_amount: Uint128,
    pub last_claimed: u64
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.address == other.address
    }
}

impl Account {
    pub(crate) fn to_stored(&self, api: &impl Api) -> StdResult<AccountStored> {
        Ok(AccountStored {
            address: api.canonical_address(&self.address)?,
            locked_amount: self.locked_amount,
            last_claimed: self.last_claimed,
        })
    }
}

impl AccountStored {
    pub(crate) fn to_normal(self, api: &impl Api) -> StdResult<Account> {
        Ok(Account {
            address: api.human_address(&self.address)?,
            locked_amount: self.locked_amount,
            last_claimed: self.last_claimed
        })
    }
}

impl RewardPool {
    pub(crate) fn to_stored(&self, api: &impl Api) -> StdResult<RewardPoolStored> {
        Ok(RewardPoolStored {
            lp_token: self.lp_token.to_stored(api)?,
            share: self.share
        })
    }
}

impl RewardPoolStored {
    pub(crate) fn to_normal(self, api: &impl Api) -> StdResult<RewardPool> {
        Ok(RewardPool {
            lp_token: self.lp_token.to_normal(api)?,
            share: self.share
        })
    }
}