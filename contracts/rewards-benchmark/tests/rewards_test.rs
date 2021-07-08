#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate kukumba;
mod harness; use harness::RewardsHarness;

use fadroma::scrt::cosmwasm_std::{HumanAddr, StdError};

kukumba! {
    StdError,

    #[ok_init]
    given "no instance" {
        let mut test = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
    }
    when  "someone inits with an asset token address" {
        let result = test.init_configured(0, &admin)?;
    }
    then  "the instance is ready"
    and   "it goes live when someone locks funds" {
        assert_error!(test.q_status(1u64), "missing data");
        let result = test.tx_lock(2, &admin, 1u128)?;
        let result = test.q_status(3u64)?;
    }

    #[ok_init_then_provide]
    given  "no instance" {
        let mut test = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
        let badman = HumanAddr::from("badman");
    }
    when  "someone inits without providing an asset token address" {
        let result = test.init_partial(0, &admin)?;
    }
    then  "the instance is not ready" {
        assert_error!(test.q_status(1u64), "not configured");
    }
    when  "a stranger tries to provide an asset token address" {
        let result = test.tx_set_token(2, &badman, "bad_addr", "bad_hash")?
    }
    then  "an error is returned and nothing changes" {
        assert_error!(test.q_status(3u64), "not configured");
    }
    when  "the admin provides an asset token address" {
        let result = test.tx_set_token(4, &admin, "ok_addr", "ok_hash")?
    }
    then  "the instance is ready"
    and   "it goes live when someone locks funds" {
        assert_error!(test.q_status(1u64), "missing data");
        let result = test.tx_lock(2, &admin, 1u128)?;
        let result = test.q_status(5u64)?;
    }

    #[lock_and_retrieve]
    given "an instance" {
        let mut test = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let mallory = HumanAddr::from("mallory");
        let result  = test.init_configured(0, &admin)?;
    }
    when  "someone requests to lock tokens"
    then  "the instance transfers them to itself"
    and   "the liquidity provider starts accruing a reward" {
        let result = test.tx_lock(1, &alice, 100u128)?;
        println!("{:?}", &result);
        panic!();
        let result = test.q_status(2u64)?;
    }
    when  "a provider requests to retrieve tokens"
    then  "the instance transfers them to the provider"
    and   "the reward now increases at a reduced rate" {
        let result = test.tx_retrieve(3, &alice, 50u128)?;
        let result = test.q_status(4u64.into())?;
    }
    when  "a provider requests to retrieve all their tokens"
    then  "the instance transfers them to the provider"
    and   "their reward stops increasing" {
        let result = test.tx_retrieve(5, &alice, 50u128.into())?;
        let result = test.q_status(5u64)?;
    }
    when  "someone else requests to lock tokens"
    then  "the previous provider's share of the rewards begins to diminish" {
        let result = test.tx_lock(6, &bob, 500)?;
        let result = test.q_status(7u64)?;
        let result = test.q_status(8u64)?;
    }
    when  "a provider tries to retrieve too many tokens"
    then  "they get an error" {
        assert_error!(
            test.tx_retrieve(9, &bob, 1000u128),
            "not enough balance (500 < 1000)"
        );
    }
    when  "a stranger tries to retrieve any tokens"
    then  "they get an error" {
        assert_error!(
            test.tx_retrieve(10, &mallory, 100u128),
            "never provided liquidity"
        );
    }

    #[ok_claim]
    given "an instance" {
        let mut test = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let result  = test.init_configured(0, &admin)?;
    }
    when  "a stranger tries to claim rewards"
    then  "they get an error" {
        assert_error!(
            test.tx_claim(1, &alice),
            "never provided liquidity"
        );
    }
    when  "a provider claims their rewards"
    then  "the instance sends them reward tokens" {
        let result = test.tx_lock(2, &alice, 100)?;
        let result = test.tx_claim(3, &alice)?;
    }
    when  "a provider claims their rewards twice"
    then  "they are sent only once" {
        let result = test.tx_claim(3, &alice)?;
    }
    when  "a provider claims their rewards later"
    then  "they receive an increment" {
        let result = test.tx_claim(4, &alice)?;
    }

    #[rewards_parallel_or_sequential]
    given "todo" {}
    when  "do"   {}
    then  "done" {}

}
