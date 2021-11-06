use crate::test::*;

#[test] fn test_0001_init () {
    // Given no instance
    // When  the admin inits an instance with a configured reward token
    // Then  the default values are used where applicable
    // And   the rewards module emits a message that sets the reward token viewing key
    Context::named("test_0001_init").admin().later().init();
}

#[test] fn test_0002_init_invalid () {
    // Given no instance
    // When  the admin inits an instance without providing a reward token
    // Then  the init fails
    Context::named("test_0002_init_invalid").admin().later().init_invalid();

}

#[test] fn test_0003_configure () {
    let Context { ref reward_token, ref reward_vk, .. } = Context::new();
    // Given no instance
    Context::named("test_0003_configure")
        // When the admin inits an instance with an empty configuration
        // Then the default values are used where applicable
        // And  no viewing key config message is returned
        .admin().init()
        // When someone else tries to set the config
        // Then the config remains unchanged
        .later().badman().cannot_configure()
        // When the admin sets the config, including a reward token
        // Then a reward token viewing key config message is returned
        .later().admin().configures(RewardsConfig {
            lp_token:     None,
            reward_token: Some(reward_token.link.clone()),
            reward_vk:    Some(reward_vk.clone()),
            bonding:      None,
        });
}

/// Given an instance
///
///  When non admin-tries to call release
///  Then gets rejected
///
///  When calling with reward token info
///  Then the viewing key changes
#[test] fn test_0004_drain () {
    Context::named("test_0004_drain").init().fund(100)
        .later().badman().cannot_drain("key")
        .later().admin().drains_pool("key");
}
