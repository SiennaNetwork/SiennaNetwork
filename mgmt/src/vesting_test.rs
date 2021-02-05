use crate::vesting::claimed;
use cosmwasm_std::HumanAddr;

#[test]
fn test_claimed () {
    let alice = HumanAddr::from("alice");
    let bobby = HumanAddr::from("bob");
    let log = vec![ (alice.clone(), 100, 100u128.into())
                  , (bobby.clone(), 100, 200u128.into())
                  , (alice.clone(), 200, 300u128.into()) ];
    assert_eq!(claimed(&alice, &log,   0),   0);
    assert_eq!(claimed(&alice, &log,   1),   0);
    assert_eq!(claimed(&alice, &log, 100), 100);
    assert_eq!(claimed(&alice, &log, 101), 100);
    assert_eq!(claimed(&alice, &log, 200), 400);
    assert_eq!(claimed(&alice, &log, 999), 400);
    assert_eq!(claimed(&bobby, &log, 999), 200);
    assert_eq!(claimed(&bobby, &log,  99),   0);
}
