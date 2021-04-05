#![allow(unused_macros)]
#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;
#[macro_use] mod helpers; use helpers::{harness, mock_env};

kukumba! {

    #[no_unauthorized_set_admin]
    given "a contract instance" { harness!(deps; ADMIN, ADMIN2, STRANGER); }
    when "a stranger tries to set a new admin"
    then "that fails" {
        tx!(deps; STRANGER, 1, 1; SetOwner { new_admin: STRANGER.clone() } == err!(auth)); }

    #[ok_set_admin]
    given "a contract instance" { harness!(deps; ADMIN, ADMIN2, STRANGER); }
    when "the admin tries to set a new admin"
    then "the admin is updated" {
        tx!(deps; ADMIN, 2, 2; SetOwner { new_admin: ADMIN2.clone() } == ok!()); }
    when "the former admin tries to set a new admin"
    then "that fails" {
        tx!(deps; ADMIN, 3, 3; SetOwner { new_admin: ADMIN.clone() } == err!(auth)); }
    when "the new admin tries to set the admin"
    then "the admin is updated" {
        tx!(deps; ADMIN2, 4, 4; SetOwner { new_admin: ADMIN.clone() } == ok!()); }

    #[no_unauthorized_disown]
    given "a contract instance" { harness!(deps; ADMIN, STRANGER); }
    when "someone else tries to disown the contract"
    and  "that fails" {
        tx!(deps; STRANGER, 5, 5; Disown {} == err!(auth)); }

    #[ok_disown]
    given "a contract instance" { harness!(deps; ADMIN, STRANGER); }
    when "the admin disowns the contract"
    then "there is no admin"
    and  "nobody can control the contract" {
        tx!(deps; ADMIN, 6, 6; Disown {} == ok!());
        tx!(deps; ADMIN, 2, 2; SetOwner { new_admin: ADMIN.clone() } == err!(auth)); }

}
