use crate::test::*;

#[test] fn test_sequential () {

    Context::named("algo_021_sequential")
        .admin().at(1).init()
        .fund(100u128)
        .user("Alice")
            .at(2).deposits(100u128)
            .at(86402).withdraws(100u128).claims(100u128)
        .fund(100u128)
        .user("Bob")
            .at(86402).deposits(100u128)
            .at(86400*2+2).withdraws(100u128).claims(100u128);

}
