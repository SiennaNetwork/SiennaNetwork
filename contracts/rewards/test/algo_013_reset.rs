use crate::test::*;

#[test] fn test_reset () {

    let mut context = Context::new();
    let stake   = context.rng.gen_range(0..100000)*2;
    let reward  = context.rng.gen_range(0..100000);

    // Given a pool and a user
    // When the user claims reward
    // Then the user can withdraw stake
    Context::named("algo_013_reset_a")
        .admin().init().fund(reward)
        .user("Alice").set_vk("")
            .later().deposits(stake)
            .epoch().claims(reward).withdraws(stake).volume(0);

    // Given a pool and a user
    // When the user withdraws full stake
    // Then the user auto claims
    Context::named("algo_013_reset_b")
        .admin().init().fund(reward)
        .user("Alice").set_vk("")
            .later().deposits(stake)
            .epoch().withdraws_claims(stake, reward).volume(0);

    // Given a pool and a user
    // When the user withdraws partial stake
    // Then the user doesn't auto claim
    // When the user withdraws the rest of the stake
    // Then the user auto claims
    Context::named("algo_013_reset_c")
        .admin().init().fund(reward)
        .user("Alice").set_vk("")
            .later().deposits(stake)
            .epoch().withdraws(stake/2)
            .later().withdraws_claims(stake/2, reward);

}
