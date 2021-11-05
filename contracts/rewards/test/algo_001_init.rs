use crate::test::*;

#[test] fn test_init () {

    // Given no instance
    //  When the admin inits an instance without providing a reward token
    //  Then the init fails
    Context::named("algo_001_init").admin()
        .later().init_invalid()

    // Given no instance
    //  When the admin inits an instance with a configured reward token
    //  Then the default values are used where applicable
    //   And the rewards module emits a message that sets the reward token viewing key
        .later().init();

}
