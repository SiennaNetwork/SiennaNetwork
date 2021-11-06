//use crate::test::*;

/// Given two instances
///
///  When a user tries to initiate a migration
///  Then they fail
///
///  When the admin calls SetMigrationStatus
///   And passes the address of the new contract
///  Then the old contract is in migration mode
///
///  When a user calls ImportState on the new contract
///  Then the new contract fetches data from the old one
#[test] fn test_migration () {
}
