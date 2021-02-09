#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env, tx};

use sienna_mgmt as mgmt;
//use mgmt::{DAY, MONTH, ONE_SIENNA, err_allocation, Stream, Vesting};

use cosmwasm_std::{StdError, HumanAddr, Uint128};

use secret_toolkit::snip20;

//kukumba!(

    //#[configure]

    //given "the contract is not yet launched" {
        //harness!(deps; ALICE, BOB, MALLORY);
    //}

    //when "the admin sets the recipients"
    //then "the recipients should be updated" {
        //let r1 = vec![(BOB.clone(), SIENNA!(100))];
        //let _ = tx(&mut deps, mock_env(1, 1, &ALICE),
            //mgmt::msg::Handle::Configure { recipients: r1.clone() });
        //test_q!(deps; Schedule; Schedule {
            //recipients: r1
        //});
    //}

    //when "the admin tries to set the recipients above the total" {
        //let r2 = vec![(BOB.clone(), SIENNA!(10000000))];
    //}
    //then "an error should be returned" {
        //test_tx!(deps
            //=> from [ALICE] at [block 4, T=4]
            //=> mgmt::msg::Handle::Configure { recipients: r2 }
            //=> Err(StdError::GenericErr {
                //msg: err_allocation(10000000*ONE_SIENNA, 2500*ONE_SIENNA),
                //backtrace: None
            //}));
    //}
    //and  "the recipients should not be updated" {
        //test_q!(deps; Schedule; Schedule {
            //recipients: r1
        //});
    //}

    //when "a stranger tries to set the recipients"
    //then "they should not be able to" {
        //let r3 = vec![(MALLORY.clone(), Uint128::from(100u128))];
        //let _ = tx(&mut deps,
            //mock_env(1, 1, &MALLORY),
            //mgmt::msg::Handle::Configure {
                //recipients: r3
            //});
        //test_q!(deps; Schedule; Schedule {
            //recipients: r1
        //});
    //}

    //given "the contract is already launched" {
        //let _ = tx(&mut deps, mock_env(2, 2, &ALICE),
            //mgmt::msg::Handle::Launch {});
    //}

    //when "the admin tries to set the recipients"
    //then "the recipients should be updated" {
        //let r4 = vec![(BOB.clone(), Uint128::from(200u128))];
        //let _ = tx(&mut deps,
            //mock_env(3, 3, &ALICE),
            //mgmt::msg::Handle::Configure { recipients: r4.clone() });
        //test_q!(deps; Schedule; Schedule { recipients: r4 });
    //}

    //when "the admin tries to set the recipients above the total"
    //then "an error should be returned"
    //and  "the recipients should not be updated" {
        //let r5 = vec![(BOB.clone(), SIENNA!(10000000))];
        //test_tx!(deps
            //=> from [ALICE] at [block 4, T=4]
            //=> mgmt::msg::Handle::Configure { recipients: r5 }
            //=> Err(StdError::GenericErr {
                //msg: err_allocation(10000000*ONE_SIENNA, 2500*ONE_SIENNA),
                //backtrace: None}) );
        //test_q!(deps; Schedule; Schedule { recipients: r4 });
    //}

    //when "a stranger tries to set the recipients"
    //then "an error should be returned" {
        //let r6 = vec![(MALLORY.clone(), Uint128::from(100u128))];
        //let _ = tx(&mut deps,
            //mock_env(4, 4, &MALLORY),
            //mgmt::msg::Handle::Configure { recipients: r6 });
        //test_q!(deps; Schedule; Schedule { recipients: r4 });
    //}

//);

//kukumba!(
    //#[transfer_ownership]
    //given "a contract instance" {
        //harness!(deps; ALICE, BOB, MALLORY);
    //}

    //when "a stranger tries to set a new admin"
    //then "just the hit counter goes up" {
        //test_tx!(deps
            //=> from [MALLORY] at [block 2, T=DAY]
            //=> mgmt::msg::Handle::TransferOwnership { new_admin: MALLORY.clone() }
            //=> Err(StdError::Unauthorized { backtrace: None }))
    //}

    //when "the admin tries to set a new admin"
    //then "the admin is updated" {
        //test_tx!(deps
            //=> from [ALICE] at [block 2, T=DAY]
            //=> mgmt::msg::Handle::TransferOwnership { new_admin: BOB.clone() }
            //=> Ok(cosmwasm_std::HandleResponse {
                //data: None, log: vec![], messages: vec![] }))
    //}

    //when "the old admin tries to set a new admin"
    //then "just the hit counter goes up" {
        //test_tx!(deps
            //=> from [ALICE] at [block 2, T=DAY]
            //=> mgmt::msg::Handle::TransferOwnership { new_admin: ALICE.clone() }
            //=> Err(StdError::Unauthorized { backtrace: None }))
    //}

    //when "the new admin tries to set the admin"
    //then "the admin is updated" {
        //test_tx!(deps
            //=> from [BOB] at [block 2, T=DAY]
            //=> mgmt::msg::Handle::TransferOwnership { new_admin: ALICE.clone() }
            //=> Ok(cosmwasm_std::HandleResponse {
                //data: None, log: vec![], messages: vec![] }))
    //}
//);
