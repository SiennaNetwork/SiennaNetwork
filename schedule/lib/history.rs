//! Model of vesting history.

use crate::units::*;
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

pub type Claim = (HumanAddr, Seconds, Uint128);

/// Log of executed claims
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct History {
    pub history: Vec<Claim>
}
impl History {
    /// How much has been claimed by address `a` at time `t`
    pub fn claimed (&self, a: &HumanAddr, t: Seconds) -> u128 {
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
    let log = History { history: (vec![(alice.clone(), 100, 100u128.into())
                                     ,(bobby.clone(), 100, 200u128.into())
                                     ,(alice.clone(), 200, 300u128.into())]) };
    assert_eq!(log.claimed(&alice,   0),   0);
    assert_eq!(log.claimed(&alice,   1),   0);
    assert_eq!(log.claimed(&alice, 100), 100);
    assert_eq!(log.claimed(&alice, 101), 100);
    assert_eq!(log.claimed(&alice, 200), 400);
    assert_eq!(log.claimed(&alice, 999), 400);
    assert_eq!(log.claimed(&bobby, 999), 200);
    assert_eq!(log.claimed(&bobby,  99),   0);

    todo!("history records count of portions claimed in order to support varying amounts")
}
