use crate::test::*;

#[test] fn test_parallel () {

    Context::new()
        .admin().at(1).init().fund(100u128)
        .at(2).user("Alice").deposits(100u128)
              .user("Bob").deposits(100u128)
        .at(86402).user("Alice").withdraws(100u128).claims(50u128)
                  .user("Bob").withdraws(100u128).deposits(50u128);

}
