#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate kukumba;
mod harness; use harness::{RewardsHarness, Snip20};
use fadroma::scrt::cosmwasm_std::{HumanAddr, StdError};
use sienna_rewards_benchmark::{msg::Response, rewards_math::{Monotonic, Amount, Volume}};

const DAY: Monotonic = 17280; // blocks

macro_rules! test {

    ($T:ident = $now:expr ; pool_unprepared -> {
        balance: $balance:expr, lifetime: $lifetime:expr, updated: $updated:expr
    }) => {
        assert_eq!($T.q_pool_info($now as u64)?, Response::PoolInfo {
            lp_token: None,
            balance:  Amount::from($balance   as u128),
            lifetime: Volume::zero($lifetime as u128),
            updated:  $updated as u64,
            now:      $now   as u64,
        });
    };

    ($T:ident = $now:expr ; pool -> {
        balance: $balance:expr, lifetime: $lifetime:expr, updated: $updated:expr
    }) => {
        assert_eq!($T.q_pool_info($now as u64)?, Response::PoolInfo {
            lp_token: $T.lp_token(),
            balance:  Amount::from($balance   as u128),
            lifetime: Volume::from($lifetime as u128),
            updated:  $updated as u64,
            now:      $now   as u64,
        });
    };

    ($T:ident = $now:expr ; user($who:expr) -> {
        age: $age:expr, balance: $balance:expr, lifetime: $lifetime:expr,
        unlocked: $unlocked:expr, claimed: $claimed:expr, claimable: $claimable:expr
    }) => {
        assert_eq!($T.q_user_info($now as u64, &$who)?, Response::UserInfo {
            age:       $age as u64,
            balance:   Amount::from($balance      as u128),
            lifetime:  Volume::from($lifetime as u128),
            unlocked:  Amount::from($unlocked    as u128),
            claimed:   Amount::from($claimed     as u128),
            claimable: Amount::from($claimable   as u128)
        });
    };

    ($T:ident = $now:expr ; lock($who:expr, $amount:expr) -> [ $($msg:expr),* ]) => {
        assert_eq!($T.tx_lock($now, &$who, ($amount as u128).into())?, (vec![ $($msg,)* ], 0, 0))
    };

    ($T:ident = $now:expr ; retr($who:expr, $amount:expr) -> [ $($msg:expr),* ]) => {
        assert_eq!($T.tx_retrieve($now, &$who, ($amount as u128).into())?, (vec![ $($msg,)* ], 0, 0))
    };

    ($T:ident = $now:expr ; claim($who:expr) -> [ $($msg:expr),* ]) => {
        assert_eq!($T.tx_claim($now, &$who)?, (vec![ $($msg,)* ], 0, 0))
    };

}

kukumba! {
    StdError,

    #[ok_pool_init]
    given "no instance" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
    }
    when  "someone inits with an asset token address"
    then  "the instance configures a viewing key for itself" {
        assert_eq!(T.init_configured(1, &admin)?, (vec![Snip20::set_viewing_key("")], 0, 0));
    }
    when  "someone locks funds"
    then  "the instance goes live" {
        assert_error!(T.q_pool_info(1u64), "missing POOL_SINCE");
        test!(T=2 ; pool -> { balance: 0, lifetime: 0, updated: 1 });
        test!(T=3 ; pool -> { balance: 0, lifetime: 0, updated: 1 });
        test!(T=4 ; pool -> { balance: 0, lifetime: 0, updated: 1 });
        test!(T=4 ; lock(admin, 1) -> [Snip20::transfer_from("admin", "contract_addr", "1")]);
        test!(T=4 ; pool -> { balance: 1, lifetime: 0, updated: 4 });
        test!(T=5 ; pool -> { balance: 1, lifetime: 1, updated: 4 });
        test!(T=6 ; pool -> { balance: 1, lifetime: 2, updated: 4 });
    }

    #[ok_pool_init_then_set_lp_token]
    given  "no instance" {
        let mut T  = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
        let badman = HumanAddr::from("badman");
    }
    when  "someone inits without providing an asset token address"
    then  "the instance is not ready" {
        assert_eq!(T.init_partial(0, &admin)?, (vec![Snip20::set_viewing_key(""),], 0, 0));
        assert_error!(T.q_pool_info(1), "missing liquidity provision token");
    }
    when  "a stranger tries to provide an asset token address"
    then  "an error is returned and nothing changes" {
        assert_eq!(T.tx_set_token(2, &badman, "bad_addr", "bad_hash"), Err(StdError::unauthorized()));
        assert_error!(T.q_pool_info(3), "missing liquidity provision token");
    }
    when  "the admin provides an asset token address"
    then  "the instance configures a viewing key for itself"
    and   "it goes live when someone locks funds" {
        assert_eq!(T.tx_set_token(4, &admin, "lp_token_address", "lp_token_hash")?, (vec![], 0, 0));
        assert_error!(T.q_pool_info(5), "missing POOL_SINCE");
        test!(T=6 ; lock(admin, 1) -> [ Snip20::transfer_from("admin", "contract_addr", "1") ]);
        test!(T=6 ; pool -> { balance: 1, lifetime: 0, updated: 6 })
        test!(T=7 ; pool -> { balance: 1, lifetime: 1, updated: 6 })
        test!(T=8 ; pool -> { balance: 1, lifetime: 2, updated: 6 })
    }

    #[ok_one]
    given "an instance:" {
        let VOL   = 100u128;
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
        let alice = HumanAddr::from("alice");
        let _ = T.init_configured(0, &admin)?;
        let _ = T.tx_set_vk(0, &alice, "")?;
    }
    when "one first locks lp tokens," {
        //test!(T =  1 ; user(alice)      -> { age:     0, balance:   0, lifetime:   0, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  1 ; lock(alice, 100) -> [ Snip20::transfer_from("alice", "contract_addr", "100") ]);
        //test!(T =  1 ; user(alice)      -> { age:     0, balance: 100, lifetime:   0, unlocked: 0, claimed: 0, claimable: 0 });
    }
    then "one's age starts incrementing;" {
        test!(T =  2 ; user(alice)      -> { age:     1, balance: 100, lifetime:   100, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  3 ; user(alice)      -> { age:     2, balance: 100, lifetime:   200, unlocked: 0, claimed: 0, claimable: 0 });
    }
    when "one retrieves half of the tokens," {
        test!(T =  4 ; user(alice)      -> { age:     3, balance: 100, lifetime:   300, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  4 ; retr(alice,  50) -> [ Snip20::transfer("alice",  "50") ]);
    }
    then "one's age keeps incrementing;" {
        test!(T =  4 ; user(alice)      -> { age:     3, balance:  50, lifetime:   300, unlocked: 0, claimed: 0, claimable: 0 });
    }
    when "one retrieves all of the tokens," {
        test!(T =  5 ; user(alice)      -> { age:     4, balance:  50, lifetime:   350, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  6 ; user(alice)      -> { age:     5, balance:  50, lifetime:   400, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  6 ; retr(alice,  50) -> [ Snip20::transfer("alice", "50") ]);
    }
    then "one's age stops incrementing;" {
        test!(T =  6 ; user(alice)      -> { age:     5, balance:   0, lifetime:   400, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  7 ; user(alice)      -> { age:     5, balance:   0, lifetime:   400, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  8 ; user(alice)      -> { age:     5, balance:   0, lifetime:   400, unlocked: 0, claimed: 0, claimable: 0 });
    }
    when "one locks tokens again," {
        test!(T =  9 ; user(alice)      -> { age:     5, balance:   0, lifetime:   400, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T =  9 ; lock(alice,   1) -> [ Snip20::transfer_from("alice", "contract_addr", "1") ]);
    }
    then "one's age resumes incrementing;" {
        test!(T =  9 ; user(alice)      -> { age:     5, balance:   1, lifetime:   400, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T = 10 ; user(alice)      -> { age:     6, balance:   1, lifetime:   401, unlocked: 0, claimed: 0, claimable: 0 });
        test!(T = 11 ; user(alice)      -> { age:     7, balance:   1, lifetime:   402, unlocked: 0, claimed: 0, claimable: 0 });
    }
    when "one's age reaches the configured threshold,"
    then "one is eligible to claim the whole pool" {
        test!(T = DAY+4 ; user(alice)   -> { age:   DAY, balance:   1, lifetime: 17675, unlocked: 0, claimed: 0, claimable: 0 });
    }

    #[ok_two_simultaneous]
    given "an instance:" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
        let alice = HumanAddr::from("alice");
        let bob   = HumanAddr::from("bob");
        let _ = T.init_configured(0, &admin)?;
    }
    when "two first lock lp tokens simultaneously,"
    then "their ages starts incrementing;" {}
    when "their ages reache the configured threshold,"
    then "each is eligible to claim half of the pool" {}

    #[ok_two_sequential]
    given "an instance" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
        let alice = HumanAddr::from("alice");
        let bob   = HumanAddr::from("bob");
        let _ = T.init_configured(0, &admin)?;
    }
    when "one locks lp tokens,"
    and "one retrieves them after reaching the threshold;" {}
    then "one is eligible to claim the whole pool" {}

    when "their ages reache the configured threshold,"
    then "each is eligible to claim half of the pool" {}

    #[ok_partial_overlap]
    given "an instance" {}

    #[ok_lock_and_retrieve]
    given "an instance" {
        let mut T = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let mallory = HumanAddr::from("mallory");
        assert_eq!(T.init_configured(0, &admin)?, (vec![
            Snip20::set_viewing_key(""),
        ], 0, 0));
    }
    when  "someone requests to lock tokens"
    then  "the instance transfers them to itself"
    and   "the liquidity provider starts accruing a reward" {
        assert_eq!(T.tx_lock(1, &alice, 100u128)?, (vec![
            Snip20::transfer_from("alice", "contract_addr", "100")
        ], 0, 0));
        assert_eq!(T.q_pool_info(2)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(100u128),
            lifetime: Volume::from(100u128),
            updated:  1,
            now:    2
        });
    }
    when  "a provider requests to retrieve tokens"
    then  "the instance transfers them to the provider"
    and   "the reward now increases at a reduced rate" {
        assert_eq!(T.q_pool_info(3)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(100u128),
            lifetime: Volume::from(200u128),
            updated:  1,
            now:    3
        });
        assert_eq!(T.tx_retrieve(3, &alice, 50u128)?, (vec![
            Snip20::transfer("alice", "50")
        ], 0, 0));
        assert_eq!(T.q_pool_info(4)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(50u128),
            lifetime: Volume::from(250u128),
            updated:  3,
            now:    4
        });
    }
    when  "a provider requests to retrieve all their tokens"
    then  "the instance transfers them to the provider"
    and   "their reward stops increasing" {
        assert_eq!(T.q_pool_info(5)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(50u128),
            lifetime: Volume::from(300u128),
            updated:  3,
            now:    5
        });
        assert_eq!(T.tx_retrieve(5, &alice, 50u128)?, (vec![
            Snip20::transfer("alice", "50")
        ], 0, 0));
        assert_eq!(T.q_pool_info(6)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(0u128),
            lifetime: Volume::from(300u128),
            updated:  5,
            now:    6
        });
    }
    when  "someone else requests to lock tokens"
    then  "the previous provider's share of the rewards begins to diminish" {
        assert_eq!(T.q_pool_info(7)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(0u128),
            lifetime: Volume::from(300u128),
            updated:  5,
            now:    7
        });
        assert_eq!(T.tx_lock(7, &bob, 500u128)?, (vec![
            Snip20::transfer_from("bob", "contract_addr", "500")
        ], 0, 0));
        assert_eq!(T.q_pool_info(8)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(500u128),
            lifetime: Volume::from(800u128),
            updated:  7,
            now:    8
        });
    }
    when  "a provider tries to retrieve too many tokens"
    then  "they get an error" {
        assert_eq!(T.q_pool_info(9)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(500u128),
            lifetime: Volume::from(1300u128),
            updated:  7,
            now:    9
        });
        assert_error!(
            T.tx_retrieve(9, &bob, 1000u128),
            "not enough balance (500 < 1000)"
        );
        assert_eq!(T.q_pool_info(10)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(500u128),
            lifetime: Volume::from(1800u128),
            updated:  7,
            now:    10
        });
    }
    when  "a stranger tries to retrieve any tokens"
    then  "they get an error" {
        assert_error!(
            T.tx_retrieve(10, &mallory, 100u128),
            "never provided liquidity"
        );
        assert_eq!(T.q_pool_info(11)?, Response::PoolInfo {
            lp_token: T.lp_token(),
            balance: Amount::from(500u128),
            lifetime: Volume::from(2300u128),
            updated:  7,
            now:    11
        });
    }

    #[ok_claim]
    given "an instance" {
        let mut T = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let _ = T.init_configured(0, &admin)?;
    }
    when  "strangers try to claim rewards"
    then  "they get an error" {
        assert_error!(T.tx_claim(1, &alice), "missing user age data");
        assert_error!(T.tx_claim(1, &bob),   "missing user age data");
    }
    when  "users provide liquidity"
    and   "they wait for rewards to accumulate" {
        assert_eq!(T.tx_lock(2, &alice, 100)?, (vec![
            Snip20::transfer_from("alice", "contract_addr", "100")
        ], 0, 0));
        assert_error!(T.tx_claim(2, &alice), "17280 blocks until eligible");
        assert_eq!(T.tx_lock(2, &bob, 100)?, (vec![
            Snip20::transfer_from("bob", "contract_addr", "100")
        ], 0, 0));
        assert_error!(T.tx_claim(2, &alice), "17280 blocks until eligible");
        assert_error!(T.tx_claim(3, &bob),   "17279 blocks until eligible");
        assert_error!(T.tx_claim(4, &alice), "17278 blocks until eligible");
    }
    and   "a provider claims rewards"
    then  "that provider receives reward tokens" {
        T = T.fund(100)
        assert_eq!(T.tx_claim(17282, &alice)?, (vec![
            Snip20::transfer("alice", "50")
        ], 0, 0));
    }
    when  "a provider claims rewards twice"
    then  "rewards are sent only once" {
        assert_error!(T.tx_claim(17282, &alice), "already claimed");
    }
    when  "a provider claims their rewards less often"
    then  "they receive equivalent rewards as long as the liquidity balance hasn't changed" {
        //assert_eq!(T.tx_claim(4, &alice)?, (vec![Snip20::transfer("alice",  "5")], 0, 0));
        T = T.fund(100)
        assert_eq!(T.tx_claim(3 + DAY * 2, &alice)?, (vec![
            Snip20::transfer("alice", "50")
        ], 0, 0));
        assert_eq!(T.tx_claim(3 + DAY * 2, &bob)?, (vec![
            Snip20::transfer("bob", "100")
        ], 0, 0));
        //println!("{:#?}", T.tx_claim(10, &alice));
        //println!("{:#?}", T.tx_claim(4, &bob)?);
        //panic!()
    }

    #[rewards_parallel_or_sequential]
    given "three users providing liquidity" {
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let cyril   = HumanAddr::from("cyril");
    }
    when "they provide the liquidity simultaneously" {
        let mut T = RewardsHarness::new().fund(100);
        let _ = T.init_configured(0, &admin)?;
        let _ = T.tx_set_vk(0, &alice, "")?;
        let _ = T.tx_set_vk(0, &bob,   "")?;
        let _ = T.tx_set_vk(0, &cyril, "")?;

        let _ = T.tx_lock(0, &alice, 100)?;
        let _ = T.tx_lock(0, &bob,   100)?;
        let _ = T.tx_lock(0, &cyril, 100)?;
        //println!("{:#?}", T.q_pool_info(0));
        assert_eq!(T.tx_claim(DAY, &alice)?, (vec![Snip20::transfer("alice", "33")], 0, 0));
        assert_eq!(T.tx_claim(DAY, &bob  )?, (vec![Snip20::transfer("bob",   "33")], 0, 0));
        assert_eq!(T.tx_claim(DAY, &cyril)?, (vec![Snip20::transfer("cyril", "33")], 0, 0));
        println!("{:#?}", T.q_pool_info(DAY));
        println!("{:#?}", T.q_user_info(DAY, &alice));
        println!("{:#?}", T.q_user_info(DAY, &bob));
        println!("{:#?}", T.q_user_info(DAY, &cyril));
    }
    then "it's the same as if they provided the liquidity sequentially, as long as nobody claims" {
        let mut T = RewardsHarness::new().fund(100);
        let _ = T.init_configured(0, &admin)?;
        let _ = T.tx_set_vk(0, &alice, "")?;
        let _ = T.tx_set_vk(0, &bob,   "")?;
        let _ = T.tx_set_vk(0, &cyril, "")?;

        let _ = T.tx_lock(              2, &alice, 100)?;
        let _ = T.tx_retrieve(DAY * 1 + 2, &alice, 100)?;
        let _ = T.tx_lock(    DAY * 1 + 3, &bob,   100)?;
        let _ = T.tx_retrieve(DAY * 2 + 3, &bob,   100)?;
        let _ = T.tx_lock(    DAY * 2 + 4, &cyril, 100)?;
        let _ = T.tx_retrieve(DAY * 3 + 4, &cyril, 100)?;
        println!("{:#?}", T.q_pool_info(DAY * 4));
        println!("{:#?}", T.q_user_info(DAY * 4, &alice));
        println!("{:#?}", T.q_user_info(DAY * 4, &bob));
        println!("{:#?}", T.q_user_info(DAY * 4, &cyril));
        assert_eq!(T.tx_claim(DAY * 4, &alice)?, (vec![Snip20::transfer("alice", "33")], 0, 0));
        assert_eq!(T.tx_claim(DAY * 4, &bob  )?, (vec![Snip20::transfer("bob",   "33")], 0, 0));
        assert_eq!(T.tx_claim(DAY * 4, &cyril)?, (vec![Snip20::transfer("cyril", "33")], 0, 0));
    }
    when "one of the users claims when providing liquidity sequentially"
    then "the remaining rewards are split between the late-comer and the late-claimer" {
        let mut T = RewardsHarness::new();
        let _ = T.init_configured(0, &admin)?;
    }

}
