#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_mgmt as mgmt;
use mgmt::{DAY, MONTH, ONE_SIENNA, err_allocation};

use cosmwasm_std::{StdError, HumanAddr, Uint128};

use secret_toolkit::snip20;

kukumba!(

    #[claim_stranger]

    given "the contract is not yet launched" {
        harness!(deps; ALICE, MALLORY);
    }

    when "a stranger tries to claim funds"
    then "they should be denied" {
        assert_tx!(deps
            => from [MALLORY] at [block 4, T=4]
            => mgmt::msg::Handle::Claim {}
            => Err(StdError::GenericErr {
                msg: mgmt::PRELAUNCH.to_string(),
                backtrace: None }) );
    }

    given "the contract is already launched" {
        let _ = tx(
            &mut deps,
            mock_env(1, 1, &ALICE),
            mgmt::msg::Handle::Launch {});
    }

    when "a stranger tries to claim funds"
    then "they should be denied" {
        assert_tx!(deps
            => from [MALLORY] at [block 4, T=4]
            => mgmt::msg::Handle::Claim {}
            => Err(StdError::GenericErr {
                msg: mgmt::NOTHING.to_string(),
                backtrace: None }) );
    }

);

//kukumba!(

    //#[claim_predefined]

    //given "the contract is not yet launched" {
        //harness!(deps; ALICE, BOB);

        //let configured_claim_amount: Uint128 = Uint128::from(200u128);
        //let r = vec![(BOB, configured_claim_amount)];
        //let _ = tx(&mut deps,
            //mock_env(0, 0, &ALICE),
            //mgmt::msg::Handle::Configure { schedule: r.clone() });

        //assert_query!(deps; Schedule; Schedule { schedule: r });
    //}

    //when "a predefined claimant tries to claim funds"
    //then "they should be denied" {
        //let Stream { addr, vesting, .. } = SCHEDULE.predefined.get(0).unwrap()
        //match vesting {
            //Vesting::Periodic {..} => {
                //assert_tx!(deps
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
        //assert_tx!(deps
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
                //assert_tx!(deps
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
        //assert_tx!(deps
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
        //assert_tx!(deps
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
        //assert_tx!(deps
            //=> from [PREDEF] at [block 4, T=start+1*MONTH]
            //=> mgmt::msg::Handle::Claim {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![msg.clone()] }) );
        //assert_tx!(deps
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
                //assert_tx!(deps
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
        //assert_query!(deps; Schedule; Schedule { schedule: r });
    //}

    //when "a configurable claimant tries to claim funds"
    //then "they should be denied" {
        //assert_tx!(deps
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
        //assert_tx!(deps
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
        //assert_tx!(deps
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
        //assert_tx!(deps
            //=> from [BOB] at [block 2, T=DAY]
            //=> mgmt::msg::Handle::Claim {}
            //=> Ok(cosmwasm_std::HandleResponse {
                //data:     None,
                //log:      vec![],
                //messages: vec![msg] }) );
    //}

//);
