use crate::types::{Uint128, HumanAddr, Seconds, Amount};

pub type Claim = (HumanAddr, Seconds, Uint128);

/// Log of executed claims
pub struct History {
    history: Vec<Claim>
}
impl History {
    pub fn new (history: Vec<Claim>) -> Self { Self { history } }

    /// How much has been claimed by address `a` at time `t`
    pub fn claimed (&self, a: &HumanAddr, t: Seconds) -> Amount {
        let mut sum = 0;
        for (addr, time, amount) in self.history.iter().rev() {
           if addr != a { continue }
           if *time > t { continue }
           sum += amount.u128();
        }
        sum
    }
}

#[test]
fn test_claimed () {
    let alice = HumanAddr::from("alice");
    let bobby = HumanAddr::from("bob");
    let log = History::new(vec![(alice.clone(), 100, 100u128.into())
                               ,(bobby.clone(), 100, 200u128.into())
                               ,(alice.clone(), 200, 300u128.into())]);
    assert_eq!(claimed(&alice, &log,   0),   0);
    assert_eq!(claimed(&alice, &log,   1),   0);
    assert_eq!(claimed(&alice, &log, 100), 100);
    assert_eq!(claimed(&alice, &log, 101), 100);
    assert_eq!(claimed(&alice, &log, 200), 400);
    assert_eq!(claimed(&alice, &log, 999), 400);
    assert_eq!(claimed(&bobby, &log, 999), 200);
    assert_eq!(claimed(&bobby, &log,  99),   0);
}
