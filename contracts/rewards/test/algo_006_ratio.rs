use crate::test::*;

/// Given an instance with 0/1 ratio
///
///  When user becomes eligible for rewards
///  Then rewards are zero
///
///  When ratio is set to 1/2
///  Then rewards are halved
///
///  When ratio is set to 1/1
///  Then rewards are normal
#[test] fn test_global_ratio () {
    Context::named("algo_006_ratio")
        .admin()
            .at(1).init().fund(100u128).set_ratio((0u128, 1u128))
        .user("Alice")
            .tick().deposits(100u128)
            .tick().must_wait(86399)
            .tick().must_wait(86398)
            .at(86401).must_wait(1)
            .tick().ratio_is_zero()
        .admin()
            .tick().set_ratio((1u128, 2u128))
        .user("Alice")
            .tick().claims(50u128)
        .admin()
            .tick().set_ratio((1u128, 1u128))
                   .fund(100u128)
        .user("Alice")
            .at(86402*2).claims(150u128);
}
