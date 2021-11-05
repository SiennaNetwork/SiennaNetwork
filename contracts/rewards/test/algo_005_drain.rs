use crate::test::*;

/// Given an instance
///
///  When non admin-tries to call release
///  Then gets rejected
///
///  When calling with reward token info
///  Then the viewing key changes
#[test] fn test_drain () {
    Context::named("algo_005_drain")
        .admin() .at(1).init().fund(100u128)
        .badman().at(2).cannot_drain("key")
        .admin() .at(3).drains_pool("key");
}
