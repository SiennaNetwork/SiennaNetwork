#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use cosmwasm_std::{StdError, HumanAddr, Uint128};
use secret_toolkit::snip20::handle::{mint_msg, set_minters_msg, transfer_msg};
use sienna_mgmt::{PRELAUNCH, NOTHING, msg::Handle};
use sienna_schedule::Schedule;

kukumba!(

    #[claim_as_user]

    given "a contract with the production schedule" {
        harness!(deps; ADMIN);
        let founder_1 = HumanAddr::from("secret1TODO20xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let founder_2 = HumanAddr::from("secret1TODO21xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let founder_3 = HumanAddr::from("secret1TODO22xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        let source = include_str!("../../config_msg.json");
        println!("{}", source);
        let s: Schedule = serde_json::from_str(include_str!("../../config_msg.json")).unwrap();
        test_tx!(deps, ADMIN, 0, 0;
            Handle::Configure { schedule: s.clone() } => tx_ok!());
    }
    when "the contract is not yet launched"
    when "anyone tries to claim funds"
    then "they are denied" {
        test_tx!(deps, founder_1, 1, 1;
            Handle::Claim {} => tx_err!(PRELAUNCH));
    }
    when "the contract is launched"
    then "tokens should be minted and minting should be disabled" {
        let t_launch = 2;
        test_tx!(deps, ADMIN, 2, t_launch;
            Handle::Launch {} => tx_ok!(
                mint_msg(
                    HumanAddr::from("mgmt"), Uint128::from(s.total),
                    None, 256, String::new(), HumanAddr::from("token")
                ).unwrap(),
                set_minters_msg(
                    vec![],
                    None, 256, String::new(), HumanAddr::from("token")
                ).unwrap()
            ));
    }

    when "Founder1 tries to claim funds before the cliff"
    then "they are denied" {
        test_tx!(deps, founder_1, 3, t_launch + 1;
            Handle::Claim {} => tx_err!(NOTHING));
        test_tx!(deps, founder_1, 4, t_launch + 15552000 - 1;
            Handle::Claim {} => tx_err!(NOTHING));
    }
    when "Founder1 claims funds right after the cliff"
    then "they receive 80000 SIENNA" {
        test_tx!(deps, founder_1, 5, t_launch + 15552000;
            Handle::Claim {} => tx_ok_claim!(founder_1, SIENNA!(80000u128)));
    }
    when "Founder1 tries to claim funds before the next vesting"
    then "they are denied" {
        test_tx!(deps, founder_1, 6, t_launch + 15552001;
            Handle::Claim {} => tx_err!(NOTHING));
    }
    when "Founder1 claims funds again after 1 day"
    then "they receive 1 vesting's worth of 1500 SIENNA" {
        test_tx!(deps, founder_1, 7, t_launch + 15552000 + 86400;
            Handle::Claim {} => tx_ok_claim!(founder_1, SIENNA!(1500u128)));
    }
    when "Founder1 claims funds again after 2 more days"
    then "they receive 2 vestings' worth of 3000 SIENNA" {
        test_tx!(deps, founder_1, 8, t_launch + 15520000 + 86400 + 86400 * 2;
            Handle::Claim {} => tx_ok_claim!(founder_1, SIENNA!(3000u128)));
    }

    when "Founder2 tries to claim funds before the cliff"
    then "they are denied" {
        test_tx!(deps, founder_2, 9, t_launch + 15552000;
            Handle::Claim {} => tx_err!(NOTHING));
    }
    when "Founder2 claims funds for the 1st time 10 days after the cliff"
    then "they receive cliff 80000 + 10 vestings' worth of 15000 = 95000 SIENNA" {
        test_tx!(deps, founder_2, 10, t_launch + 15552001 + 10 * 86400;
            Handle::Claim {} => tx_ok_claim!(founder_2, SIENNA!(95000u128)));
    }
    when "Founder 3 claims funds 500 days after the cliff"
    then "they receive the full amount of 731000 SIENNA" {
        test_tx!(deps, founder_3, 11, t_launch + 15552001 + 500 * 86400;
            Handle::Claim {} => tx_ok_claim!(founder_3, SIENNA!(731000u128)));
    }

);

    //given "the contract is not yet launched" {
        //harness!(deps; ADMIN, BOB);

        //let configured_claim_amount: Uint128 = Uint128::from(200u128);
        //let r = vec![(BOB, configured_claim_amount)];
    //let _ = tx(&mut deps,
            //mock_env(0, 0, &ADMIN),
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
            //=> from [ADMIN] at [block 0, T=0]
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
        //harness!(deps; ADMIN, BOB);
        //let configured_claim_amount = Uint128::from(200u128);
        //let r = vec![(BOB.clone(), configured_claim_amount)];
        //let _ = tx(&mut deps,
            //mock_env(0, 0, &ADMIN),
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
            //mock_env(0, 0, &ADMIN),
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
