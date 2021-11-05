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
        let reward = context.rng.gen_range(0..100000);
        let stake1 = context.rng.gen_range(0..100000);
        let stake2 = context.rng.gen_range(0..100000);
        let return_funds = context.lp_token
            .transfer(&HumanAddr::from("Alice"), (stake1+stake2).into());
        context
            .admin().init().fund(reward)
            .later().user("Alice").deposits(stake1)
            .later().badman().cannot_close_pool()
            .later().user("Alice").deposits(stake2)
            .later().admin().closes_pool()
            // always retrieval, optionally claim transfer
            .later().user("Alice").test_handle(
                msg,
                HandleResponse::default()
                    .msg(return_funds.unwrap()).unwrap()
                    .log("closed", "5 closed"));
    }
}
