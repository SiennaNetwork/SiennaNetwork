#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate kukumba;
mod harness; use harness::RewardsHarness;
use fadroma::scrt::cosmwasm_std::{Uint128, HumanAddr, StdError};
use sienna_rewards_benchmark::msg::Response;

struct Snip20;
impl Snip20 {
    pub fn set_viewing_key (key: &str) -> String {
        format!(
            "{{\"set_viewing_key\":{{\"key\":\"{}\",\"padding\":null}}}}",
            key
        ).into()
    }
    pub fn transfer_from (owner: &str, recipient: &str, amount: &str) -> String {
        format!(
            "{{\"transfer_from\":{{\"owner\":\"{}\",\"recipient\":\"{}\",\"amount\":\"{}\",\"padding\":null}}}}",
            owner, recipient, amount
        ).into()
    }
    pub fn transfer (recipient: &str, amount: &str) -> String {
        format!(
            "{{\"transfer\":{{\"recipient\":\"{}\",\"amount\":\"{}\",\"padding\":null}}}}",
            recipient, amount
        ).into()
    }
}

kukumba! {
    StdError,

    #[ok_init_status]
    given "no instance" {
        let mut test = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
    }
    when  "someone inits with an asset token address"
    then  "the instance configures a viewing key for itself" {
        assert_eq!(test.init_configured(0, &admin)?, (vec![
            Snip20::set_viewing_key("")
        ], 0, 0));
    }
    when  "someone locks funds"
    then  "the instance goes live" {
        assert_error!(test.q_status(1u64), "missing POOL_SINCE");
        assert_eq!(test.tx_lock(2, &admin, 1u128)?, (vec![
            Snip20::transfer_from("admin", "contract_addr", "1")
        ], 0, 0));
        assert_eq!(test.q_status(2u64)?, Response::Status {
            volume: Uint128::from(1u128),
            total:  Uint128::zero(),
            since:  2,
            now:    2
        });
        assert_eq!(test.q_status(3u64)?, Response::Status {
            volume: Uint128::from(1u128),
            total:  Uint128::from(1u128),
            since:  2,
            now:    3
        });
        assert_eq!(test.q_status(4u64)?, Response::Status {
            volume: Uint128::from(1u128),
            total:  Uint128::from(2u128),
            since:  2,
            now:    4
        });
    }

    #[ok_init_then_provide]
    given  "no instance" {
        let mut test = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
        let badman = HumanAddr::from("badman");
    }
    when  "someone inits without providing an asset token address"
    then  "the instance is not ready" {
        assert_eq!(test.init_partial(0, &admin)?, (vec![
            Snip20::set_viewing_key(""),
        ], 0, 0));
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
        assert_eq!(test.tx_set_token(4, &admin, "ok_addr", "ok_hash")?, (vec![], 0, 0),);
        assert_error!(test.q_status(5), "missing POOL_SINCE");
        assert_eq!(test.tx_lock(6, &admin, 1)?, (vec![
            Snip20::transfer_from("admin", "contract_addr", "1")
        ], 0, 0));
        assert_eq!(test.q_status(6)?, Response::Status {
            volume: Uint128::from(1u128),
            total:  Uint128::zero(),
            since:  6,
            now:    6
        });
        assert_eq!(test.q_status(7)?, Response::Status {
            volume: Uint128::from(1u128),
            total:  Uint128::from(1u128),
            since:  6,
            now:    7
        });
        assert_eq!(test.q_status(8)?, Response::Status {
            volume: Uint128::from(1u128),
            total:  Uint128::from(2u128),
            since:  6,
            now:    8
        });
    }

    #[ok_lock_and_retrieve]
    given "an instance" {
        let mut test = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let mallory = HumanAddr::from("mallory");
        assert_eq!(test.init_configured(0, &admin)?, (vec![
            Snip20::set_viewing_key(""),
        ], 0, 0));
    }
    when  "someone requests to lock tokens"
    then  "the instance transfers them to itself"
    and   "the liquidity provider starts accruing a reward" {
        assert_eq!(test.tx_lock(1, &alice, 100u128)?, (vec![
            Snip20::transfer_from("alice", "contract_addr", "100")
        ], 0, 0));
        assert_eq!(test.q_status(2)?, Response::Status {
            volume: Uint128::from(100u128),
            total:  Uint128::from(100u128),
            since:  1,
            now:    2
        });
    }
    when  "a provider requests to retrieve tokens"
    then  "the instance transfers them to the provider"
    and   "the reward now increases at a reduced rate" {
        assert_eq!(test.q_status(3)?, Response::Status {
            volume: Uint128::from(100u128),
            total:  Uint128::from(200u128),
            since:  1,
            now:    3
        });
        assert_eq!(test.tx_retrieve(3, &alice, 50u128)?, (vec![
            Snip20::transfer("alice", "50")
        ], 0, 0));
        assert_eq!(test.q_status(4)?, Response::Status {
            volume: Uint128::from(50u128),
            total:  Uint128::from(250u128),
            since:  3,
            now:    4
        });
    }
    when  "a provider requests to retrieve all their tokens"
    then  "the instance transfers them to the provider"
    and   "their reward stops increasing" {
        assert_eq!(test.q_status(5)?, Response::Status {
            volume: Uint128::from(50u128),
            total:  Uint128::from(300u128),
            since:  3,
            now:    5
        });
        assert_eq!(test.tx_retrieve(5, &alice, 50u128)?, (vec![
            Snip20::transfer("alice", "50")
        ], 0, 0));
        assert_eq!(test.q_status(6)?, Response::Status {
            volume: Uint128::from(0u128),
            total:  Uint128::from(300u128),
            since:  5,
            now:    6
        });
    }
    when  "someone else requests to lock tokens"
    then  "the previous provider's share of the rewards begins to diminish" {
        assert_eq!(test.q_status(7)?, Response::Status {
            volume: Uint128::from(0u128),
            total:  Uint128::from(300u128),
            since:  5,
            now:    7
        });
        assert_eq!(test.tx_lock(7, &bob, 500u128)?, (vec![
            Snip20::transfer_from("bob", "contract_addr", "500")
        ], 0, 0));
        assert_eq!(test.q_status(8)?, Response::Status {
            volume: Uint128::from(500u128),
            total:  Uint128::from(800u128),
            since:  7,
            now:    8
        });
    }
    when  "a provider tries to retrieve too many tokens"
    then  "they get an error" {
        assert_eq!(test.q_status(9)?, Response::Status {
            volume: Uint128::from(500u128),
            total:  Uint128::from(1300u128),
            since:  7,
            now:    9
        });
        assert_error!(
            test.tx_retrieve(9, &bob, 1000u128),
            "not enough balance (500 < 1000)"
        );
        assert_eq!(test.q_status(10)?, Response::Status {
            volume: Uint128::from(500u128),
            total:  Uint128::from(1800u128),
            since:  7,
            now:    10
        });
    }
    when  "a stranger tries to retrieve any tokens"
    then  "they get an error" {
        assert_error!(
            test.tx_retrieve(10, &mallory, 100u128),
            "never provided liquidity"
        );
        assert_eq!(test.q_status(11)?, Response::Status {
            volume: Uint128::from(500u128),
            total:  Uint128::from(2300u128),
            since:  7,
            now:    11
        });
    }

    #[ok_claim]
    given "an instance" {
        let mut test = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let _ = test.init_configured(0, &admin)?;
    }
    when  "strangers tries to claim rewards"
    then  "they get an error" {
        assert_error!(test.tx_claim(1, &alice), "missing user liquidity data");
        assert_error!(test.tx_claim(1, &bob),   "missing user liquidity data");
    }
    when  "users provide liquidity" {
        assert_eq!(test.tx_lock(2, &alice, 100)?, (vec![
            Snip20::transfer_from("alice", "contract_addr", "100")
        ], 0, 0));
        assert_eq!(test.tx_lock(2, &bob, 100)?, (vec![
            Snip20::transfer_from("bob", "contract_addr", "100")
        ], 0, 0));
    }
    and   "they weit for rewards to accumulate" {
        assert_error!(test.tx_claim(2, &alice), "pool is empty");
        assert_error!(test.tx_claim(2, &bob), "pool is empty");
    }
    and   "a provider claims rewards"
    then  "that provider receives reward tokens" {
        assert_eq!(test.tx_claim(3, &alice)?, (vec![Snip20::transfer("alice", "1")], 0, 0));
    }
    when  "a provider claims rewards twice"
    then  "rewards are sent only once" {
        assert_error!(test.tx_claim(3, &alice), "already claimed");
    }
    when  "a provider claims their rewards less often"
    then  "they receive equivalent rewards as long as the liquidity balance hasn't changed" {
        assert_eq!(test.tx_claim(4, &alice)?, (vec![Snip20::transfer("alice", "1")], 0, 0));
        assert_eq!(test.tx_claim(4, &bob)?,   (vec![Snip20::transfer("bob",   "2")], 0, 0));
    }

    #[rewards_parallel_or_sequential]
    given "three users providing liquidity" {
    }
    when "they provide the liquidity simultaneously" {
    }
    then "it's the same as if they provided the liquidity sequentially as long as nobody claims" {
    }

}
