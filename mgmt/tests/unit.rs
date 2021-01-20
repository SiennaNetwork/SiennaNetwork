use cosmwasm_std::{
    coins, from_binary,
    StdResult, StdError,
    HumanAddr, Coin,
    Extern, MemoryStorage
};

use cosmwasm_std::testing::{
    mock_dependencies_with_balances, mock_env,
    MockApi, MockQuerier
};

use sienna_mgmt as mgmt;

fn harness (balances: &[(&HumanAddr, &[Coin])])
-> Extern<MemoryStorage, MockApi, MockQuerier> {
    let mut deps = mock_dependencies_with_balances(20, &balances);

    // As the admin
    // When I init the contract
    // Then I want to be able to query its state
    let res = mgmt::init(
        &mut deps,
        mock_env("Alice", &coins(1000, "SIENNA")),
        mgmt::msg::InitMsg { token: None }
    ).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

macro_rules! query {
    (
        $deps:ident $Query:ident
        ($res:ident: $Response:ident) $Assertions:block
    ) => {
        let $res: mgmt::msg::$Response = from_binary(
            &mgmt::query(&$deps, mgmt::msg::QueryMsg::$Query {}).unwrap()
        ).unwrap();
        $Assertions
    }
}

macro_rules! tx {
    ($deps:ident $env:ident $Msg:ident) => {
        let _ = mgmt::handle(&mut $deps, $env, mgmt::msg::HandleMsg::$Msg {});
    }
}

#[test] fn init () {
    let mut deps = harness(&[
        (&HumanAddr::from("Alice"),   &coins(1000, "SIENNA")),
    ]);

    // When  I init the contract
    // Then  I should become admin
    // And   I should be able to query its state
    // And   it should not be launched
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, None)
    });
}

#[test] fn launch () {

    let mut deps = harness(&[
        (&HumanAddr::from("Alice"),   &coins(1000, "SIENNA")),
        (&HumanAddr::from("Mallory"), &coins(   0, "SIENNA"))
    ]);

    // Given the contract IS NOT YET launched

    // As    a stranger
    // When  I try to launch the contract
    // Then  I should fail
    let env = mock_env("Mallory", &coins(0, "SIENNA"));
    tx!(deps env Launch);
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, None)
    });

    // As    the admin
    // When  I launch the contract
    // Then  it should remember when it was first launched
    let env = mock_env("Alice", &coins(1000, "SIENNA"));
    let time1 = env.block.time;
    tx!(deps env Launch);
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, Some(time1))
    });

    // Given the contract IS ALREADY launched

    // As    the admin
    // When  I launch the contract
    // Then  it should say it's already launched
    // And   it should not update its launch date
    let env = mock_env("Alice", &coins(1000, "SIENNA"));
    let time2 = env.block.time;
    assert!(time2 != time1);
    tx!(deps env Launch);
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, Some(time1))
    });
}

#[test] fn configure () {

    let mut deps = harness(&[
        (&HumanAddr::from("Alice"),   &coins(1000, "SIENNA")),
        (&HumanAddr::from("Bob"),     &coins(   0, "SIENNA")),
        (&HumanAddr::from("Mallory"), &coins(   0, "SIENNA"))
    ]);

    // Given the contract IS NOT YET launched

    // As    the admin
    // When  I add a recipient
    // Then  I should be able to fetch it
    let env = mock_env("Alice", &coins(1000, "SIENNA"));

    // As    a stranger
    // When  I try to add a recipient
    // Then  I should be denied access
    let env = mock_env("Mallory", &coins(0, "SIENNA"));

    // As    the admin
    // When  I add a recipient with the same address as an existing one
    // Then  I will have updated that recipient's parameters
    let env = mock_env("Alice", &coins(1000, "SIENNA"));

    // As    a stranger
    // When  I try to modify a recipient
    // Then  I should be denied access
    let env = mock_env("Mallory", &coins(0, "SIENNA"));

    // As    the admin
    // When  I delete a recipient
    // Then  I that recipient should no longer be on the list
    let env = mock_env("Alice", &coins(1000, "SIENNA"));

    // As    a stranger
    // When  I try to delete a recipient
    // Then  I should be denied access
    let env = mock_env("Mallory", &coins(0, "SIENNA"));

    // Given the contract IS ALREADY launched
    // As    the admin
    // When  I try to modify recipients
    // Then  I should be denied access

    // As    a stranger
    // When  I try to modify recipients
    // Then  I should be denied access
}

/*
 *
 * TODO . . . (maybe there's already a library for this?)

    given!("the contract is not yet launched"
        as "a stranger" [ 0 SIENNA, 0 SCRT ] {
            when "I try to launch the contract" {
                tx Launch;
            }
            then "I should fail" {
                q StatusQuery (res: StatusResponse) {
                    assert_eq!(res.launched, None)
                };
            }
        }
        as "the admin" [ 1000 SIENNA, 1000 SCRT ] {
            when "I launch the contract" {
                let time1 = env.block.time;
                tx Launch;
            }
            then "it should remember that moment" {
                q StatusQuery (res: StatusResponse) {
                    assert_eq!(res.launched, time1)
                };
            }
            let time2 = env.block.time;
            assert(time1 != time2);
            tx Launch;
            q StatusQuery (res: StatusResponse) {
                assert_eq!(res.launched, time2)
            };
        }
    );

    given!("the contract is already launched"
        as "the admin" [ 1000 SIENNA, 1000 SCRT ] {
            when "I try to launch the contract again" {}
            then "it should say it's already launched" {}
            and  "it should not update its launch date" {}
        }
    );

*/
