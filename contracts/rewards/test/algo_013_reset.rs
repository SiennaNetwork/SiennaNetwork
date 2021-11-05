use crate::test::*;

/// Given a pool and a user
///
///  When user deposits tokens and becomes eligible for rewards
///   And user first claims rewards and then withdraws all tokens
///  Then user volume is preserved so they can re-stake and continue
///
///  When user deposits tokens and becomes eligible for rewards
///   And user first withdraws all tokens and then claims rewards
///  Then user volume and claimed is reset so they can start over
#[test] fn test_reset () {

    Context::new()
        .admin()
            .at(1).init().fund(100u128)
        .user("Alice")
            .set_vk("")
            .at(    2).deposits(100u128)
            .at(86402).claims(100u128)
            .at(86402).withdraws(100u128).volume(200u128).claimed(100u128);

    Context::new()
        .admin()
            .at(1).init().fund(100u128)
            .user("Alice")
                .set_vk("")
                .at(    2).deposits(100u128)
                .at(86402).withdraws(100u128)
                .at(86402).claims(100u128).volume(0u128).claimed(0u128);

}
