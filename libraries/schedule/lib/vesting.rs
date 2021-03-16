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
        let mut amount = 0u128;
        let mut t_cursor = self.start_at;
        if self.cliff > Uint128::zero() {
            amount += self.cliff.u128();
            t_cursor += self.interval;
        }
        if self.interval > 0 {
            let t_end = self.start_at + self.duration;
            loop {
                if t_cursor >= t_end {
                    amount += self.remainder();
                    break
                }
                if t_cursor > t_query {
                    break
                }
                amount += self.portion_size();
                t_cursor += self.interval;
            }
        } else {
            amount += self.portion_size()
        }
        amount
    }
}
impl Account {
    pub fn amount_sans_cliff (&self) -> u128 {
        self.amount.u128() - self.cliff.u128()
    }
    /// 1 if immediate, or `duration/interval` if periodic.
    /// Returns error if `duration` is not a multiple of `interval`.
    pub fn portion_count (&self) -> u128 {
        if self.interval == 0 {
            0
        } else {
            self.amount_sans_cliff() / self.interval as u128
        }
    }
    /// Full `amount` if immediate, or `(amount-cliff)/portion_count` if periodic.
    /// Returns error if amount can't be divided evenly in that number of portions.
    pub fn portion_size (&self) -> u128 {
        if self.interval == 0 {
            self.amount_sans_cliff()
        } else {
            self.amount_sans_cliff() / self.portion_count()
        }
    }
    /// If `(amount-cliff)` doesn't divide evenly by `portion_size`,
    /// the remainder is added to the last portion.
    pub fn remainder (&self) -> u128 {
        self.amount_sans_cliff() % self.portion_size()
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
            Schedule::new(&[ Pool::complete("", &[]) ]),
            Schedule::new(&[ Pool::complete("", &[ Account::immediate("", &Alice, 0) ]) ]),
            Schedule::new(&[ Pool::partial("", 1, &[ Account::immediate("", &Alice, 0) ]) ])
        ] {
          assert_eq!(0, s.unlocked(&Alice, 0));
          assert_eq!(0, s.unlocked(&Alice, 1));
          assert_eq!(0, s.unlocked(&Bob,   1001));
        }
    }

    #[test] fn test_vest_immediate () {
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let S = Schedule::new(&[Pool::complete("", &[Account::immediate("", &Alice, 100)])]);
        assert_eq!(100, S.total.u128());
        assert_eq!(100, S.pools.get(0).unwrap().total.u128());
        assert_eq!(100, S.pools.get(0).unwrap().accounts.get(0).unwrap().amount.u128());
        for &t in &[0,1,100,1000] {
            assert_eq!(100, S.unlocked(&Alice, t));
            assert_eq!(0,   S.unlocked(&Bob,   t));
        }
    }

    #[test] fn test_vest_periodic () {
        // a periodic `Account`...
        let Alice = HumanAddr::from("Alice");
        let Bob   = HumanAddr::from("Bob");
        let A = Account::periodic("", &Alice, 100, 10, 20, 11, 90);
        assert_eq!(100, A.amount.u128());
        assert_eq!( 10, A.cliff.u128());
        assert_eq!( 90, A.amount_sans_cliff());
        assert_eq!( 20, A.start_at);
        assert_eq!( 11, A.interval);
        assert_eq!( 90, A.duration);
        assert_eq!(  8, A.portion_count());
        assert_eq!( 11, A.portion_size());
        assert_eq!(  2, A.remainder());
        // ...in a `Pool`...
        let P = Pool::complete("", &[A.clone()]);
        assert_eq!(100, P.total.u128());
        // ...in a `Schedule`.
        let S = Schedule::new(&[P.clone()]);
        assert_eq!(100, S.total.u128());

        // Before start...
        println!("\nbefore start");
        for t in 0..A.start_at {
            print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
            assert_eq!(0, S.unlocked(&Alice, t))
        }
        // ...cliff...
        println!("\n\ncliff (+{})", &A.cliff);
        for t in A.start_at..(A.start_at+A.interval) {
            print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
            assert_eq!(S.unlocked(&Alice, t), 10);
            assert_eq!(S.unlocked(&Bob, t), 0);
        }
        // ...portions...
        for n in 1..(A.portion_count() as u64) {
            let t_portion:      u64 = A.start_at + n*A.interval;
            let t_next_portion: u64 = t_portion + A.interval;
            println!("\n\nportion {} (+{})", &n, &A.portion_size());
            for t in t_portion..t_next_portion {
                print!("[@{}: {}] ", &t, &S.unlocked(&Alice, t));
                assert_eq!(S.unlocked(&Alice, t), (10 + n*11) as u128);
                assert_eq!(S.unlocked(&Bob, t),   0u128);
            }
        }
        // ...last portion and remainder.
        let t_remainder = A.start_at + A.duration;
        for t in t_remainder..t_remainder + A.duration {
            println!("\n\nlast portion {} (+{}) + remainder (+{})",
                &A.portion_count(),
                &A.portion_size(),
                &A.remainder());
            assert_eq!(S.unlocked(&Alice, t), 100);
            assert_eq!(S.unlocked(&Bob, t), 0);
        }
    }
}
