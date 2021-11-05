use crate::test::*;

/// Given a pool with some activity
///
///  When someone unauthorized tries to close the pool
///  Then they can't
///
///  When the admin closes the pool
///  Then the pool is closed
///   And every user transaction returns all LP tokens to the user
#[test] fn test_close () {
    for msg in [
        RewardsHandle::Lock     { amount: 100u128.into() },
        RewardsHandle::Retrieve { amount: 100u128.into() },
    ] {
        let mut context = Context::named("algo_004_close");
        let return_funds = context.lp_token
            .transfer(&HumanAddr::from("Alice"), 200u128.into());
        context
            .admin().init().fund(100u128)
            .later().user("Alice").deposits(100u128)
            .later().badman().cannot_close_pool()
            .later().user("Alice").deposits(100u128)
            .later().admin().closes_pool()
            // always retrieval, optionally claim transfer
            .later().user("Alice").test_handle(
                msg,
                HandleResponse::default()
                    .msg(return_funds.unwrap()).unwrap()
                    .log("closed", "5 closed"));
    }
}
