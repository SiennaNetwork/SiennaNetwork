use cosmwasm_std::{
    Api, Env, BlockInfo, MessageInfo, ContractInfo,
    coins, from_binary,
    StdResult, StdError,
    HumanAddr, Coin,
    Extern, MemoryStorage
};

use cosmwasm_std::testing::{
    mock_dependencies_with_balances, /*mock_env,*/
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
        mock_env(0, 0, balances[0].0, balances[0].1.into()),
        mgmt::msg::InitMsg { token: None }
    ).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

fn mock_env (height: u64, time: u64, sender: &HumanAddr, sent_funds: Vec<Coin>)
    -> Env {
    Env {
        block: BlockInfo { height, time, chain_id: "secret".into() },
        message: MessageInfo { sender: sender.into(), sent_funds },
        contract: ContractInfo { address: "contract".into() },
        contract_key: Some("".into()),
        contract_code_hash: "".into()
    }
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
    (
        $deps:ident $env:ident
        $Msg:ident $({ $($arg:ident : $val:expr),* })?
    ) => {
        let msg = mgmt::msg::HandleMsg::$Msg { $($($arg:$val)*)? };
        let _ = mgmt::handle(&mut $deps, $env, msg);
    }
}

#[test] fn init () {

    let alice: HumanAddr = "Alice".into();
    let mut deps = harness(&[(&alice, &coins(1000, "SIENNA")),]);

    // When  I init the contract
    // Then  I should become admin
    // And   I should be able to query its state
    // And   it should not be launched
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, None)
    });
}

#[test] fn launch () {

    let alice:   HumanAddr = "Alice".into();
    let mallory: HumanAddr = "Mallory".into();
    let mut deps = harness(&[
        (&alice,   &coins(1000, "SIENNA")),
        (&mallory, &coins(   0, "SIENNA"))
    ]);

    // Given the contract IS NOT YET launched

    // As    a stranger
    // When  I try to launch the contract
    // Then  I should fail
    let env = mock_env(1, 1, &mallory, coins(0, "SIENNA"));
    println!("{:#?}", env);
    tx!(deps env Launch);
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, None)
    });

    // As    the admin
    // When  I launch the contract
    // Then  it should remember when it was first launched
    let env = mock_env(2, 2, &alice, coins(0, "SIENNA"));
    tx!(deps env Launch);
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, Some(2))
    });

    // Given the contract IS ALREADY launched

    // As    the admin
    // When  I launch the contract
    // Then  it should say it's already launched
    // And   it should not update its launch date
    let env = mock_env(3, 3, &alice, coins(0, "SIENNA"));
    tx!(deps env Launch);
    query!(deps StatusQuery (res: StatusResponse) {
        assert_eq!(res.launched, Some(2))
    });
}

#[test] fn configure () {

    let alice:   HumanAddr = "Alice".into();
    let bob:     HumanAddr = "Bob".into();
    let mallory: HumanAddr = "Mallory".into();
    let mut deps = harness(&[
        (&alice,   &coins(1000, "SIENNA")),
        (&bob,     &coins(   0, "SIENNA")),
        (&mallory, &coins(   0, "SIENNA"))
    ]);

    // Given the contract IS NOT YET launched

    // As    the admin
    // When  I set the recipients
    // Then  I should be able to fetch them
    let env = mock_env(1, 1, &alice, coins(1000, "SIENNA"));
    tx!(deps env SetRecipients { recipients: vec![
        mgmt::Recipient {
            address:  deps.api.canonical_address(&bob).unwrap(),
            cliff:    0,
            vestings: 10,
            interval: 10,
            claimed:  0
        }
    ] });

    // As    a stranger
    // When  I try to set the recipients
    // Then  I should be denied access
    let env = mock_env(2, 2, &mallory, coins(0, "SIENNA"));

    // Given the contract IS ALREADY launched

    // As    the admin
    // When  I try to set the recipients
    // Then  I should be denied access
    let env = mock_env(3, 3, &alice, coins(1000, "SIENNA"));

    // As    a stranger
    // When  I try to set the recipients
    // Then  I should be denied access
    let env = mock_env(4, 4, &mallory, coins(0, "SIENNA"));
}

#[test] fn claim () {

    let alice:   HumanAddr = "Alice".into();
    let bob:     HumanAddr = "Bob".into();
    let mallory: HumanAddr = "Mallory".into();
    let mut deps = harness(&[
        (&alice,   &coins(1000, "SIENNA")),
        (&bob,     &coins(   0, "SIENNA")),
        (&mallory, &coins(   0, "SIENNA"))
    ]);

    // Given the contract IS NOT YET launched
    // As    a stranger
    // When  I try to claim funds
    // Then  I should be denied
    // As    a claimant
    // When  I try to claim funds
    // Then  I should be denied

    // Given the contract IS ALREADY launcher
    // As    a stranger
    // When  I try to claim funds
    // Then  I should be denied
    // As    a claimant
    // When  I try to claim funds
    // Then  the contract should transfer them to my address
    // And   the contract should update how much I've claimed
}

/*

kukumba!(

    given "the contract is not yet launched"

        as "a stranger" [ 0 SIENNA, 0 SCRT ]
            when "I try to launch the contract"
                tx Launch;
            then "I should fail"
                q StatusQuery (res: StatusResponse)
                    assert_eq!(res.launched, None)

        as "the admin" [ 1000 SIENNA, 1000 SCRT ]
            when "I launch the contract" {
                let time1 = env.block.time;
                tx Launch;
            }
            then "it should remember that moment"
                q StatusQuery (res: StatusResponse)
                    assert_eq!(res.launched, time1)

    given "the contract is already launched"

        as "the admin" [ 1000 SIENNA, 1000 SCRT ]
            when "I try to launch the contract again" {}
            then "it should say it's already launched" {}
            and  "it should not update its launch date" {}

);

*/
