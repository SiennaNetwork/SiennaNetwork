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
    fn unlocked (&self, t_query: Seconds, a: &HumanAddr) -> u128 {
        if *a != self.address { // if asking about someone else
            return 0
        }
        if t_query < self.start_at { // if asking about a moment before the start
            return 0
        }
        let mut vested = 0u128;
        let mut t_cursor = self.start_at;
        if self.cliff > Uint128::zero() { // if there's a cliff
            vested += self.cliff.u128();  // vest it first
            t_cursor += self.interval;    // and push the rest of the portions by one
        }
        if self.portion_size() > 0 { // prevent infinite loop
            let t_end = self.end();
            let max = self.amount.u128();
            loop {
                if vested >= max || t_cursor >= t_end { // clamp by both time and amount
                    vested = max;  // this implicitly adds the remainder
                    break // and makes sure the contract never overspends
                }
                if t_cursor > t_query { // if asking about a point of time before the end
                    break               // stop here
                }
                vested += self.portion_size();
                t_cursor += self.interval;
            }
        }
        vested
    }
}
impl Account {
    /// Amount to vest after the cliff
    pub fn amount_after_cliff (&self) -> u128 {
        self.amount.u128() - self.cliff.u128()
    }
    /// Number of non-cliff portions.
    pub fn portion_count (&self) -> u128 {
        if self.amount_after_cliff() > 0 {
            if self.duration > 0 && self.interval > 0 {
                (self.duration / self.interval) as u128 // 1 or more portions besides cliff
            } else {
                1 // one portion besides cliff (e.g. cliff + remainder)
            }
        } else {
            0 // immediate vesting (cliff only, no extra portions)
        }
    }
    /// Size of non-cliff portions.
    pub fn portion_size (&self) -> u128 {
        if self.amount_after_cliff() > 0 {
            self.amount_after_cliff() / self.portion_count()
        } else {
            0 // immediate vesting (cliff only, no extra portions)
        }
    }
    /// If `(amount-cliff)` doesn't divide evenly by `portion_size`,
    /// the remainder is added to the last portion.
    pub fn remainder (&self) -> u128 {
        self.amount_after_cliff() - self.portion_size() * self.portion_count()
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
    pub fn most_recent_portion (&self, t: Seconds) -> Option<Seconds> {
        match self.elapsed(t) {
            Some(elapsed) => Some(elapsed / self.interval + 1),
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
    use crate::{Schedule, Pool, Account, Vesting};
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
            (A.portion_count().into(), 1),
            (A.portion_size(),       100),
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
        println!(" {:<11}│ {:<11} │ {:<11}│ {:<11}", "T", "Event", "Alice", "Bob");
        println!("{:─^52}┐", "");
        let mut a = 0;
        let mut b = 0;
        for t in 1..200 {
            if t == A.start_at + A.duration + A.interval { break }
            a = S.unlocked(t, &Alice);
            b = S.unlocked(t, &Bob);
            println!("{:>12}│{:>12}│{:>12}│{:>12}│", t, if t < A.start_at {
                String::from("😴 pre")
            } else if t == A.start_at {
                String::from("✨ cliff")
            } else if A.vests_at(t) {
                format!("💸 vest #{}", A.most_recent_portion(t).unwrap())
            } else if t == A.end() && A.remainder() > 0 {
                String::from("✨ remainder")
            } else if t >= A.end() {
                String::from("✅ done")
            } else {
                String::from("⏳ wait")
            }, &a, &b);
        }
        assert_eq!(a, A.amount.u128());
        assert_eq!(b, 0);
        panic!()
    }
    #[test] fn vest_periodic_no_cliff () {
        let Alice = HumanAddr::from("Alice");
        let Bob = HumanAddr::from("Bob");
        let A = Account::periodic("", &Alice, 90, 0, 20, 11, 90);
        let P = Pool::full("", &[A.clone()]);
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(90, S.total.u128());
        assert_eq!(90, P.total.u128());
        for (l, r) in &[
            (A.amount.u128(),         90),
            (A.cliff.u128(),           0),
            (A.amount_after_cliff(),  90),
            (A.start_at.into(),       20),
            (A.interval.into(),       11),
            (A.duration.into(),       90),
            (A.end().into(),         110),
            (A.portion_count().into(), 8),
            (A.portion_size(),        11),
            (A.remainder(),            2),
        ] {
            assert_eq!(l, r);
        }
        println!(" {:<11}│ {:<11} │ {:<11}│ {:<11}", "T", "Event", "Alice", "Bob");
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
            } else if A.vests_at(t) {
                let p = A.most_recent_portion(t).unwrap() as u128;
                assert_eq!(a, p * A.portion_size());
                assert_eq!(b, 0);
                format!("💸 vest #{}", p)
            } else if t == A.end() && A.remainder() > 0 {
                String::from("✨ remainder")
            } else if t >= A.end() {
                String::from("✅ done")
            } else {
                String::from("⏳ wait")
            }, &a, &b);
        }
        assert_eq!(a, A.amount.u128());
        assert_eq!(b, 0);
        panic!()
    }
}
