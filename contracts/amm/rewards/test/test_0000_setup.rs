use crate::drain::WAIT_PERIOD;
use crate::test::{Context, *};

#[test]
fn test_0001_init() {
    // Given no instance
    Context::new("0001_init")
        .admin()
        .later()
        // When  the admin inits an instance without providing a reward token
        // Then  the init fails
        .branch("invalid", |mut context| {
            context.init_invalid();
        })
        // When  the admin inits an instance with a configured reward token
        // Then  the default values are used where applicable
        // And   the rewards module emits a message that sets the reward token viewing key
        .branch("ok", |mut context| {
            context.init();
        });
}

#[test]
fn test_0002_config() {
    // Given no instance
    Context::new("0002_config")
        .admin()
        .init()
        .later()
        // When someone else tries to set the config
        // Then the config remains unchanged
        .branch("config_auth_fail", |mut context| {
            context.badman().cannot_configure();
        })
        // When the admin sets the config, including a reward token
        // Then a reward token viewing key config message is returned
        .branch("config_auth_ok", |mut context| {
            let reward_token = context.reward_token.clone().link;
            let reward_vk = context.reward_vk.clone();
            context.admin().configures(RewardsConfig {
                lp_token: None,
                reward_token: Some(reward_token),
                reward_vk: Some(reward_vk),
                bonding: None,
                timekeeper: None,
            });
        });
}

#[test]
fn test_0003_drain() {
    // Given an instance
    Context::new("0003_drain")
        .init()
        .fund(100)
        .later()
        // When non admin-tries to call release
        // Then gets rejected
        .branch("auth_fail", |mut context| {
            context.badman().cannot_drain("key");
        })
        // When calling with reward token info
        // Then the viewing key changes
        .branch("auth_ok", |mut context| {
            context
                .admin()
                .cannot_drain("key")
                .closes_pool()
                .cannot_drain("key")
                .after(WAIT_PERIOD)
                .drains_pool("key");
        });
}
