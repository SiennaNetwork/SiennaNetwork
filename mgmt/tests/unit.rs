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
        let Alice: HumanAddr = "Alice".into();
        let mut deps = harness(&[(&Alice, &coins(1000, "SIENNA")),]);
    }

    when "someone deploys the contract" {
        let res = mgmt::init(
            &mut deps,
            mock_env(0, 0, &Alice, coins(1000, "SIENNA")),
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
        let Alice:   HumanAddr = "Alice".into();
        let Mallory: HumanAddr = "Mallory".into();
        let mut deps = harness(&[
            (&Alice,   &coins(1000, "SIENNA")),
            (&Mallory, &coins(   0, "SIENNA"))
        ]);
    }

    when "a stranger tries to start the vesting"
    then "they should fail" {
        let env = mock_env(1, 1, &Mallory, coins(0, "SIENNA"));
        println!("{:#?}", env);
        tx!(deps env Launch);
        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, None) });
    }

    when "the admin tries to start the vesting"
    then "the contract should remember that moment" {
        let env = mock_env(2, 2, &Alice, coins(0, "SIENNA"));
        tx!(deps env Launch);
        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, Some(2)) });
    }

    given "the contract is already launched"
    when "the admin tries to start the vesting"
    then "the contract should say it's already launched"
    and "it should not update its launch date" {
        let env = mock_env(3, 3, &Alice, coins(0, "SIENNA"));
        tx!(deps env Launch);
        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, Some(2)) });
    }

    #[configure]

    given "the contract is not yet launched" {
        let Alice:   HumanAddr = "Alice".into();
        let Bob:     HumanAddr = "Bob".into();
        let Mallory: HumanAddr = "Mallory".into();
        let mut deps = harness(&[
            (&Alice,   &coins(1000, "SIENNA")),
            (&Bob,     &coins(   0, "SIENNA")),
            (&Mallory, &coins(   0, "SIENNA"))
        ]);
    }

    when "the admin sets the recipients"
    then "they should be updated" {
        let env = mock_env(1, 1, &Alice, coins(0, "SIENNA"));
        let r = vec![(Bob, 100)];
        tx!(deps env Launch);
        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, []) });
    }

    when "a stranger tries to set the recipients" 
    then "they should be denied access" {
        let env = mock_env(1, 1, &Mallory, coins(0, "SIENNA"));
        let recipients = vec![(&Mallory, 100)];
        tx!(deps env Launch);
        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, []) });
    }

    given "the contract is already launched" {
        let env = mock_env(2, 2, &Alice, coins(0, "SIENNA"));
        tx!(deps env Launch);
    }

    when "the admin tries to set the recipients" {
        let env = mock_env(3, 3, &Alice, coins(1000, "SIENNA"));
    }
    then "they should be denied access" { todo!() }

    when "a stranger tries to set the recipients" {
        let env = mock_env(4, 4, &Mallory, coins(0, "SIENNA"));
    }
    then "they should be denied access" { todo!() }

    #[claim]
    given "the contract is not yet launched"

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
