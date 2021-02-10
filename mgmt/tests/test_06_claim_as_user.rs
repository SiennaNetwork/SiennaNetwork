#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128};
use secret_toolkit::snip20::handle::{mint_msg, transfer_msg};
use sienna_mgmt::{PRELAUNCH, NOTHING, msg::Handle};
use sienna_schedule::Schedule;

kukumba!(

    #[claim_as_user]

    given "a contract with a configured schedule" {
        harness!(deps; ALICE, BOB);
        let source = include_str!("../../config_msg.json");
        println!("{}", source);
        let s: Schedule = serde_json::from_str(include_str!("../../config_msg.json")).unwrap();
        test_tx!(deps, ALICE, 0, 0;
            Handle::Configure { schedule: s.clone() } => tx_ok!());
    }
    when "the contract is not yet launched"
    when "the user tries to claim funds"
    then "they are denied" {
        test_tx!(deps, BOB, 1, 1;
            Handle::Claim {} => tx_err!(PRELAUNCH));
    }
    when "the contract is launched"
    and  "the user tries to claim funds"
    then "they receive the first portion of the allocated funds" {
        test_tx!(deps, BOB, 2, 2;
            Handle::Claim {} => tx_ok!(transfer_msg(
                HumanAddr::from("mgmt"),
                Uint128::from(100u128),
                None, 256, String::new(), HumanAddr::from("token")
            ).unwrap()));
    }
    when "the user tries to claim funds before the next vesting point"
    then "they are denied" {
        todo!();
    }
    when "the user tries to claim funds after the next vesting point"
    the  "they receive the next portion of the allocated funds" {
        todo!();
    }
    when "the user tries to claim funds before the next vesting point"
    then "they are denied" {
        todo!();
    }
    when "the user tries to claim funds after several vesting points"
    the  "they receive the next portions of the allocated funds" {
        todo!();
    }
    when "the user tries to claim funds before the next vesting point"
    then "they are denied" {
        todo!();
    }

);

    //given "the contract is not yet launched" {
        //harness!(deps; ALICE, BOB);

        //let configured_claim_amount: Uint128 = Uint128::from(200u128);
        //let r = vec![(BOB, configured_claim_amount)];
        //let _ = tx(&mut deps,
            //mock_env(0, 0, &ALICE),
            //mgmt::msg::Handle::Configure { schedule: r.clone() });

        //test_q!(deps; Schedule; Schedule { schedule: r });
    //}

    //when "a predefined claimant tries to claim funds"
    //then "they should be denied" {
        //let Stream { addr, vesting, .. } = SCHEDULE.predefined.get(0).unwrap()
        //match vesting {
            //Vesting::Periodic {..} => {
                //test_tx!(deps
                    //=> from [addr] at [block 4, T=1]
                    //=> mgmt::msg::Handle::Claim {}
                    //=> Err(StdError::GenericErr {
                        //msg: mgmt::constants::PRELAUNCH.to_string(),
                        //backtrace: None }) );
            //},
            //_ => unreachable!()
        //}
    //}

    //given "the contract is launched" {
        //test_tx!(deps
            //=> from [ALICE] at [block 0, T=0]
            //=> mgmt::msg::Handle::Launch {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![
                    //snip20::handle::HandleMsg::Mint {
                        //recipient: HumanAddr::from("mgmt"),
                        //amount:    Uint128::from(10000000 * ONE_SIENNA),
                        //padding:   None
                    //}.to_cosmos_msg(
                        //256,
                        //"".to_string(),
                        //HumanAddr::from("mgmt"),
                        //None
                    //).unwrap()
                //] }) );
    //}

    //when "a predefined claimant tries to claim funds before the cliff"
    //then "they should be denied" {
        //let start;
        //let Stream { addr: PREDEF, vesting, .. } = SCHEDULE.predefined.get(0).unwrap();
        //match vesting {
            //Vesting::Periodic { start_at, .. } => {
                //start = *start_at;
                //test_tx!(deps
                    //=> from [PREDEF] at [block 4, T=start-1]
                    //=> mgmt::msg::Handle::Claim {}
                    //=> Err(StdError::GenericErr {
                        //msg: mgmt::constants::NOTHING.to_string(),
                        //backtrace: None }) );
            //},
            //_ => unreachable!()
        //}
    //}

    //when "a predefined claimant tries to claim funds at/after the cliff"
    //and  "the first post-cliff vesting has not passed"
    //then "the contract should transfer the cliff amount"
    //and  "it should remember how much that address has claimed so far" {
        //test_tx!(deps
            //=> from [PREDEF] at [block 4, T=start]
            //=> mgmt::msg::Handle::Claim {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![
                    //snip20::handle::HandleMsg::Transfer {
                        //recipient: PREDEF.clone(),
                        //amount:    SIENNA!(75000),
                        //padding:   None
                    //}.to_cosmos_msg(
                        //256,
                        //"".to_string(),
                        //HumanAddr::from("mgmt"),
                        //None
                    //).unwrap()
                //] }) );
    //}

    //when "a predefined claimant tries to claim funds"
    //and  "the claimant has already claimed within this time period"
    //then "the contract should respond that there's nothing at this time" {
        //test_tx!(deps
            //=> from [PREDEF] at [block 6, T=start+1]
            //=> mgmt::msg::Handle::Claim {}
            //=> Err(StdError::GenericErr {
                //msg: mgmt::constants::NOTHING.to_string(),
                //backtrace: None }) );
    //}

    //when "a predefined claimant tries to claim funds"
    //and  "enough time has passed since their last claim"
    //then "the contract should transfer more funds" {
        //let msg = snip20::handle::HandleMsg::Transfer {
            //recipient: PREDEF.clone(),
            //amount:    SIENNA!(75000),
            //padding:   None
        //}.to_cosmos_msg(
            //256,
            //"".to_string(),
            //HumanAddr::from("mgmt"),
            //None
        //).unwrap();
        //test_tx!(deps
            //=> from [PREDEF] at [block 4, T=start+1*MONTH]
            //=> mgmt::msg::Handle::Claim {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![msg.clone()] }) );
        //test_tx!(deps
            //=> from [PREDEF] at [block 4, T=start+2*MONTH]
            //=> mgmt::msg::Handle::Claim {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![msg.clone()] }) );
    //}

    //when "another predefined claimant tries to claim funds"
    //and  "this is the first time they make a claim"
    //and  "it is a long time after the end of the vesting"
    //then "the contract should transfer everything in one go" {
        //let Stream { addr: PREDEF, vesting, .. } = SCHEDULE.predefined.get(1).unwrap();
        //match vesting {
            //Vesting::Periodic { start_at, duration, .. } => {
                //let T = (start_at + duration) + 48 * MONTH;
                //let msg = snip20::handle::HandleMsg::Transfer {
                    //recipient: PREDEF.clone(),
                    //amount:    Uint128::from(1999999999999999999999680u128),
                    ////amount:    SIENNA!(75000),
                    //padding:   None
                //}.to_cosmos_msg(
                    //256,
                    //"".to_string(),
                    //HumanAddr::from("mgmt"),
                    //None
                //).unwrap();
                //test_tx!(deps
                    //=> from [PREDEF] at [block 4, T=T]
                    //=> mgmt::msg::Handle::Claim {}
                    //=> Ok(cosmwasm_std::HandleResponse {
                        //data:     None,
                        //log:      vec![],
                        //messages: vec![msg.clone()] }) );
            //},
            //_ => unreachable!()
        //}
    //}

//);


//kukumba!(

    //#[claim_configurable]

    //given "the contract is not yet launched" {
        //harness!(deps; ALICE, BOB);
        //let configured_claim_amount = Uint128::from(200u128);
        //let r = vec![(BOB.clone(), configured_claim_amount)];
        //let _ = tx(&mut deps,
            //mock_env(0, 0, &ALICE),
            //mgmt::msg::Handle::Configure { schedule: r.clone() });
        //test_q!(deps; Schedule; Schedule { schedule: r });
    //}

    //when "a configurable claimant tries to claim funds"
    //then "they should be denied" {
        //test_tx!(deps
            //=> from [BOB] at [block 0, T=0]
            //=> mgmt::msg::Handle::Claim {}
            //=> Err(StdError::GenericErr {
                //msg: mgmt::constants::PRELAUNCH.to_string(),
                //backtrace: None }) );
    //}

    //given "the contract is already launched" {
        //let _ = tx(
            //&mut deps,
            //mock_env(0, 0, &ALICE),
            //mgmt::msg::Handle::Launch {});
    //}

    //when "a configured claimant tries to claim funds"
    //then "the contract should transfer them to their address"
    //and  "it should remember how much that address has claimed so far" {
        //let msg = snip20::handle::HandleMsg::Transfer {
            //recipient: BOB.clone(),
            //amount:    configured_claim_amount,
            //padding:   None
        //}.to_cosmos_msg(
            //256,
            //"".to_string(),
            //HumanAddr::from("mgmt"),
            //None
        //).unwrap();
        //test_tx!(deps
            //=> from [BOB] at [block 0, T=0]
            //=> mgmt::msg::Handle::Claim {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![ msg ] }) );
    //}

    //when "a configured claimant tries to claim funds"
    //and  "the claimant has already claimed within this time period"
    //then "the contract should respond that there's nothing at this time" {
        //test_tx!(deps
            //=> from [BOB] at [block 1, T=1]
            //=> mgmt::msg::Handle::Claim {}
            //=> Err(StdError::GenericErr {
                //msg: mgmt::constants::NOTHING.to_string(),
                //backtrace: None }) );
    //}

    //when "a configured claimant tries to claim funds"
    //and  "enough time has passed since their last claim"
    //then "the contract should transfer more funds" {
        //let msg = snip20::handle::HandleMsg::Transfer {
            //recipient: BOB.clone(),
            //amount:    configured_claim_amount,
            //padding:   None
        //}.to_cosmos_msg(
            //256,
            //"".to_string(),
            //HumanAddr::from("mgmt"),
            //None
        //).unwrap();
        //test_tx!(deps
            //=> from [BOB] at [block 2, T=DAY]
            //=> mgmt::msg::Handle::Claim {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![msg] }) );
    //}

//);
