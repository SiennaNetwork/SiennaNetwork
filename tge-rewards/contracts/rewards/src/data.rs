use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult, Uint128};
use fadroma_scrt_callback::ContractInstance;
use fadroma_scrt_addr::{Canonize, Humanize};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
/// Represents a pair that is eligible for rewards.
pub struct RewardPool<A> {
    pub lp_token: ContractInstance<A>,
    /// The reward amount allocated to this pool.
    pub share: Uint128,
    /// Total amount locked by all participants.
    pub size: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
/// Accounts are based on (user - LP token) pairs. This means that a single
/// user can have multiple accounts - one for each LP token.
pub struct Account<A> {
    /// The owner of this account.
    pub owner: A,
    /// The address of the LP token that this account is for.
    pub lp_token_addr: A,
    /// The amount of LP tokens the owner has locked into this contract.
    pub locked_amount: Uint128,
    /// The last time that the user claimed their rewards.
    pub last_claimed: u64
}

impl PartialEq for Account<HumanAddr> {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner && self.lp_token_addr == self.lp_token_addr
    }
}

impl PartialEq for RewardPool<HumanAddr> {
    fn eq(&self, other: &Self) -> bool {
        self.lp_token.address == other.lp_token.address
    }
}

impl Account<HumanAddr> {
    pub fn new(owner: HumanAddr, lp_token_addr: HumanAddr) -> Self {
        Account {
            owner,
            lp_token_addr,
            locked_amount: Uint128::zero(),
            last_claimed: 0
        }
    }
}

impl Humanize<Account<HumanAddr>> for Account<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Account<HumanAddr>> {
        Ok(Account {
            owner: self.owner.humanize(api)?,
            lp_token_addr: self.lp_token_addr.humanize(api)?,
            locked_amount: self.locked_amount,
            last_claimed: self.last_claimed
        })
    }
}

impl Canonize<Account<CanonicalAddr>> for Account<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Account<CanonicalAddr>> {
        Ok(Account {
            owner: self.owner.canonize(api)?,
            lp_token_addr: self.lp_token_addr.canonize(api)?,
            locked_amount: self.locked_amount,
            last_claimed: self.last_claimed,
        })
    }
}

impl Humanize<RewardPool<HumanAddr>> for RewardPool<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<RewardPool<HumanAddr>> {
        Ok(RewardPool {
            lp_token: self.lp_token.humanize(api)?,
            share: self.share,
            size: self.size
        })
    }
}

impl Canonize<RewardPool<CanonicalAddr>> for RewardPool<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<RewardPool<CanonicalAddr>> {
        Ok(RewardPool {
            lp_token: self.lp_token.canonize(api)?,
            share: self.share,
            size: self.size
        })
    }
}
