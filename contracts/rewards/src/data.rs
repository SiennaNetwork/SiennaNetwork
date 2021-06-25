use cosmwasm_std::{Api, CanonicalAddr, HumanAddr, StdResult, Uint128, StdError};
use fadroma_scrt_callback::ContractInstance;
use fadroma_scrt_addr::{Canonize, Humanize};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

use crate::msg::{OVERFLOW_MSG, UNDERFLOW_MSG};

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
    /// The last time that the user claimed their rewards.
    pub last_claimed: u64,
    /// The amount of LP tokens the owner has locked into this contract.
    locked_amount: Uint128,
    /// A history of submitted tokens that aren't included in the rewards calculations yet.
    pending_balances: Option<Vec<PendingBalance>>
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema)]
pub struct PendingBalance {
    pub submitted_at: u64,
    pub amount: Uint128
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
            pending_balances: None,
            last_claimed: 0
        }
    }

    pub fn locked_amount(&self) -> Uint128 {
        self.locked_amount
    }

    pub fn total_pending(&self) -> u128 {
        if let Some(pending) = &self.pending_balances {
            pending.iter().map(|x| x.amount.u128()).sum()
        } else {
            0    
        }
    }

    /// Adds the pending `balance` to the deposit history.
    /// Also checks if calling `unlock_pending` would cause an overflow.
    /// Returns the pending balance after the add operation.
    pub fn add_pending_balance(&mut self, balance: PendingBalance) -> StdResult<u128> {
        let total = self.total_pending();

        let new_total = total.checked_add(balance.amount.u128());

        if let Some(nt) = new_total {
            if self.locked_amount.u128().checked_add(nt).is_some() {
                if let Some(history) = &mut self.pending_balances {
                    history.push(balance);
                } else {
                    self.pending_balances = Some(vec![balance]);
                }

                return Ok(nt);
            }
        }

        return Err(StdError::generic_err(OVERFLOW_MSG));
    }

    /// Adds any of the pending balance that can be unlocked to the actual locked amount.
    /// Returns the amount unlocked.
    pub fn unlock_pending(&mut self, current_time: u64, interval: u64) -> StdResult<u128> {
        let mut total_unlocked = 0u128;

        if let Some(balances) = &mut self.pending_balances {
            let mut index = 0;

            for (i, balance) in balances.iter().enumerate() {
                let gap = current_time.checked_sub(balance.submitted_at).ok_or_else(||
                    // Will happen if a wrong time has been provided in claim simulation
                    StdError::generic_err(UNDERFLOW_MSG)
                )?;

                if gap < interval {
                    if i == 0 {
                        return Ok(0);
                    }

                    break;
                }

                total_unlocked += balance.amount.u128();
                self.locked_amount += balance.amount;
                
                index = i;
            }

            let remaining: Vec<PendingBalance> = balances
                .drain(0..=index)
                .collect();

            if remaining.len() == 0 {
                self.pending_balances = None;
            }
        }

        Ok(total_unlocked)
    }

    /// Subtracts the specified `amount` from the account, starting from
    /// the pending balance and any remainder - from the actual locked amount.
    /// Returns the amount subtracted **ONLY** from the locked amount.
    pub fn subtract_balance(&mut self, mut amount: u128) -> StdResult<u128> {
        if let Some(balances) = &mut self.pending_balances {
            let mut index = 0;

            for (i, balance) in balances.iter_mut().rev().enumerate() {
                if balance.amount.u128() <= amount {
                    amount -= balance.amount.u128();
                } else {
                    balance.amount = Uint128(balance.amount.u128() - amount);

                    if balance.amount > Uint128::zero() {
                        return Ok(0);
                    }

                    break;
                }

                index = i;
            }

            let remaining: Vec<PendingBalance> = balances
                .drain(index..balances.len())
                .collect();

            if remaining.len() == 0 {
                self.pending_balances = None;
            }

            if amount > 0 {
                self.locked_amount = self.locked_amount
                    .u128()
                    .checked_sub(amount)
                    .ok_or_else(||
                        StdError::generic_err("Insufficient balance.")
                    )?
                    .into();

                return Ok(amount);
            }

            Ok(0)
        } else {
            self.locked_amount = self.locked_amount
                .u128()
                .checked_sub(amount)
                .ok_or_else(||
                    StdError::generic_err("Insufficient balance.")
                )?
                .into();
            
            Ok(amount)
        }   
    }
}

impl Humanize<Account<HumanAddr>> for Account<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Account<HumanAddr>> {
        Ok(Account {
            owner: self.owner.humanize(api)?,
            lp_token_addr: self.lp_token_addr.humanize(api)?,
            locked_amount: self.locked_amount,
            pending_balances: self.pending_balances.clone(),
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
            pending_balances: self.pending_balances.clone(),
            last_claimed: self.last_claimed
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

#[cfg(test)]
mod tests {
    use super::*;

    fn create_account() -> Account<HumanAddr> {
        Account::new("user".into(), "lp_token".into())
    }

    #[test]
    fn test_add_pending_balance() {
        let mut acc = create_account();

        let amount = 100;

        let total = acc.add_pending_balance(PendingBalance {
            amount: Uint128(amount),
            submitted_at: 50
        }).unwrap();

        assert_eq!(total, amount);

        let total = acc.add_pending_balance(PendingBalance {
            amount: Uint128(amount),
            submitted_at: 50
        }).unwrap();

        assert_eq!(total, amount * 2);
        assert_eq!(acc.total_pending(), amount * 2);

        let result = acc.add_pending_balance(PendingBalance {
            amount: Uint128(u128::MAX),
            submitted_at: 50
        });
        assert!(result.is_err());

        // Should overflow when trying to add to locked_amount
        let result = acc.add_pending_balance(PendingBalance {
            amount: Uint128((u128::MAX - amount * 2) + 1),
            submitted_at: 50
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_unlock_pending_balance() {
        let mut acc = create_account();
        let interval = 100;

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(100),
            submitted_at: 100
        }).unwrap();

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(50),
            submitted_at: 200
        }).unwrap();

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(20),
            submitted_at: 300
        }).unwrap();

        let unlocked = acc.unlock_pending(310, interval).unwrap();
        assert_eq!(unlocked, 150);
        assert_eq!(acc.total_pending(), 20);

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(50),
            submitted_at: 340
        }).unwrap();

        let unlocked = acc.unlock_pending(350, interval).unwrap();
        assert_eq!(unlocked, 0);
        assert_eq!(acc.total_pending(), 70);

        let unlocked = acc.unlock_pending(440, interval).unwrap();
        assert_eq!(unlocked, 70);
        assert_eq!(acc.total_pending(), 0);
    }
}
