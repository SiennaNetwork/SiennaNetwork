#[macro_use] extern crate kukumba;
#[macro_use] mod macros;

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
        mgmt::msg::Init { token: None }
    ).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

fn mock_env (
    height: u64, time: u64, sender: &HumanAddr, sent_funds: Vec<Coin>
) -> Env { Env {
    block: BlockInfo { height, time, chain_id: "secret".into() },
    message: MessageInfo { sender: sender.into(), sent_funds },
    contract: ContractInfo { address: "contract".into() },
    contract_key: Some("".into()),
    contract_code_hash: "".into()
} }


kukumba!(

    #[init]

    given "the contract is not yet deployed" {
        let ALICE:   HumanAddr = HumanAddr::from("ALICE");
        let mut deps = harness(&[(&ALICE, &coins(1000, "SIENNA")),]);
    }

    when "someone deploys the contract" {
        let res = mgmt::init(
            &mut deps,
            mock_env(0, 0, &ALICE, coins(1000, "SIENNA")),
            mgmt::msg::Init { token: None }
        ).unwrap();
    }

    then "they become admin"
    and  "they should be able to query its state"
    and  "it should say it's not launched yet" {
        query!(Status(deps) -> Status(launched)
            { assert_eq!(launched, None) });
    }

    #[launch]

    given "the contract is not yet launched" {
        let ALICE:   HumanAddr = HumanAddr::from("ALICE");
        let MALLORY: HumanAddr = HumanAddr::from("MALLORY");
        let mut deps = harness(&[
            (&ALICE,   &coins(1000, "SIENNA")),
            (&MALLORY, &coins(   0, "SIENNA"))
        ]);
    }

    when "a stranger tries to start the vesting"
    then "they should fail" {
        let env = mock_env(1, 1, &MALLORY, coins(0, "SIENNA"));
        tx!(deps env Launch);
        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, None) });
    }

    when "the admin tries to start the vesting"
    then "the contract should remember that moment" {
        let env = mock_env(2, 2, &ALICE, coins(0, "SIENNA"));
        tx!(deps env Launch);
        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, Some(2)) });
    }

    given "the contract is already launched"
    when "the admin tries to start the vesting"
    then "the contract should say it's already launched"
    and "it should not update its launch date" {
        let env = mock_env(3, 3, &ALICE, coins(0, "SIENNA"));
        tx!(deps env Launch);
        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, Some(2)) });
    }

    #[configure]

    given "the contract is not yet launched" {
        let ALICE:   HumanAddr = HumanAddr::from("ALICE");
        let BOB:     HumanAddr = HumanAddr::from("BOB");
        let MALLORY: HumanAddr = HumanAddr::from("MALLORY");
        let mut deps = harness(&[
            (&ALICE,   &coins(1000, "SIENNA")),
            (&BOB,     &coins(   0, "SIENNA")),
            (&MALLORY, &coins(   0, "SIENNA"))
        ]);
    }

    when "the admin sets the recipients"
    then "the recipients should be updated" {
        let env = mock_env(1, 1, &ALICE, coins(10, "SIENNA"));
        let r = vec![(canon!(deps, &BOB), 100)];
        tx!(deps env SetRecipients { recipients: r.clone() });
        query!(Recipients(deps)->Recipients(recipients) {
            assert_eq!(recipients, r)
        });
    }

    when "a stranger tries to set the recipients"
    then "they should not be able to" {
        let env = mock_env(1, 1, &MALLORY, coins(10, "SIENNA"));
        let r2 = vec![(canon!(deps, &MALLORY), 100)]
        tx!(deps env SetRecipients { recipients: r2 });
        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, r) });
    }

    given "the contract is already launched" {
        let env = mock_env(2, 2, &ALICE, coins(0, "SIENNA"));
        tx!(deps env Launch);
    }

    when "the admin tries to set the recipients"
    then "they should be denied access" {
        let env = mock_env(3, 3, &ALICE, coins(1000, "SIENNA"));
        tx!(deps env SetRecipients {
            recipients: vec![(canon!(deps, &BOB), 100)]
        });
        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, r) });
    }

    when "a stranger tries to set the recipients"
    then "they should be denied access" {
        let env = mock_env(4, 4, &MALLORY, coins(0, "SIENNA"));
        tx!(deps env SetRecipients {
            recipients: vec![(canon!(deps, &MALLORY), 100)]
        });
        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, r) });
    }

    #[claim]

    given "the contract is not yet launched" {
        let ALICE:   HumanAddr = HumanAddr::from("ALICE");
        let BOB:     HumanAddr = HumanAddr::from("BOB");
        let MALLORY: HumanAddr = HumanAddr::from("MALLORY");
    }

    when "a stranger tries to claim funds"
    then "they should be denied" { todo!() }

    when "a claimant tries to claim funds" {}
    then "they should be denied" { todo!() }

    given "the contract is already launched"

    when "a stranger tries to claim funds"
    then "they should be denied" { todo!() }

    when "a claimant tries to claim funds"
    then "the contract should transfer them to their address" { todo!() }
    and  "the contract should remember how much I've claimed so far"  { todo!() }

);
