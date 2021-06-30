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
/// An account for a single user.
pub struct Account<A> {
    /// The owner of this account.
    pub owner: A,
    /// The last time that the user claimed their rewards.
    pub last_claimed: u64,
    /// The snapshot that was last claimed at.
    pub claimed_at_snapshot: u64,
    /// The amount of LP tokens the owner has locked into this contract.
    locked_amount: Uint128,
    /// A list of submitted tokens that aren't included in the rewards calculations yet.
    pending_balances: Option<Vec<PendingBalance>>,
    /// A history of any changes that occurred to `self.locked_amount` at a given snapshot index. It is reset upon claiming.
    balance_history: Option<Vec<Snapshot>>
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema)]
pub struct PendingBalance {
    pub submitted_at: u64,
    pub amount: Uint128
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, JsonSchema)]
pub struct Snapshot {
    pub index: u64,
    pub amount: Uint128
}

impl PartialEq for Account<HumanAddr> {
    fn eq(&self, other: &Self) -> bool {
        self.owner == other.owner
    }
}

impl Account<HumanAddr> {
    pub fn new(owner: HumanAddr, current_time: u64, current_snapshot: u64) -> Self {
        Account {
            owner,
            locked_amount: Uint128::zero(),
            pending_balances: None,
            balance_history: None,
            last_claimed: current_time,
            claimed_at_snapshot: current_snapshot
        }
    }

    pub fn locked_amount(&self) -> u128 {
        self.locked_amount.u128()
    }

    pub fn total_pending(&self) -> u128 {
        if let Some(pending) = &self.pending_balances {
            pending.iter().map(|x| x.amount.u128()).sum()
        } else {
            0    
        }
    }

    pub fn history(&self) -> Option<&Vec<Snapshot>> {
        self.balance_history.as_ref()
    }

    pub fn clear_history(&mut self) {
        self.balance_history = None;
    }

    /// Adds the pending `balance` to the deposit history.
    /// Also checks if calling `unlock_pending` would cause an overflow.
    /// Returns the pending balance after the add operation.
    pub fn add_pending_balance(&mut self, balance: PendingBalance) -> StdResult<u128> {
        let total = self.total_pending();

        let new_total = total.checked_add(balance.amount.u128());

        if let Some(nt) = new_total {
            if self.locked_amount.u128().checked_add(nt).is_some() {
                if let Some(pending) = &mut self.pending_balances {
                    pending.push(balance);
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
    pub fn unlock_pending(
        &mut self,
        current_time: u64,
        interval: u64,
        current_snapshot: u64
    ) -> StdResult<u128> {
        let unlocked = self.unlock(current_time, interval)?;

        if unlocked > 0 {
            self.push_history(current_snapshot);
        }

        Ok(unlocked)
    }

    /// Unlocks any pending balance and subtracts the specified `amount` from the actual locked amount.
    pub fn subtract_balance(
        &mut self,
        amount: u128,
        current_time: u64,
        interval: u64,
        current_snapshot: u64
    ) -> StdResult<()> {
        if amount == 0 {
            return Ok(());
        }

        // Unlock without adding to history yet.
        self.unlock(current_time, interval)?;

        self.locked_amount = self.locked_amount
            .u128()
            .checked_sub(amount)
            .ok_or_else(||
                StdError::generic_err("Insufficient balance.")
            )?
            .into();

        self.push_history(current_snapshot);
        
        Ok(())
    }

    fn unlock(&mut self, current_time: u64, interval: u64) -> StdResult<u128> {
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

            for _ in 0..=index {
                balances.remove(0);
            }

            if balances.len() == 0 {
                self.pending_balances = None;
            }
        }

        Ok(total_unlocked)
    }

    fn push_history(&mut self, current_snapshot: u64) {
        let snapshot = Snapshot {
            index: current_snapshot,
            amount: self.locked_amount
        };

        if let Some(history) = &mut self.balance_history {
            history.push(snapshot);
        } else {
            self.balance_history = Some(vec![snapshot]);
        }
    }
}

impl Humanize<Account<HumanAddr>> for Account<CanonicalAddr> {
    fn humanize(&self, api: &impl Api) -> StdResult<Account<HumanAddr>> {
        Ok(Account {
            owner: self.owner.humanize(api)?,
            locked_amount: self.locked_amount,
            pending_balances: self.pending_balances.clone(),
            balance_history: self.balance_history.clone(),
            last_claimed: self.last_claimed,
            claimed_at_snapshot: self.claimed_at_snapshot
        })
    }
}

impl Canonize<Account<CanonicalAddr>> for Account<HumanAddr> {
    fn canonize (&self, api: &impl Api) -> StdResult<Account<CanonicalAddr>> {
        Ok(Account {
            owner: self.owner.canonize(api)?,
            locked_amount: self.locked_amount,
            pending_balances: self.pending_balances.clone(),
            balance_history: self.balance_history.clone(),
            last_claimed: self.last_claimed,
            claimed_at_snapshot: self.claimed_at_snapshot
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
        Account::new("user".into(), 0, 0)
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

        let unlocked = acc.unlock_pending(310, interval, 1).unwrap();
        assert_eq!(acc.locked_amount(), 150);
        assert_eq!(unlocked, 150);
        assert_eq!(acc.total_pending(), 20);

        {
            let history = acc.history().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].index, 1);
            assert_eq!(history[0].amount, Uint128(150));
        }

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(50),
            submitted_at: 340
        }).unwrap();

        let unlocked = acc.unlock_pending(350, interval, 2).unwrap();
        assert_eq!(acc.locked_amount(), 150);
        assert_eq!(unlocked, 0);
        assert_eq!(acc.total_pending(), 70);

        {
            let history = acc.history().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].index, 1);
            assert_eq!(history[0].amount, Uint128(150));
        }

        let unlocked = acc.unlock_pending(440, interval, 3).unwrap();
        assert_eq!(acc.locked_amount(), 220);
        assert_eq!(unlocked, 70);
        assert_eq!(acc.total_pending(), 0);

        {
            let history = acc.history().unwrap();
            assert_eq!(history.len(), 2);
            assert_eq!(history[0].index, 1);
            assert_eq!(history[0].amount, Uint128(150));

            assert_eq!(history[1].index, 3);
            assert_eq!(history[1].amount, Uint128(220));
        }

        acc.clear_history();
        assert!(acc.history().is_none());
    }

    #[test]
    fn test_subtract_balance() {
        let mut acc = create_account();

        let interval = 100;

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(100),
            submitted_at: 100
        }).unwrap();

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(100),
            submitted_at: 120
        }).unwrap();

        acc.subtract_balance(201, 150, interval, 0).unwrap_err();
        // No unlocked balance yet.
        acc.subtract_balance(100, 150, interval, 0).unwrap_err();

        assert_eq!(acc.locked_amount(), 0);
        assert!(acc.history().is_none());
        assert_eq!(acc.total_pending(), 200);

        acc.subtract_balance(100, 219, interval, 1).unwrap();
        assert_eq!(acc.locked_amount(), 0);
        assert_eq!(acc.total_pending(), 100);

        {
            let history = acc.history().unwrap();
            assert_eq!(history.len(), 1);
            assert_eq!(history[0].index, 1);
            assert_eq!(history[0].amount, Uint128::zero());
        }

        acc.subtract_balance(99, 220, interval, 1).unwrap();
        assert_eq!(acc.locked_amount(), 1);
        assert_eq!(acc.total_pending(), 0);

        {
            let history = acc.history().unwrap();
            assert_eq!(history.len(), 2);
            assert_eq!(history[0].index, 1);
            assert_eq!(history[0].amount, Uint128::zero());

            assert_eq!(history[1].index, 1);
            assert_eq!(history[1].amount, Uint128(1));
        }

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(100),
            submitted_at: 250
        }).unwrap();

        acc.subtract_balance(101, 350, interval, 2).unwrap();
        assert_eq!(acc.locked_amount(), 0);
        assert_eq!(acc.total_pending(), 0);

        {
            let history = acc.history().unwrap();
            assert_eq!(history.len(), 3);
            assert_eq!(history[0].index, 1);
            assert_eq!(history[0].amount, Uint128::zero());

            assert_eq!(history[1].index, 1);
            assert_eq!(history[1].amount, Uint128(1));

            assert_eq!(history[2].index, 2);
            assert_eq!(history[2].amount, Uint128::zero());
        }

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(100),
            submitted_at: 300
        }).unwrap();

        acc.add_pending_balance(PendingBalance {
            amount: Uint128(100),
            submitted_at: 320
        }).unwrap();

        acc.subtract_balance(100, 410, interval, 3).unwrap();
        assert_eq!(acc.locked_amount(), 0);
        assert_eq!(acc.total_pending(), 100);

        {
            let history = acc.history().unwrap();
            assert_eq!(history.len(), 4);
            assert_eq!(history[0].index, 1);
            assert_eq!(history[0].amount, Uint128::zero());

            assert_eq!(history[1].index, 1);
            assert_eq!(history[1].amount, Uint128(1));

            assert_eq!(history[2].index, 2);
            assert_eq!(history[2].amount, Uint128::zero());

            assert_eq!(history[3].index, 3);
            assert_eq!(history[3].amount, Uint128::zero());
        }
    }
}
