#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

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

        let time = 2;

        let _ = tx(&mut deps,
            mock_env(1, time, &MALLORY, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});

        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, None) });

    }

    when "the admin tries to start the vesting"
    then "the contract should remember that moment" {

        let time = 3;

        let _ = tx(&mut deps, mock_env(1, time, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});

        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, Some(time)) });

    }

    given "the contract is already launched"
    when "the admin tries to start the vesting"
    then "the contract should say it's already launched"
    and "it should not update its launch date" {

        let next_time = 4;

        let _ = tx(&mut deps,
            mock_env(3, next_time, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});

        query!(Status(deps)->Status(launched)
            { assert_eq!(launched, Some(time)) });

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

        let r1 = vec![(canon!(deps, &BOB), 100)];

        let _ = tx(&mut deps, mock_env(1, 1, &ALICE, coins(10, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r1.clone() });

        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, r1) });

    }

    when "the admin tries to set the recipients above the total"
    then "an error should be returned"
    and  "the recipients should not be updated" {
        let recipients = vec![(canon!(deps, &BOB), 10000000)];
        assert_eq!(

            tx(&mut deps,
                mock_env(4, 4, &ALICE, coins(0, "SIENNA")),
                mgmt::msg::Handle::SetRecipients { recipients }),

            Err(cosmwasm_std::StdError::GenericErr {
                msg: mgmt::strings::err_allocation(0, 0),
                backtrace: None
            })

        );
        todo!();
    }

    when "a stranger tries to set the recipients"
    then "they should not be able to" {

        let r2 = vec![(canon!(deps, &MALLORY), 100)];

        let _ = tx(&mut deps,
            mock_env(1, 1, &MALLORY, coins(10, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r2 });

        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, r1) });

    }

    given "the contract is already launched" {
        let _ = tx(&mut deps, mock_env(2, 2, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
    }

    when "the admin tries to set the recipients"
    then "the recipients should be updated" {

        let r3 = vec![(canon!(deps, &BOB), 200)];

        let _ = tx(&mut deps,
            mock_env(3, 3, &ALICE, coins(1000, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r3 });

        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, r3) });

    }

    when "the admin sets the recipients above the total"
    then "an error should be returned" {
        todo!();
    }

    when "a stranger tries to set the recipients"
    then "an error should be returned" {

        let r4 = vec![(canon!(deps, &MALLORY), 100)];

        let _ = tx(&mut deps,
            mock_env(4, 4, &MALLORY, coins(0, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r4 });

        query!(Recipients(deps)->Recipients(recipients)
            { assert_eq!(recipients, r3) });

    }

    #[claim]

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

    when "a stranger tries to claim funds"
    then "they should be denied" {
        assert_eq!(

            tx(&mut deps,
                mock_env(4, 4, &MALLORY, coins(0, "SIENNA")),
                mgmt::msg::Handle::Claim {}),

            Err(cosmwasm_std::StdError::GenericErr {
                msg: mgmt::strings::PRELAUNCH.to_string(),
                backtrace: None
            })

        );
    }

    when "a claimant tries to claim funds"
    then "they should be denied" {
        assert_eq!(

            tx(&mut deps,
                mock_env(4, 4, &BOB, coins(0, "SIENNA")),
                mgmt::msg::Handle::Claim {}),

            Err(cosmwasm_std::StdError::GenericErr {
                msg: mgmt::strings::PRELAUNCH.to_string(),
                backtrace: None
            })

        );
    }

    given "the contract is already launched" {
        let _ = tx(
            &mut deps,
            mock_env(1, 1, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
    }

    when "a stranger tries to claim funds"
    then "they should be denied" {
        assert_eq!(

            tx(&mut deps,
                mock_env(4, 4, &MALLORY, coins(0, "SIENNA")),
                mgmt::msg::Handle::Claim {}),

            Err(cosmwasm_std::StdError::GenericErr {
                msg: mgmt::strings::NOTHING.to_string(),
                backtrace: None
            })

        );
    }

    when "a claimant tries to claim funds"
    and  "the claimant is eligible"
    then "the contract should transfer them to their address"
    and  "it should remember how much that address has claimed so far" {
        assert_eq!(

            tx(&mut deps,
                mock_env(4, 4, &BOB, coins(0, "SIENNA")),
                mgmt::msg::Handle::Claim {}),

            Ok(cosmwasm_std::HandleResponse {
                data:     None,
                log:      vec![],
                messages: vec![
                    cosmwasm_std::CosmosMsg::Bank(
                        cosmwasm_std::BankMsg::Send {
                            from_address: HumanAddr::from("contract"),
                            to_address:   BOB,
                            amount:       coins(100, "SIENNA")
                        }
                    )
                ]
            })

        );
        todo!();
    }

    when "a claimant tries to claim funds"
    and  "the claimant has already claimed within this time period"
    then "the contract should tell the user there's nothing at this time" {
        assert_eq!(

            tx(&mut deps,
                mock_env(4, 4, &BOB, coins(0, "SIENNA")),
                mgmt::msg::Handle::Claim {}),

            Err(cosmwasm_std::StdError::GenericErr {
                msg: mgmt::strings::NOTHING.to_string(),
                backtrace: None
            })

        )
    }

);
