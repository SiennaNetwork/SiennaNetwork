use crate::types::*;

/// Log of executed claims
pub type FulfilledClaims = Vec<(Address, Time, Amount)>;

pub fn at (
    a:      &Address,
    claims: &FulfilledClaims,
    t:      Time
) -> Amount {
    for (addr, time, amount) in claims.iter().rev() {
       if addr != a { continue }
       if *time > t { continue }
       return *amount
    }
    0
}


