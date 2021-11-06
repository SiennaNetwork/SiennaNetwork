use crate::test::*;

/// Given a pool with some activity
///
///  When someone unauthorized tries to close the pool
///  Then they can't
///
///  When the admin closes the pool
///  Then the pool is closed
///   And every user transaction returns all LP tokens to the user
#[test] fn test_0300_close () {
    let mut context  = Context::named("test_0300_close");
    let reward: u128 = context.rng.gen_range(0..100000);
    let stake1: u128 = context.rng.gen_range(0..100000);
    let stake2: u128 = context.rng.gen_range(0..100000);
    context
        .admin().init().fund(reward)
        .later().badman().cannot_close_pool()
        .later().user("Alice").deposits(stake1)
        .later().badman().cannot_close_pool()
        .later().user("Alice").deposits(stake2)
        .later().badman().cannot_close_pool()
        .later().admin().closes_pool();
}

/// Given a pool with some activity
///
///  When someone unauthorized tries to close the pool
///  Then they can't
///
///  When the admin closes the pool
///  Then the pool is closed
///   And every user transaction returns all LP tokens to the user
#[test] fn test_0301_lock_after_close () {
    let mut context  = Context::named("test_0301_lock_after_close");
    let reward: u128 = context.rng.gen_range(0..100000);
    let stake1: u128 = context.rng.gen_range(0..100000);
    let stake2: u128 = context.rng.gen_range(0..100000);
    let return_funds = context.lp_token.transfer(&HumanAddr::from("Alice"), (stake1+stake2).into());
    context
        .admin().init().fund(reward)
        .later().user("Alice").deposits(stake1)
        .later().user("Alice").deposits(stake2)
        .later().admin().closes_pool();
    let (ref when, ref why) = context.closed.clone().unwrap();
    context
        .later().user("Alice").test_handle(
            RewardsHandle::Lock { amount: 100u128.into() },
            HandleResponse::default()
                .msg(return_funds.unwrap()).unwrap()
                .log("close_time",   &format!("{}", when)).unwrap()
                .log("close_reason", why));
}

/// Given a pool with some activity
///
///  When someone unauthorized tries to close the pool
///  Then they can't
///
///  When the admin closes the pool
///  Then the pool is closed
///   And every user transaction returns all LP tokens to the user
#[test] fn test_0302_retrieve_after_close () {
    let mut context  = Context::named("test_0302_retrieve_after_close");
    let reward: u128 = context.rng.gen_range(0..100000);
    let stake1: u128 = context.rng.gen_range(0..100000);
    let stake2: u128 = context.rng.gen_range(0..100000);
    let return_funds = context.lp_token.transfer(&HumanAddr::from("Alice"), (stake1+stake2).into());
    context
        .admin().init().fund(reward)
        .later().user("Alice").deposits(stake1)
        .later().user("Alice").deposits(stake2)
        .later().admin().closes_pool();
    let (ref when, ref why) = context.closed.clone().unwrap();
    context
        .later().user("Alice").test_handle(
            RewardsHandle::Retrieve { amount: 100u128.into() },
            HandleResponse::default()
                .msg(return_funds.unwrap()).unwrap()
                .log("close_time",   &format!("{}", when)).unwrap()
                .log("close_reason", why));
}

/// Given a pool with some activity
///
///  When someone unauthorized tries to close the pool
///  Then they can't
///
///  When the admin closes the pool
///  Then the pool is closed
///   And every user transaction returns all LP tokens to the user
#[test] fn test_0303_claim_after_close () {
    let mut context  = Context::named("test_0303_claim_after_close");
    let reward: u128 = context.rng.gen_range(0..100000);
    let stake1: u128 = context.rng.gen_range(0..100000);
    let stake2: u128 = context.rng.gen_range(0..100000);
    let return_funds = context.lp_token.transfer(&HumanAddr::from("Alice"), (stake1+stake2).into());
    context
        .admin().init().fund(reward)
        .later().user("Alice").deposits(stake1)
        .later().user("Alice").deposits(stake2)
        .later().admin().closes_pool();
    let (ref when, ref why) = context.closed.clone().unwrap();
    context
        .later().user("Alice").test_handle(
            RewardsHandle::Claim {},
            HandleResponse::default()
                .msg(return_funds.unwrap()).unwrap()
                .log("close_time",   &format!("{}", when)).unwrap()
                .log("close_reason", why));
}
