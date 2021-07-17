#![allow(unused_macros)]
#![allow(non_snake_case)]

use fadroma::scrt::cosmwasm_std::{HumanAddr, StdError};
use crate::{
    test, assert_error, // macro-exports
    msg::Response,
    rewards_math::{Amount, Volume},
    rewards_harness::{RewardsHarness, Snip20},
    DAY
};

const RPT: u128 = 100;

kukumba! {
    StdError,

    #[ok_pool_init]
    given "no instance" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin"); }
    when  "admin inits with an asset token address"
    then  "the instance configures a viewing key for itself" {
        assert_eq!(T.init_configured(1, &admin)?, (vec![Snip20::set_viewing_key("")], 0, 0)); }
    when  "admin locks funds"
    then  "the instance starts counting the liquidity that accumulates" {
        test!(T=1 ; pool -> { locked: 0, lifetime: 0, updated: 0 });
        test!(T=2 ; pool -> { locked: 0, lifetime: 0, updated: 0 });
        test!(T=3 ; pool -> { locked: 0, lifetime: 0, updated: 0 });
        test!(T=4 ; pool -> { locked: 0, lifetime: 0, updated: 0 });
        test!(T=4 ; admin locks 1 -> [Snip20::transfer_from("admin", "contract_addr", "1")]);
        test!(T=4 ; pool -> { locked: 1, lifetime: 0, updated: 4 });
        test!(T=5 ; pool -> { locked: 1, lifetime: 1, updated: 4 });
        test!(T=6 ; pool -> { locked: 1, lifetime: 2, updated: 4 }); }

    #[ok_pool_init_then_set_lp_token]
    given  "no instance" {
        let mut T  = RewardsHarness::new();
        let admin  = HumanAddr::from("admin");
        let badman = HumanAddr::from("badman"); }
    when  "admin inits without providing an asset token address"
    then  "the instance is not ready" {
        assert_eq!(T.init_partial(0, &admin)?, (vec![Snip20::set_viewing_key(""),], 0, 0));
        assert_error!(T.q_pool_info(1), "missing liquidity provision token"); }
    when  "badman tries to provide an asset token address"
    then  "an error is returned and nothing changes" {
        assert_eq!(T.tx_set_token(2, &badman, "bad_addr", "bad_hash"), Err(StdError::unauthorized()));
        assert_error!(T.q_pool_info(3), "missing liquidity provision token"); }
    when  "admin provides an asset token address"
    then  "the instance configures a viewing key for itself"
    and   "it starts counting when someone locks funds" {
        assert_eq!(T.tx_set_token(4, &admin, "lp_token_address", "lp_token_hash")?, (vec![], 0, 0));
        test!(T=6 ; pool -> { locked: 0, lifetime: 0, updated: 0 })
        test!(T=6 ; admin locks 1 -> [ Snip20::transfer_from("admin", "contract_addr", "1") ]);
        test!(T=6 ; pool -> { locked: 1, lifetime: 0, updated: 6 })
        test!(T=7 ; pool -> { locked: 1, lifetime: 1, updated: 6 })
        test!(T=8 ; pool -> { locked: 1, lifetime: 2, updated: 6 }) }

    #[ok_one]
    given "an instance:" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
        let alice = HumanAddr::from("alice");
        let _ = T.init_configured(0, &admin)?;
        let _ = T.tx_set_vk(0, &alice, "")?;
        T = T.fund(RPT)
        test!(T =  0 ; user(alice)      -> { age:     0, locked:   0, lifetime:     0, unlocked:   0, claimed: 0, claimable:   0 }); }
    when "alice first locks lp tokens," {
        test!(T =  1 ; user(alice)      -> { age:     0, locked:   0, lifetime:     0, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  1 ; alice locks 100    -> [ Snip20::transfer_from("alice", "contract_addr", "100") ]);
        test!(T =  1 ; user(alice)      -> { age:     0, locked: 100, lifetime:     0, unlocked:   0, claimed: 0, claimable:   0 }); }
    then "alice's age starts incrementing;" {
        test!(T =  2 ; user(alice)      -> { age:     1, locked: 100, lifetime:   100, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  3 ; user(alice)      -> { age:     2, locked: 100, lifetime:   200, unlocked:   0, claimed: 0, claimable:   0 }); }
    when "alice retrieves half of the tokens," {
        test!(T =  4 ; user(alice)      -> { age:     3, locked: 100, lifetime:   300, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  4 ; alice retrieves 50 -> [ Snip20::transfer("alice",  "50") ]); }
    then "alice's age keeps incrementing;" {
        test!(T =  4 ; user(alice)      -> { age:     3, locked:  50, lifetime:   300, unlocked:   0, claimed: 0, claimable:   0 }); }
    when "alice retrieves all of the tokens," {
        test!(T =  5 ; user(alice)      -> { age:     4, locked:  50, lifetime:   350, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  6 ; user(alice)      -> { age:     5, locked:  50, lifetime:   400, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  6 ; alice retrieves 50 -> [ Snip20::transfer("alice", "50") ]); }
    then "alice's age stops incrementing;" {
        test!(T =  6 ; user(alice)      -> { age:     5, locked:   0, lifetime:   400, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  7 ; user(alice)      -> { age:     5, locked:   0, lifetime:   400, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  8 ; user(alice)      -> { age:     5, locked:   0, lifetime:   400, unlocked:   0, claimed: 0, claimable:   0 }); }
    when "alice locks tokens again," {
        test!(T =  9 ; user(alice)      -> { age:     5, locked:   0, lifetime:   400, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =  9 ; alice locks 1      -> [ Snip20::transfer_from("alice", "contract_addr", "1") ]); }
    then "alice's age resumes incrementing;" {
        test!(T =  9 ; user(alice)      -> { age:     5, locked:   1, lifetime:   400, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T = 10 ; user(alice)      -> { age:     6, locked:   1, lifetime:   401, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T = 11 ; user(alice)      -> { age:     7, locked:   1, lifetime:   402, unlocked:   0, claimed: 0, claimable:   0 }); }
    when "alice's age reaches the configured threshold,"
    then "alice is eligible to claim the whole pool" {
        test!(T = DAY+4 ; user(alice)   -> { age:   DAY, locked:   1, lifetime: 17675, unlocked: 100, claimed: 0, claimable: 100 }); }

    #[ok_two_simultaneous]
    given "an instance:" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
        let alice = HumanAddr::from("alice");
        let bob   = HumanAddr::from("bob");
        let _ = T.init_configured(0, &admin)?;
        let _ = T.tx_set_vk(0, &alice, "")?;
        let _ = T.tx_set_vk(0, &bob,   "")?;
        T = T.fund(RPT)
        test!(T =  0 ; user(alice)     -> { age:     0, locked:   0, lifetime:         0, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  0 ; user(bob)       -> { age:     0, locked:   0, lifetime:         0, unlocked:  0, claimed: 0, claimable:  0 }); }
    when "alice and bob first lock lp tokens simultaneously,"
    then "their ages start incrementing simultaneously;" {
        test!(T =  1 ; user(alice)     -> { age:     0, locked:   0, lifetime:         0, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  1 ; user(bob)       -> { age:     0, locked:   0, lifetime:         0, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  1 ; alice locks 100 -> [ Snip20::transfer_from("alice", "contract_addr", "100") ]);
        test!(T =  1 ; bob   locks 100 -> [ Snip20::transfer_from("bob",   "contract_addr", "100") ]);
        test!(T =  1 ; user(alice)     -> { age:     0, locked: 100, lifetime:         0, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  1 ; user(bob)       -> { age:     0, locked: 100, lifetime:         0, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  2 ; user(alice)     -> { age:     1, locked: 100, lifetime:       100, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  2 ; user(bob)       -> { age:     1, locked: 100, lifetime:       100, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  3 ; user(alice)     -> { age:     2, locked: 100, lifetime:       200, unlocked:  0, claimed: 0, claimable:  0 });
        test!(T =  3 ; user(bob)       -> { age:     2, locked: 100, lifetime:       200, unlocked:  0, claimed: 0, claimable:  0 }); }
    when "alice and bob's ages reach the configured threshold,"
    then "each is eligible to claim half of the pool" {
        test!(T =  DAY+1 ; user(alice) -> { age:   DAY, locked: 100, lifetime: DAY * 100, unlocked: 50, claimed: 0, claimable: 50 });
        test!(T =  DAY+1 ; user(bob)   -> { age:   DAY, locked: 100, lifetime: DAY * 100, unlocked: 50, claimed: 0, claimable: 50 }); }

    #[ok_two_sequential]
    given "an instance" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
        let alice = HumanAddr::from("alice");
        let bob   = HumanAddr::from("bob");
        let _ = T.init_configured(0, &admin)?;
        let _ = T.tx_set_vk(0, &alice, "")?;
        let _ = T.tx_set_vk(0, &bob,   "")?; }
    when "alice locks lp tokens,"
    and  "alice retrieves them after reaching the threshold;"
    then "alice is eligible to claim the whole pool" {
        T = T.fund(RPT)
        test!(T =       1 ; user(alice)          -> { age:   0, locked:   0, lifetime:         0, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =       1 ; alice locks 100        -> [ Snip20::transfer_from("alice", "contract_addr", "100") ]);
        test!(T =   DAY+1 ; user(alice)          -> { age: DAY, locked: 100, lifetime: DAY * 100, unlocked: 100, claimed: 0, claimable: 100 });
        test!(T =   DAY+1 ; alice retrieves 100    -> [ Snip20::transfer("alice", "100") ]);
        test!(T =   DAY+1 ; user(alice)          -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked: 100, claimed: 0, claimable: 100 }); }
    when "bob locks the same amount of tokens" {
        test!(T =   DAY+2 ; user(bob)            -> { age:   0, locked:   0, lifetime:         0, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =   DAY+2 ; bob locks 100          -> [ Snip20::transfer_from("bob", "contract_addr", "100") ]);
        test!(T =   DAY+2 ; user(bob)            -> { age:   0, locked: 100, lifetime:         0, unlocked:   0, claimed: 0, claimable:   0 }); }
    then "alice's rewards start decreasing proportionally" {
        test!(T = DAY+2+1 ; user(alice)          -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  99, claimed: 0, claimable:   99 });
        test!(T = DAY+2+DAY/2 ; user(alice)      -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  66, claimed: 0, claimable:   66 });
        test!(T = DAY+2+DAY/2+1000 ; user(alice) -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  64, claimed: 0, claimable:   64 }); }
    when "bob reaches the age threshold"
    then "each is eligible to claim half of the pool" {
        test!(T = 2*DAY+2 ; user(bob)            -> { age: DAY, locked: 100, lifetime: DAY * 100, unlocked:  50, claimed: 0, claimable:  50 });
        test!(T = 2*DAY+2 ; bob retrieves 100      -> [ Snip20::transfer("bob", "100") ]);
        test!(T = 2*DAY+2 ; user(bob)            -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  50, claimed: 0, claimable:  50 });
        test!(T = 2*DAY+2 ; user(alice)          -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  50, claimed: 0, claimable:  50 }); }

    #[ok_two_sequential_with_claim]
    given "an instance" {
        let mut T = RewardsHarness::new();
        let admin = HumanAddr::from("admin");
        let alice = HumanAddr::from("alice");
        let bob   = HumanAddr::from("bob");
        let _ = T.init_configured(0, &admin)?;
        let _ = T.tx_set_vk(0, &alice, "")?;
        let _ = T.tx_set_vk(0, &bob,   "")?; }
    when "alice locks lp tokens,"
    and  "alice retrieves them after reaching the threshold;"
    then "alice is eligible to claim the whole pool" {
        T = T.fund(RPT)
        test!(T =       1 ; user(alice)     -> { age:   0, locked:   0, lifetime:         0, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =       1 ; alice locks 100 -> [ Snip20::transfer_from("alice", "contract_addr", "100") ]);
        test!(T =   DAY+1 ; user(alice)     -> { age: DAY, locked: 100, lifetime: DAY * 100, unlocked: 100, claimed: 0, claimable: 100 });
        test!(T =   DAY+1 ; alice locks 100 -> [ Snip20::transfer("alice", "100") ]);
        test!(T =   DAY+1 ; user(alice)     -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked: 100, claimed: 0, claimable: 100 }); }
    when "bob locks the same amount of tokens" {
        test!(T =   DAY+2 ; user(bob)     -> { age:   0, locked:   0, lifetime:         0, unlocked:   0, claimed: 0, claimable:   0 });
        test!(T =   DAY+2 ; bob locks 100 -> [ Snip20::transfer_from("bob", "contract_addr", "100") ]);
        test!(T =   DAY+2 ; user(bob)     -> { age:   0, locked: 100, lifetime:         0, unlocked:   0, claimed: 0, claimable:   0 }); }
    then "alice's rewards start decreasing proportionally" {
        test!(T = DAY+2+1 ; user(alice) -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  99, claimed: 0, claimable:   99 }); }
    when "alice claims some time after maturing" {
        test!(T = DAY+2+DAY/2 ; user(alice)      -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  66, claimed: 0, claimable:   66 });
        test!(T = DAY+2+DAY/2 ; alice claims     -> [ Snip20::transfer("alice", "66") ]);
        test!(T = DAY+2+DAY/2+1000 ; user(alice) -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  64, claimed: 66, claimable:   0 }); }
    when "bob reaches the age threshold"
    then "each is eligible to claim half of the pool" {
        test!(T = 2*DAY+2 ; user(bob)         -> { age: DAY, locked: 100, lifetime: DAY * 100, unlocked:  50, claimed:  0, claimable:  50 });
        test!(T = 2*DAY+2 ; bob retrieves 100 -> [ Snip20::transfer("bob", "100") ]);
        test!(T = 2*DAY+2 ; user(bob)         -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  50, claimed:  0, claimable:  50 });
        test!(T = 2*DAY+2 ; user(alice)       -> { age: DAY, locked:   0, lifetime: DAY * 100, unlocked:  50, claimed: 66, claimable:   0 }); }

    #[ok_lock_and_retrieve]
    given "an instance" {
        let mut T = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let mallory = HumanAddr::from("mallory");
        assert_eq!(T.init_configured(0, &admin)?, (vec![ Snip20::set_viewing_key("") ], 0, 0)); }
    when  "someone requests to lock tokens"
    then  "the instance transfers them to itself"
    and   "the liquidity provider starts accruing a reward" {
        test!(T = 1; bob locks 100 -> [ Snip20::transfer_from("alice", "contract_addr", "100") ]);
        test!(T = 2; pool -> { locked: 100, lifetime: 100, updated: 1 }) }
    when  "a provider requests to retrieve tokens"
    then  "the instance transfers them to the provider"
    and   "the reward now increases at a reduced rate" {
        test!(T = 3; pool -> { locked: 100, lifetime: 200, updated: 1 })
        test!(T = 4; alice retrieves 50 -> [ Snip20::transfer("alice", "50") ]); }
    when  "a provider requests to retrieve all their tokens"
    then  "the instance transfers them to the provider"
    and   "their reward stops increasing" {
        test!(T = 5; pool -> { locked:  50, lifetime: 350, updated: 4 })
        test!(T = 5; alice retrieves 50 -> [ Snip20::transfer("alice", "50") ]);
        test!(T = 6; pool -> { locked:   0, lifetime: 350, updated: 5 }) }
    when  "someone else requests to lock tokens"
    then  "the previous provider's share of the rewards begins to diminish" {
        test!(T = 7; pool -> { locked:   0, lifetime: 350, updated: 5 })
        test!(T = 7; bob locks 500 -> [ Snip20::transfer_from("bob", "contract_addr", "500") ]);
        test!(T = 8; pool -> { locked: 500, lifetime: 850, updated: 7 }) }
    when  "a provider tries to retrieve too many tokens"
    then  "they get an error" {
        test!(T = 9; pool -> { locked: 500, lifetime: 1350, updated: 7 })
        assert_error!(T.tx_retrieve(9, &bob, 1000u128), "not enough locked (500 < 1000)");
        test!(T = 10; pool -> { locked: 500, lifetime: 1850, updated: 7 }) }
    when  "a stranger tries to retrieve any tokens"
    then  "they get an error" {
        assert_error!(T.tx_retrieve(10, &mallory, 100u128), "not enough locked (0 < 100)");
        test!(T = 11; pool -> { locked: 500, lifetime: 2350, updated: 7 }) }

    #[ok_claim]
    given "an instance" {
        let mut T = RewardsHarness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let _ = T.init_configured(0, &admin)?; }
    when  "strangers try to claim rewards"
    then  "they get an error" {
        assert_error!(T.tx_claim(1, &alice), "lock tokens for 17280 more blocks to be eligible");
        assert_error!(T.tx_claim(1, &bob),   "lock tokens for 17280 more blocks to be eligible"); }
    when  "users provide liquidity"
    and   "they wait for rewards to accumulate" {
        assert_eq!(T.tx_lock(2, &alice, 100)?, (vec![Snip20::transfer_from("alice", "contract_addr", "100")], 0, 0));
        assert_error!(T.tx_claim(2, &alice), "lock tokens for 17280 more blocks to be eligible");
        assert_eq!(T.tx_lock(2, &bob, 100)?, (vec![Snip20::transfer_from("bob", "contract_addr", "100")], 0, 0));
        assert_error!(T.tx_claim(2, &alice), "lock tokens for 17280 more blocks to be eligible");
        assert_error!(T.tx_claim(3, &bob),   "lock tokens for 17279 more blocks to be eligible");
        assert_error!(T.tx_claim(4, &alice), "lock tokens for 17278 more blocks to be eligible");
        assert_error!(T.tx_claim(5, &bob),   "lock tokens for 17277 more blocks to be eligible"); }
    and   "a provider claims rewards"
    then  "that provider receives reward tokens" {
        T = T.fund(RPT)
        assert_eq!(T.tx_claim(17282, &alice)?, (vec![Snip20::transfer("alice", "50")], 0, 0)); }
    when  "a provider claims rewards twice"
    then  "rewards are sent only once" {
        assert_error!(T.tx_claim(17282, &alice), "already claimed"); }
    when  "a provider claims their rewards less often"
    then  "they receive equivalent rewards as long as the liquidity locked hasn't changed" {
        //assert_eq!(T.tx_claim(4, &alice)?, (vec![Snip20::transfer("alice",  "5")], 0, 0));
        T = T.fund(RPT)
        assert_eq!(T.tx_claim(3 + DAY * 2, &alice)?, (vec![Snip20::transfer("alice", "50")], 0, 0));
        assert_eq!(T.tx_claim(3 + DAY * 2, &bob)?,   (vec![Snip20::transfer("bob", "100")], 0, 0));
        //println!("{:#?}", T.tx_claim(10, &alice));
        //println!("{:#?}", T.tx_claim(4, &bob)?);
        //panic!()
    }

    #[rewards_parallel_or_sequential]
    given "three users providing liquidity" {
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let cyril   = HumanAddr::from("cyril"); }
    when "they provide the liquidity simultaneously" {
        let mut T = RewardsHarness::new().fund(RPT);
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
        println!("{:#?}", T.q_user_info(DAY, &cyril)); }
    then "it's the same as if they provided the liquidity sequentially, as long as nobody claims" {
        let mut T = RewardsHarness::new().fund(RPT);
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
        assert_eq!(T.tx_claim(DAY * 4, &cyril)?, (vec![Snip20::transfer("cyril", "33")], 0, 0)); }
    when "one of the users claims when providing liquidity sequentially"
    then "the remaining rewards are split between the late-comer and the late-claimer" {
        let mut T = RewardsHarness::new();
        let _ = T.init_configured(0, &admin)?; }

}
