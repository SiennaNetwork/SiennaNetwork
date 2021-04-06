//! Core vesting logic

use crate::*;

pub trait Vesting {
    /// Get total amount unlocked for address `a` at time `t`.
    fn unlocked (&self, t: Seconds, a: &HumanAddr) -> u128;
}
impl Vesting for Schedule {
    /// Sum of unlocked amounts for this address for all pools
    fn unlocked (&self, t: Seconds, a: &HumanAddr) -> u128 {
        self.pools.iter().fold(0, |total, pool| total + pool.unlocked(t, a))
    }
}
impl Vesting for Pool {
    /// Sum of unlocked amounts for this address for all accounts in this pool
    fn unlocked (&self, t: Seconds, a: &HumanAddr) -> u128 {
        self.accounts.iter().fold(0, |total, account| total + account.unlocked(t, a))
    }
}
impl Vesting for Account {
    /// Unlocked sum for this account at a point in time
    fn unlocked (&self, elapsed: Seconds, a: &HumanAddr) -> u128 {
        if *a != self.address { // if asking about someone else
            0
        } else if elapsed < self.start_at { // if asking about a moment before the start
            0
        } else if elapsed >= self.end() { // at the end the full amount must've been vested
            self.amount.u128()
        } else {
            let n = self.most_recent_portion(elapsed).unwrap() as u128;
            self.cliff.u128() + n * self.portion_size()
        }
    }
}
impl Account {
    /// Size of regular (non-cliff) portions.
    pub fn portion_size (&self) -> u128 {
        if self.portion_count() > 0 {
            self.amount_after_cliff() / self.portion_count() as u128
        } else {
            0
        }
    }
    /// Amount to vest after the cliff
    pub fn amount_after_cliff (&self) -> u128 {
        assert!(self.amount >= self.cliff);
        self.amount.u128() - self.cliff.u128()
    }
    /// Number of non-cliff portions.
    pub fn portion_count (&self) -> u64 {
        if self.interval > 0 {
            (self.duration / self.interval) as u64
        } else {
            0
        }
    }
    /// If `(amount-cliff)` doesn't divide evenly by `portion_size`,
    /// the remainder is added to the last portion.
    pub fn remainder (&self) -> u128 {
        self.amount_after_cliff() - self.portion_size() * self.portion_count() as u128
    }
    /// Timestamp of last vesting (when remainder is received)
    pub fn end (&self) -> Seconds {
        self.start_at + self.duration
    }
    /// Time elapsed since start
    pub fn elapsed (&self, t: Seconds) -> Option<Seconds> {
        if t >= self.start_at {
            Some(t - self.start_at)
        } else {
            None
        }
    }
    /// Most recent portion vested at time `t`
    pub fn most_recent_portion (&self, t: Seconds) -> Option<u64> {
        match self.elapsed(t) {
            Some(elapsed) => Some(u64::min(
                elapsed / self.interval + match self.cliff.u128() { 0 => 1, _ => 0 },
                self.portion_count()
            )),
            None => None
        }
    }
    /// Whether a portion is unlocked at the exact moment specified
    pub fn vests_at (&self, t: Seconds) -> bool {
        match self.elapsed(t) {
            Some(elapsed) => elapsed % self.interval == 0,
            None => false
        }
    }
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use cosmwasm_std::HumanAddr;
    use crate::{Schedule, Pool, Account, vesting::Vesting};
    #[test] fn blank () {
        // some imaginary people:
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        // some empty, but valid schedules
        for S in &[
            Schedule::new(&[]),
            Schedule::new(&[ Pool::full("", &[]) ]),
            Schedule::new(&[ Pool::full("", &[ Account::immediate("", &Alice, 0) ]) ]),
            Schedule::new(&[ Pool::partial("", 1, &[ Account::immediate("", &Alice, 0) ]) ])
        ] {
          assert_eq!(S.unlocked(0, &Alice), 0);
          assert_eq!(S.unlocked(1, &Alice), 0);
          assert_eq!(S.unlocked(1001, &Bob), 0);
        }
    }
    #[test] fn vest_immediate () {
        // a periodic `Account`...
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let A = Account::immediate("", &Alice, 100);
        let P = Pool::full("", &[A.clone()]);
        let S = Schedule::new(&[P.clone()]);
        for (l, r) in &[
            (A.amount.u128(),        100),
            (A.cliff.u128(),           0),
            (A.amount_after_cliff(), 100),
            (A.start_at.into(),        0),
            (A.interval.into(),        0),
            (A.duration.into(),        0),
            (A.portion_count().into(), 0),
            (A.portion_size(),         0),
            (A.remainder(),          100),
        ] {
            assert_eq!(l, r);
        }
        assert_eq!(100, P.total.u128());
        assert_eq!(100, S.total.u128());
        for t in 0..100 {
            assert_eq!(100, S.unlocked(t, &Alice));
            assert_eq!(  0, S.unlocked(t, &Bob));
        }
    }
    #[test] fn vest_immediate_as_cliff () { // different way of expressing the same thing
        // a periodic `Account`...
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let A = Account::periodic("", &Alice, 100, 100, 0, 0, 0);
        let P = Pool::full("", &[A.clone()]);
        let S = Schedule::new(&[P.clone()]);
        for (l, r) in &[
            (A.amount.u128(),        100),
            (A.cliff.u128(),         100),
            (A.amount_after_cliff(),   0),
            (A.start_at.into(),        0),
            (A.interval.into(),        0),
            (A.duration.into(),        0),
            (A.portion_count().into(), 0),
            (A.portion_size(),         0),
            (A.remainder(),            0),
        ] {
            assert_eq!(l, r);
        }
        assert_eq!(100, P.total.u128());
        assert_eq!(100, S.total.u128());
        for t in 0..100 {
            assert_eq!(100, S.unlocked(t, &Alice));
            assert_eq!(  0, S.unlocked(t, &Bob));
        }
    }
    #[test] fn vest_periodic_with_cliff () {
        let Alice = HumanAddr::from("Alice");
        let Bob = HumanAddr::from("Bob");
        let A = Account::periodic("", &Alice, 100, 42, 7, 12, 70);
        let P = Pool::full("", &[A.clone()]);
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(100, S.total.u128());
        assert_eq!(100, P.total.u128());
        for (l, r) in &[
            (A.amount.u128(),        100),
            (A.cliff.u128(),          42),
            (A.amount_after_cliff(),  58),
            (A.start_at.into(),        7),
            (A.interval.into(),       12),
            (A.duration.into(),       70),
            (A.end().into(),          77),
            (A.portion_count().into(), 5),
            (A.portion_size(),        11),
            (A.remainder(),            3),
        ] {
            assert_eq!(l, r);
        }
        println!("\n {:<11}│ {:<11} │ {:<11}│ {:<11}", "T", "Event", "Alice", "Bob");
        println!("{:─^52}┐", "");
        let mut a = 0;
        let mut b = 0;
        for t in 1..200 {
            if t == A.start_at + A.duration + A.interval { break }
            a = S.unlocked(t, &Alice);
            b = S.unlocked(t, &Bob);
            print!("{:>12}│", t);
            println!("{:>12}│{:>12}│{:>12}│", if t < A.start_at {
                assert_eq!(a, 0);
                assert_eq!(b, 0);
                String::from("😴 pre")
            } else if t == A.start_at {
                assert_eq!(a, A.cliff.u128());
                assert_eq!(b, 0);
                String::from("🚀 cliff")
            } else if t == A.end() && A.remainder() > 0 {
                assert_eq!(a, A.amount.u128());
                assert_eq!(b, 0);
                String::from("✨ remainder")
            } else if t >= A.end() {
                assert_eq!(a, A.amount.u128());
                assert_eq!(b, 0);
                String::from("✅ done")
            } else if A.vests_at(t) {
                let p = A.most_recent_portion(t).unwrap() as u128;
                assert_eq!(a, A.cliff.u128() + p * A.portion_size());
                assert_eq!(b, 0);
                format!("💸 vest #{}", p)
            } else {
                String::from("⏳ wait")
            }, &a, &b);
        }
        assert_eq!(a, A.amount.u128());
        assert_eq!(b, 0);
    }
    #[test] fn vest_periodic_no_cliff () {
        let Alice = HumanAddr::from("Alice");
        let Bob = HumanAddr::from("Bob");
        let A = Account::periodic("", &Alice, 92, 0, 20, 11, 90);
        let P = Pool::full("", &[A.clone()]);
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(92, S.total.u128());
        assert_eq!(92, P.total.u128());
        for (l, r) in &[
            (A.amount.u128(),         92),
            (A.cliff.u128(),           0),
            (A.amount_after_cliff(),  92),
            (A.start_at.into(),       20),
            (A.interval.into(),       11),
            (A.duration.into(),       90),
            (A.end().into(),         110),
            (A.portion_count().into(), 8),
            (A.portion_size(),        11),
            (A.remainder(),            4),
        ] {
            assert_eq!(l, r);
        }
        println!("\n {:<11}│ {:<11} │ {:<11}│ {:<11}", "T", "Event", "Alice", "Bob");
        println!("{:─^52}┐", "");
        let mut a = 0;
        let mut b = 0;
        for t in 1..200 {
            if t == A.start_at + A.duration + A.interval { break }
            a = S.unlocked(t, &Alice);
            b = S.unlocked(t, &Bob);
            print!("{:>12}│", t);
            println!("{:>12}│{:>12}│{:>12}│", if t < A.start_at {
                assert_eq!(a, 0);
                assert_eq!(b, 0);
                String::from("😴 pre")
            } else if t == A.end() && A.remainder() > 0 {
                assert_eq!(a, A.amount.u128());
                assert_eq!(b, 0);
                String::from("✨ remainder")
            } else if t >= A.end() {
                assert_eq!(a, A.amount.u128());
                assert_eq!(b, 0);
                String::from("✅ done")
            } else if A.vests_at(t) {
                let p = A.most_recent_portion(t).unwrap() as u128;
                assert_eq!(a, p * A.portion_size());
                assert_eq!(b, 0);
                format!("💸 vest #{}", p)
            } else {
                String::from("⏳ wait")
            }, &a, &b);
        }
        assert_eq!(a, A.amount.u128());
        assert_eq!(b, 0);
    }
}
