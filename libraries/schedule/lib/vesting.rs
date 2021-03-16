use crate::*;

pub trait Vesting {
    /// Get total amount unlocked for address `a` at time `t`.
    fn vested (&self, a: &HumanAddr, t: Seconds) -> u128;
}
impl Vesting for Schedule {
    fn vested (&self, a: &HumanAddr, t: Seconds) -> u128 {
        self.pools.iter().fold(0, |total, pool| total + pool.vested(a, t))
    }
}
impl Vesting for Pool {
    fn vested (&self, a: &HumanAddr, t: Seconds) -> u128 {
        self.accounts.iter().fold(0, |total, account| total + account.vested(a, t))
    }
}
impl Vesting for Account {
    fn vested (&self, a: &HumanAddr, t: Seconds) -> u128 {
        if *a != self.address { return 0 }
        if t < self.start_at { return 0 }
        let mut amount = 0u128;
        let mut t_cursor = self.start_at;
        if self.cliff > Uint128::zero() {
            amount += self.cliff.u128();
            t_cursor += self.interval;
        }
        loop {
            if t_cursor > t { break }
            if t_cursor > self.duration { break }
            amount += self.portion_size();
            t_cursor += self.interval;
        }
        if t_cursor > self.duration {
            amount += self.amount.u128() - amount;
        }
        amount
    }
}
