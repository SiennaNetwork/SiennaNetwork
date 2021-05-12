use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult};
use cosmwasm_utils::{ContractInfo, ContractInfoStored};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Represents a pair that is eligible for rewards.
pub struct RewardPool {
    pub lp_token: ContractInfo,
    /// The reward amount allocated to this pool.
    pub share: u128,
    /// Total amount locked by all participants.
    pub size: u128
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
/// Accounts are based on (user - LP token) pairs. This means that a single
/// user can have multiple accounts - one for each LP token.
pub struct Account {
    /// The owner of this account.
    pub owner: HumanAddr,
    /// The address of the LP token that this account is for.
    pub lp_token_addr: HumanAddr,
    /// The amount of LP tokens the owner has locked into this contract.
    pub locked_amount: u128,
    /// The last time that the user claimed their rewards.
    pub last_claimed: u64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct RewardPoolStored {
    pub lp_token: ContractInfoStored,
    pub share: u128,
    pub size: u128
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub(crate) struct AccountStored {
    pub owner: CanonicalAddr,
    pub lp_token_addr: CanonicalAddr,
    pub locked_amount: u128,
    pub last_claimed: u64
}

impl PartialEq for Account {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.lp_token_addr == self.lp_token_addr
    }
}

impl Account {
    pub fn new(owner: HumanAddr, lp_token_addr: HumanAddr) -> Self {
        Account {
            owner,
            lp_token_addr,
            locked_amount: 0,
            last_claimed: 0
        }
    }

    pub(crate) fn to_stored(&self, api: &impl Api) -> StdResult<AccountStored> {
        Ok(AccountStored {
            owner: api.canonical_address(&self.owner)?,
            lp_token_addr: api.canonical_address(&self.lp_token_addr)?,
            locked_amount: self.locked_amount,
            last_claimed: self.last_claimed,
        })
    }
}

impl AccountStored {
    pub(crate) fn to_normal(self, api: &impl Api) -> StdResult<Account> {
        Ok(Account {
            owner: api.human_address(&self.owner)?,
            lp_token_addr: api.human_address(&self.lp_token_addr)?,
            locked_amount: self.locked_amount,
            last_claimed: self.last_claimed
        })
    }
}

impl RewardPool {
    pub(crate) fn to_stored(&self, api: &impl Api) -> StdResult<RewardPoolStored> {
        Ok(RewardPoolStored {
            lp_token: self.lp_token.to_stored(api)?,
            share: self.share,
            size: self.size
        })
    }
}

impl RewardPoolStored {
    pub(crate) fn to_normal(self, api: &impl Api) -> StdResult<RewardPool> {
        Ok(RewardPool {
            lp_token: self.lp_token.to_normal(api)?,
            share: self.share,
            size: self.size
        })
    }
}