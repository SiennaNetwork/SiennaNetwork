//! Core vesting logic

use crate::*;

pub trait Vesting {
    /// Get total amount unlocked for address `a` at time `t`.
    fn unlocked (&self, a: &HumanAddr, t: Seconds) -> u128;
}
impl Vesting for Schedule {
    fn unlocked (&self, a: &HumanAddr, t: Seconds) -> u128 {
        self.pools.iter().fold(0, |total, pool| total + pool.unlocked(a, t))
    }
}
impl Vesting for Pool {
    fn unlocked (&self, a: &HumanAddr, t: Seconds) -> u128 {
        self.accounts.iter().fold(0, |total, account| total + account.unlocked(a, t))
    }
}
impl Vesting for Account {
    fn unlocked (&self, a: &HumanAddr, t_query: Seconds) -> u128 {
        if *a != self.address { return 0 }
        if t_query < self.start_at { return 0 }
        let mut vested = 0u128;
        let mut t_cursor = self.start_at;
        if self.cliff > Uint128::zero() {
            vested += self.cliff.u128();
            t_cursor += self.interval;
        }
        if self.interval > 0 {
            let t_end = self.start_at + self.duration;
            loop {
                if t_cursor >= t_end {
                    vested += self.remainder();
                    break
                }
                if t_cursor > t_query {
                    break
                }
                vested += self.portion_size();
                t_cursor += self.interval;
            }
        } else {
            vested += self.portion_size()
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
            if self.interval > 0 {
                (self.duration / self.interval) as u128 // multiple portions besides cliff
            } else {
                1 // one portion in addition to the cliff (cliff + remainder)
            }
        } else {
            0 // immediate vesting (cliff only, no extra portions)
        }
    }
    /// Size of non-cliff portions.
    pub fn portion_size (&self) -> u128 {
        if self.amount_after_cliff () > 0 {
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
}

#[cfg(test)]
mod tests {
    #![allow(non_snake_case)]
    use cosmwasm_std::HumanAddr;
    use crate::{Schedule, Pool, Account, Vesting};
    #[test] fn test_blank () {
        // some imaginary people:
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        // some empty, but valid schedules
        for s in &[
            Schedule::new(&[]),
            Schedule::new(&[ Pool::full("", &[]) ]),
            Schedule::new(&[ Pool::full("", &[ Account::immediate("", &Alice, 0) ]) ]),
            Schedule::new(&[ Pool::partial("", 1, &[ Account::immediate("", &Alice, 0) ]) ])
        ] {
          assert_eq!(0, s.unlocked(&Alice, 0));
          assert_eq!(0, s.unlocked(&Alice, 1));
          assert_eq!(0, s.unlocked(&Bob,   1001));
        }
    }
    #[test] fn test_vest_immediate () {
        // a periodic `Account`...
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let A = Account::immediate("", &Alice, 100);
        assert_eq!(100, A.amount.u128());
        assert_eq!(  0, A.cliff.u128());
        assert_eq!(100, A.amount_after_cliff());
        assert_eq!(  0, A.start_at);
        assert_eq!(  0, A.interval);
        assert_eq!(  0, A.duration);
        assert_eq!(  0, A.portion_count());
        assert_eq!(100, A.portion_size());
        assert_eq!(  0, A.remainder());
        // ...in a `Pool`...
        let P = Pool::full("", &[A.clone()]);
        assert_eq!(100, P.total.u128());
        // ...in a `Schedule`.
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(100, S.total.u128());
        for t in 0..100 {
            assert_eq!(100, S.unlocked(&Alice, t));
            assert_eq!(  0, S.unlocked(&Bob, t));
        }
    }
    #[test] fn test_vest_periodic_no_cliff () {
        // a periodic `Account`...
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let A = Account::periodic("", &Alice, 90, 0, 20, 11, 90);
        assert_eq!(90, A.amount.u128());
        assert_eq!( 0, A.cliff.u128());
        assert_eq!(90, A.amount_after_cliff());
        assert_eq!(20, A.start_at);
        assert_eq!(11, A.interval);
        assert_eq!(90, A.duration);
        assert_eq!( 8, A.portion_count());
        assert_eq!(11, A.portion_size());
        assert_eq!( 2, A.remainder());
        // ...in a `Pool`...
        let P = Pool::full("", &[A.clone()]);
        assert_eq!(90, P.total.u128());
        // ...in a `Schedule`.
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(90, S.total.u128());

        // Before start...
        for t in 0..A.start_at {
            print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
            assert_eq!(0, S.unlocked(&Alice, t));
            assert_eq!(0, S.unlocked(&Bob, t));
        }
        // ...portions...
        for n in 0..(A.portion_count() as u64) {
            let t_portion:      u64 = A.start_at + n*A.interval;
            let t_next_portion: u64 = t_portion + A.interval;
            for t in t_portion..t_next_portion {
                print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
                assert_eq!(S.unlocked(&Alice, t), (n+1)as u128 *11u128);
                assert_eq!(S.unlocked(&Bob, t),   0u128);
            }
        }
        // ...last portion and remainder.
        let t_remainder = A.start_at + A.duration;
        for t in t_remainder..t_remainder + A.duration {
            assert_eq!(S.unlocked(&Alice, t), 100);
            assert_eq!(S.unlocked(&Bob, t), 0);
        }
    }
    #[test] fn test_vest_periodic () {
        // a periodic `Account`...
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let A = Account::periodic("", &Alice, 100, 10, 20, 11, 90);
        assert_eq!(100, A.amount.u128());
        assert_eq!( 10, A.cliff.u128());
        assert_eq!( 90, A.amount_after_cliff());
        assert_eq!( 20, A.start_at);
        assert_eq!( 11, A.interval);
        assert_eq!( 90, A.duration);
        assert_eq!(  8, A.portion_count());
        assert_eq!( 11, A.portion_size());
        assert_eq!(  2, A.remainder());
        // ...in a `Pool`...
        let P = Pool::full("", &[A.clone()]);
        assert_eq!(100, P.total.u128());
        // ...in a `Schedule`.
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(100, S.total.u128());

        // Before start...
        for t in 0..A.start_at {
            print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
            assert_eq!(0, S.unlocked(&Alice, t));
            assert_eq!(0, S.unlocked(&Bob, t));
        }
        // ...cliff...
        for t in A.start_at..(A.start_at+A.interval) {
            print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
            assert_eq!(S.unlocked(&Alice, t), 10);
            assert_eq!(S.unlocked(&Bob, t), 0);
        }
        // ...portions...
        for n in 1..(A.portion_count() as u64) {
            let t_portion:      u64 = A.start_at + n*A.interval;
            let t_next_portion: u64 = t_portion + A.interval;
            for t in t_portion..t_next_portion {
                print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
                assert_eq!(S.unlocked(&Alice, t), (10 + n*11) as u128);
                assert_eq!(S.unlocked(&Bob, t),   0u128);
            }
        }
        // ...last portion and remainder.
        let t_remainder = A.start_at + A.duration;
        for t in t_remainder..t_remainder + A.duration {
            assert_eq!(S.unlocked(&Alice, t), 100);
            assert_eq!(S.unlocked(&Bob, t), 0);
        }
    }
}
