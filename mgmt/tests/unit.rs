#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};
use serde::ser::Serialize;

use sienna_mgmt as mgmt;
use mgmt::vesting::{DAY, MONTH, SCHEDULE};

use cosmwasm_std::{
    coins, StdError, HumanAddr, Api, Uint128,
    CosmosMsg, WasmMsg, Binary, to_binary, to_vec
};

use secret_toolkit::snip20;

kukumba!(

    #[init]

    given "the contract is not yet deployed" {
        let ALICE: HumanAddr = HumanAddr::from("ALICE");
        let mut deps = harness(&[(&ALICE, &coins(1000, "SIENNA")),]);
    }

    when "someone deploys the contract" {
        let res = mgmt::init(
            &mut deps,
            mock_env(0, 0, &ALICE, coins(1000, "SIENNA")),
            mgmt::msg::Init {
                token_addr: cosmwasm_std::HumanAddr::from("mgmt"),
                token_hash: String::new()
            }
        ).unwrap();
    }

    then "they become admin"
    and  "they should be able to query its state"
    and  "it should say it's not launched yet" {
        assert_query!(deps => Status => Status { launched: None });
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
        assert_query!(deps => Status => Status { launched: None });
    }

    when "the admin tries to start the vesting"
    then "the contract should remember that moment" {
        let time = 3;
        let _ = tx(&mut deps, mock_env(1, time, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
        assert_query!(deps => Status => Status { launched: Some(time) });
    }

    given "the contract is already launched"
    when "the admin tries to start the vesting"
    then "the contract should say it's already launched"
    and "it should not update its launch date" {
        let next_time = 4;
        let _ = tx(&mut deps,
            mock_env(3, next_time, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
        assert_query!(deps => Status => Status { launched: Some(time) });
    }

    #[claim_stranger]

    given "the contract is not yet launched" {
        let ALICE:   HumanAddr = HumanAddr::from("ALICE");
        let MALLORY: HumanAddr = HumanAddr::from("MALLORY");
        let mut deps = harness(&[
            (&ALICE,   &coins(1000, "SIENNA")),
            (&MALLORY, &coins(   0, "SIENNA"))
        ]);
    }

    when "a stranger tries to claim funds"
    then "they should be denied" {
        assert_tx!(deps
            => [MALLORY, SIENNA => 0] at [block 4, T=4]
            => mgmt::msg::Handle::Claim {}
            => Err(StdError::GenericErr {
                msg: mgmt::strings::PRELAUNCH.to_string(),
                backtrace: None }) );
    }

    given "the contract is already launched" {
        let _ = tx(
            &mut deps,
            mock_env(1, 1, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
    }

    when "a stranger tries to claim funds"
    then "they should be denied" {
        assert_tx!(deps
            => [MALLORY, SIENNA => 0] at [block 4, T=4]
            => mgmt::msg::Handle::Claim {}
            => Err(StdError::GenericErr {
                msg: mgmt::strings::NOTHING.to_string(),
                backtrace: None }) );
    }

    #[claim_predefined]

    given "the contract is not yet launched" {
        let ALICE: HumanAddr = HumanAddr::from("ALICE");
        let BOB:   HumanAddr = HumanAddr::from("BOB");
        let mut deps = harness(&[
            (&ALICE, &coins(1000, "SIENNA")),
            (&BOB,   &coins(   0, "SIENNA")),
        ]);

        let configured_claim_amount: Uint128 = Uint128::from(200u128);
        let r = vec![(canon!(deps, &BOB), configured_claim_amount)];
        let _ = tx(&mut deps,
            mock_env(0, 0, &ALICE, coins(1000, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r.clone() });

        assert_query!(deps => Recipients => Recipients { recipients: r });
    }

    when "a predefined claimant tries to claim funds"
    then "they should be denied" {
        match SCHEDULE.predefined.get(0).unwrap() {
            mgmt::types::Stream::Monthly {
                amount, addr, release_months, cliff_months, cliff_percent
            } => {
                assert_tx!(deps
                    => [addr, SIENNA => 0] at [block 4, T=1]
                    => mgmt::msg::Handle::Claim {}
                    => Err(StdError::GenericErr {
                        msg: mgmt::strings::PRELAUNCH.to_string(),
                        backtrace: None }) );
            },
            _ => unreachable!()
        }
    }

    given "the contract is already launched" {
        let _ = tx(
            &mut deps,
            mock_env(0, 0, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
    }

    when "a predefined claimant tries to claim funds before the cliff"
    then "they should be denied" {
        let PREDEF;
        let cliff;
        match SCHEDULE.predefined.get(0).unwrap() {
            mgmt::types::Stream::Monthly {
                amount, addr, release_months, cliff_months, cliff_percent
            } => {
                PREDEF = addr;
                cliff = cliff_months*MONTH;
                assert_tx!(deps
                    => [PREDEF, SIENNA => 0] at [block 4, T=cliff-1]
                    => mgmt::msg::Handle::Claim {}
                    => Err(StdError::GenericErr {
                        msg: mgmt::strings::NOTHING.to_string(),
                        backtrace: None }) );
            },
            _ => unreachable!()
        }
    }

    when "a predefined claimant tries to claim funds at/after the cliff"
    and  "the first post-cliff vesting has not passed"
    then "the contract should transfer the cliff amount"
    and  "it should remember how much that address has claimed so far" {
        assert_tx!(deps
            => [PREDEF, SIENNA => 0] at [block 4, T=cliff]
            => mgmt::msg::Handle::Claim {}
            => Ok(cosmwasm_std::HandleResponse {
                data:     None,
                log:      vec![],
                messages: vec![
                    snip20::handle::HandleMsg::Transfer {
                        recipient: PREDEF.clone(),
                        amount:    Uint128::from(75000u128),
                        padding:   None
                    }.to_cosmos_msg(
                        256,
                        "".to_string(),
                        HumanAddr::from("mgmt"),
                        None
                    ).unwrap()
                ] }) );
    }

    when "a predefined claimant tries to claim funds"
    and  "the claimant has already claimed within this time period"
    then "the contract should respond that there's nothing at this time" {
        assert_tx!(deps
            => [PREDEF, SIENNA => 0] at [block 6, T=cliff+1]
            => mgmt::msg::Handle::Claim {}
            => Err(StdError::GenericErr {
                msg: mgmt::strings::NOTHING.to_string(),
                backtrace: None }) );
    }

    when "a predefined claimant tries to claim funds"
    and  "enough time has passed since their last claim"
    then "the contract should transfer more funds" {
        let msg = snip20::handle::HandleMsg::Transfer {
            recipient: PREDEF.clone(),
            amount:    Uint128::from(75000u128),
            padding:   None
        }.to_cosmos_msg(
            256,
            "".to_string(),
            HumanAddr::from("mgmt"),
            None
        ).unwrap();
        assert_tx!(deps
            => [PREDEF, SIENNA => 0] at [block 4, T=cliff+1*MONTH]
            => mgmt::msg::Handle::Claim {}
            => Ok(cosmwasm_std::HandleResponse {
                data:     None,
                log:      vec![],
                messages: vec![msg.clone()] }) );
        assert_tx!(deps
            => [PREDEF, SIENNA => 0] at [block 4, T=cliff+2*MONTH]
            => mgmt::msg::Handle::Claim {}
            => Ok(cosmwasm_std::HandleResponse {
                data:     None,
                log:      vec![],
                messages: vec![msg.clone()] }) );
    }

    when "another predefined claimant tries to claim funds"
    and  "this is the first time they make a claim"
    and  "it is a long time after the end of the vesting"
    then "the contract should transfer everything in one go" {
        match SCHEDULE.predefined.get(1).unwrap() {
            mgmt::types::Stream::Daily {
                amount, addr, release_months, cliff_months, cliff_percent
            } => {
                let T = (cliff_months + release_months + 48) * MONTH;
                let msg = cosmwasm_std::CosmosMsg::Bank(
                    cosmwasm_std::BankMsg::Send {
                        from_address: HumanAddr::from("contract"),
                        to_address:   addr.clone(),
                        amount:       coins(2000000, "SIENNA")});
                assert_tx!(deps
                    => [addr, SIENNA => 0] at [block 4, T=T]
                    => mgmt::msg::Handle::Claim {}
                    => Ok(cosmwasm_std::HandleResponse {
                        data:     None,
                        log:      vec![],
                        messages: vec![msg.clone()] }) );
            },
            _ => unreachable!()
        }
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
        let r1 = vec![(canon!(deps, &BOB), Uint128::from(100u128))];
        let _ = tx(&mut deps, mock_env(1, 1, &ALICE, coins(10, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r1.clone() });
        assert_query!(deps => Recipients => Recipients { recipients: r1 });
    }

    when "the admin tries to set the recipients above the total"
    then "an error should be returned"
    and  "the recipients should not be updated" {
        let r2 = vec![(canon!(deps, &BOB), Uint128::from(10000000u128))];
        assert_tx!(deps
            => [ALICE, SIENNA=>0] at [block 4, T=4]
            => mgmt::msg::Handle::SetRecipients { recipients: r2 }
            => Err(StdError::GenericErr {
                msg: mgmt::strings::err_allocation(10000000, 2500),
                backtrace: None}));
        assert_query!(deps => Recipients => Recipients { recipients: r1 });
    }

    when "a stranger tries to set the recipients"
    then "they should not be able to" {
        let r3 = vec![(canon!(deps, &MALLORY), Uint128::from(100u128))];
        let _ = tx(&mut deps,
            mock_env(1, 1, &MALLORY, coins(10, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r3 });
        assert_query!(deps => Recipients => Recipients { recipients: r1 });
    }

    given "the contract is already launched" {
        let _ = tx(&mut deps, mock_env(2, 2, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
    }

    when "the admin tries to set the recipients"
    then "the recipients should be updated" {
        let r4 = vec![(canon!(deps, &BOB), Uint128::from(200u128))];
        let _ = tx(&mut deps,
            mock_env(3, 3, &ALICE, coins(1000, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r4.clone() });
        assert_query!(deps => Recipients => Recipients { recipients: r4 });
    }

    when "the admin tries to set the recipients above the total"
    then "an error should be returned"
    and  "the recipients should not be updated" {
        let r5 = vec![(canon!(deps, &BOB), Uint128::from(10000000u128))];
        assert_tx!(deps
            => [ALICE, SIENNA=>0] at [block 4, T=4]
            => mgmt::msg::Handle::SetRecipients { recipients: r5 }
            => Err(StdError::GenericErr {
                msg: mgmt::strings::err_allocation(10000000, 2500),
                backtrace: None}) );
        assert_query!(deps => Recipients => Recipients { recipients: r4 });
    }

    when "a stranger tries to set the recipients"
    then "an error should be returned" {
        let r6 = vec![(canon!(deps, &MALLORY), Uint128::from(100u128))];
        let _ = tx(&mut deps,
            mock_env(4, 4, &MALLORY, coins(0, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r6 });
        assert_query!(deps => Recipients => Recipients { recipients: r4 });
    }

    #[claim_configurable]

    given "the contract is not yet launched" {
        let ALICE: HumanAddr = HumanAddr::from("ALICE");
        let BOB:   HumanAddr = HumanAddr::from("BOB");
        let mut deps = harness(&[
            (&ALICE, &coins(1000, "SIENNA")),
            (&BOB,   &coins(   0, "SIENNA")),
        ]);
        let configured_claim_amount = Uint128::from(200u128);
        let r = vec![(canon!(deps, &BOB), configured_claim_amount)];
        let _ = tx(&mut deps,
            mock_env(0, 0, &ALICE, coins(1000, "SIENNA")),
            mgmt::msg::Handle::SetRecipients { recipients: r.clone() });
        assert_query!(deps => Recipients => Recipients { recipients: r });
    }

    when "a configurable claimant tries to claim funds"
    then "they should be denied" {
        assert_tx!(deps
            => [BOB, SIENNA=>0] at [block 0, T=0]
            => mgmt::msg::Handle::Claim {}
            => Err(StdError::GenericErr {
                msg: mgmt::strings::PRELAUNCH.to_string(),
                backtrace: None }) );
    }

    given "the contract is already launched" {
        let _ = tx(
            &mut deps,
            mock_env(0, 0, &ALICE, coins(0, "SIENNA")),
            mgmt::msg::Handle::Launch {});
    }

    when "a configured claimant tries to claim funds"
    then "the contract should transfer them to their address"
    and  "it should remember how much that address has claimed so far" {
        let msg = snip20::handle::HandleMsg::Transfer {
            recipient: BOB.clone(),
            amount:    configured_claim_amount,
            padding:   None
        }.to_cosmos_msg(
            256,
            "".to_string(),
            HumanAddr::from("mgmt"),
            None
        ).unwrap();
        assert_tx!(deps
            => [BOB, SIENNA=>0] at [block 0, T=0]
            => mgmt::msg::Handle::Claim {}
            => Ok(cosmwasm_std::HandleResponse {
                data:     None,
                log:      vec![],
                messages: vec![ msg ] }) );
    }

    when "a configured claimant tries to claim funds"
    and  "the claimant has already claimed within this time period"
    then "the contract should respond that there's nothing at this time" {
        assert_tx!(deps
            => [BOB, SIENNA=>0] at [block 1, T=1]
            => mgmt::msg::Handle::Claim {}
            => Err(StdError::GenericErr {
                msg: mgmt::strings::NOTHING.to_string(),
                backtrace: None }) );
    }

    when "a configured claimant tries to claim funds"
    and  "enough time has passed since their last claim"
    then "the contract should transfer more funds" {
        let msg = snip20::handle::HandleMsg::Transfer {
            recipient: BOB.clone(),
            amount:    configured_claim_amount,
            padding:   None
        }.to_cosmos_msg(
            256,
            "".to_string(),
            HumanAddr::from("mgmt"),
            None
        ).unwrap();
        assert_tx!(deps
            => [BOB, SIENNA=>0] at [block 2, T=DAY]
            => mgmt::msg::Handle::Claim {}
            => Ok(cosmwasm_std::HandleResponse {
                data:     None,
                log:      vec![],
                messages: vec![msg] }) );
    }

);
