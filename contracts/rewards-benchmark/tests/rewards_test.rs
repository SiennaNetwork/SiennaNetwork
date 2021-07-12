#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate kukumba;
mod harness; use harness::RewardsHarness;
use fadroma::scrt::cosmwasm_std::{Uint128, HumanAddr, StdError};
use sienna_rewards_benchmark::msg::Response;

kukumba! {
    StdError,

    #[ok_init_status]
    given "no instance" {
        let mut test = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
    }
    when  "someone inits with an asset token address"
    then  "the instance configures a viewing key for itself" {
        assert_eq!(
            test.init_configured(0, &admin)?,
            (vec!["{\"set_viewing_key\":{\"key\":\"\",\"padding\":null}}".into()], 0, 0),
        );
    }
    when  "someone locks funds"
    then  "the instance goes live" {
        assert_error!(test.q_status(1u64), "missing POOL_SINCE");
        assert_eq!(
            test.tx_lock(2, &admin, 1u128)?,
            (vec!["{\"transfer_from\":{\"owner\":\"admin\",\"recipient\":\"contract_addr\",\"amount\":\"1\",\"padding\":null}}".into()], 0, 0)
        );
        assert_eq!(
            test.q_status(2u64)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::zero(),
                since:  2,
                now:    2
            }
        );
        assert_eq!(
            test.q_status(3u64)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(1u128),
                since:  2,
                now:    3
            }
        );
        assert_eq!(
            test.q_status(4u64)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(2u128),
                since:  2,
                now:    4
            }
        );
    }

    #[ok_init_then_provide]
    given  "no instance" {
        let mut test = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
        let badman = HumanAddr::from("badman");
    }
    when  "someone inits without providing an asset token address"
    then  "the instance is not ready" {
        assert_eq!(
            test.init_partial(0, &admin)?,
            (vec!["{\"set_viewing_key\":{\"key\":\"\",\"padding\":null}}".into()], 0, 0),
        );
        assert_error!(test.q_status(1), "not configured");
    }
    when  "a stranger tries to provide an asset token address"
    then  "an error is returned and nothing changes" {
        assert_eq!(
            test.tx_set_token(2, &badman, "bad_addr", "bad_hash"),
            Err(StdError::unauthorized())
        );
        assert_error!(test.q_status(3), "not configured");
    }
    when  "the admin provides an asset token address"
    then  "the instance configures a viewing key for itself"
    and   "it goes live when someone locks funds" {
        assert_eq!(
            test.tx_set_token(4, &admin, "ok_addr", "ok_hash")?,
            (vec![], 0, 0),
        );
        assert_error!(test.q_status(5), "missing POOL_SINCE");
        assert_eq!(
            test.tx_lock(6, &admin, 1)?,
            (vec!["{\"transfer_from\":{\"owner\":\"admin\",\"recipient\":\"contract_addr\",\"amount\":\"1\",\"padding\":null}}".into()], 0, 0)
        );
        let result = test.q_status(7)?;
        assert_eq!(
            test.q_status(6)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::zero(),
                since:  6,
                now:    6
            }
        );
        assert_eq!(
            test.q_status(7)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(1u128),
                since:  6,
                now:    7
            }
        );
        assert_eq!(
            test.q_status(8)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(2u128),
                since:  6,
                now:    8
            }
        );
    }

    #[ok_lock_and_retrieve]
    given "an instance" {
        let mut test = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let mallory = HumanAddr::from("mallory");
        assert_eq!(
            test.init_configured(0, &admin)?,
            (vec!["{\"set_viewing_key\":{\"key\":\"\",\"padding\":null}}".into()], 0, 0),
        );
    }
    when  "someone requests to lock tokens"
    then  "the instance transfers them to itself"
    and   "the liquidity provider starts accruing a reward" {
        assert_eq!(
            test.tx_lock(1, &alice, 100u128)?,
            (vec!["{\"transfer_from\":{\"owner\":\"alice\",\"recipient\":\"contract_addr\",\"amount\":\"100\",\"padding\":null}}".into()], 0, 0)
        );
        assert_eq!(
            test.q_status(2)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(2u128),
                since:  1,
                now:    2
            }
        );
    }
    when  "a provider requests to retrieve tokens"
    then  "the instance transfers them to the provider"
    and   "the reward now increases at a reduced rate" {
        assert_eq!(
            test.tx_retrieve(3, &alice, 50u128)?,
            (vec!["{\"transfer\":{\"recipient\":\"alice\",\"amount\":\"50\",\"padding\":null}}".into()], 0, 0)
        );
        assert_eq!(
            test.q_status(4)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(2u128),
                since:  3,
                now:    4
            }
        );
    }
    when  "a provider requests to retrieve all their tokens"
    then  "the instance transfers them to the provider"
    and   "their reward stops increasing" {
        assert_eq!(
            test.tx_retrieve(5, &alice, 50u128)?,
            (vec!["{\"transfer\":{\"recipient\":\"alice\",\"amount\":\"50\",\"padding\":null}}".into()], 0, 0)
        );
        assert_eq!(
            test.q_status(6)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(2u128),
                since:  5,
                now:    6
            }
        );
    }
    when  "someone else requests to lock tokens"
    then  "the previous provider's share of the rewards begins to diminish" {
        assert_eq!(
            test.tx_lock(7, &bob, 500u128)?,
            (vec!["{\"transfer_from\":{\"owner\":\"bob\",\"recipient\":\"contract_addr\",\"amount\":\"500\",\"padding\":null}}".into()], 0, 0)
        );
        assert_eq!(
            test.q_status(8)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(2u128),
                since:  7,
                now:    8
            }
        );
        let result = test.q_status(9u64)?;
        assert_eq!(
            test.q_status(9)?,
            Response::Status {
                volume: Uint128::from(1u128),
                total:  Uint128::from(2u128),
                since:  7,
                now:    9
            }
        );
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
        let admin  = HumanAddr::from("admin");
        let alice  = HumanAddr::from("alice");
        let result = test.init_configured(0, &admin)?;
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
