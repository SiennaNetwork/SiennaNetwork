#![allow(dead_code)]
use fadroma::{ensemble::MockEnv, ContractLink, HumanAddr, Uint128};
use sienna_mgmt::ConfigResponse;
use sienna_rpt::LinearMap;
use sienna_schedule::{Account, Pool, Schedule};

use crate::setup::{AccountFactory, ADMIN, MGMT_ADDR, TGE};

const USER_INVESTOR_MIKE: &str = "Mike";
const USER_INVESTOR_JOHN: &str = "John";
const USER_MP_RPT1: &str = "RTP1";
const USER_MP_RPT2: &str = "RTP2";
const TOKEN1: &str = "secret1TOKEN1";
const TOKEN2: &str = "secret1TOKEN2";
const TOKEN_DECIMALS: u128 = 1_000_000_000_000_000_000;

#[test]
fn rpt_init() {
    let tge = TGE::default();

    let _: ConfigResponse = tge
        .ensemble
        .query(tge.mgmt.address, sienna_mgmt::QueryMsg::Config {})
        .unwrap();
}

#[test]
fn admin_update_distribution() {
    let mut tge = TGE::default();
    let token1 = HumanAddr::from(TOKEN1);
    let token2 = HumanAddr::from(TOKEN2);
    let updated_distribution = LinearMap(vec![
        (token1.clone(), Uint128::from(10u128)),
        (token2.clone(), Uint128::from(15u128)),
    ]);

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::Configure {
                distribution: updated_distribution.clone(),
                portion: Uint128(25),
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash,
                },
            ),
        )
        .unwrap();

    let response: sienna_rpt::ConfigResponse = tge
        .ensemble
        .query(tge.rpt.address, &sienna_rpt::QueryMsg::Configuration {})
        .unwrap();

    assert_eq!(response.distribution, updated_distribution);
}
#[test]
fn stranger_cant_update_distribution() {
    let mut tge = TGE::default();
    let token1 = HumanAddr::from(TOKEN1);
    let token2 = HumanAddr::from(TOKEN2);
    let updated_distribution = LinearMap(vec![
        (token1.clone(), Uint128::from(1000u128)),
        (token2.clone(), Uint128::from(1500u128)),
    ]);

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::Configure {
                distribution: updated_distribution.clone(),
                portion: Uint128(2500),
            },
            MockEnv::new(
                "stranger",
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash,
                },
            ),
        )
        .unwrap_err();
}
#[test]
fn only_valid_distribution() {
    let mut tge = TGE::default();
    let token1 = HumanAddr::from(TOKEN1);
    let token2 = HumanAddr::from(TOKEN2);
    //must add up to 2500
    let updated_distribution = LinearMap(vec![
        (token1.clone(), Uint128::from(1200u128)),
        (token2.clone(), Uint128::from(1500u128)),
    ]);

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::Configure {
                distribution: updated_distribution.clone(),
                portion: Uint128(2500),
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash,
                },
            ),
        )
        .unwrap_err();
}
#[test]
fn should_distribute() {
    let mut tge = TGE::default();
    let initial_balance = tge.query_balance(ADMIN);

    tge.add_account(
        "TEST".into(),
        Account {
            name: "RPT_SPLIT".into(),
            address: tge.rpt.address.clone(),
            amount: Uint128(25),
            cliff: Uint128(0),
            start_at: 0,
            interval: 180,
            duration: 0,
        },
    )
    .unwrap();

    tge.ensemble
        .execute(
            &sienna_mgmt::HandleMsg::Launch {},
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "MGMT_CONTRACT".into(),
                    code_hash: tge.mgmt.code_hash.clone(),
                },
            ),
        )
        .unwrap();

    tge.ensemble
        .execute(
            &sienna_rpt::HandleMsg::Vest {},
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "RPT_CONTRACT".into(),
                    code_hash: tge.rpt.code_hash.clone(),
                },
            ),
        )
        .unwrap();

    let updated_balance = tge.query_balance(ADMIN);

    assert_ne!(initial_balance, updated_balance);
}

#[test]
fn should_not_schedule_for_invalid_pool_total() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(25_000),
        pools: vec![Pool {
            name: "Investors".to_string(),
            partial: false,
            total: Uint128(20_000),
            accounts: vec![
                Account::create(USER_INVESTOR_MIKE, 2_000, 1000, 10),
                Account::create(USER_INVESTOR_JOHN, 5_000, 1000, 10),
            ],
        }],
    };
    // wrong sum of totals inside pool
    tge.set_shedule(schedule).unwrap_err();
    //assert_eq!(tge.query_schedule().total.u128(), 25_000);
}

#[test]
fn should_not_lanuch_with_incorrect_prefund_balance() {
    // WIP
    let mut tge = TGE::new(true);

    let schedule = Schedule {
        total: Uint128(1_001_000_000_000_000_000_000),
        pools: vec![Pool {
            name: "Investors".to_string(),
            partial: false,
            total: Uint128(1_001_000_000_000_000_000_000),
            accounts: vec![
                Account::create(USER_INVESTOR_MIKE, 1_000_000_000_000_000_000_000, 1000, 10),
                Account::create(USER_INVESTOR_JOHN, 1_000_000_000_000_000_000, 1000, 10),
            ],
        }],
    };
    tge.set_shedule(schedule).unwrap();
    let _ = tge.query_balance(MGMT_ADDR);
    tge.launch().unwrap_err();
}

#[test]
fn should_set_schedule_single_pool() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(25_000),
        pools: vec![Pool {
            name: "Investors".to_string(),
            partial: false,
            total: Uint128(25_000),
            accounts: vec![
                Account::create(USER_INVESTOR_MIKE, 20_000, 1000, 10),
                Account::create(USER_INVESTOR_JOHN, 5_000, 1000, 10),
            ],
        }],
    };

    tge.set_shedule(schedule).unwrap();
    assert_eq!(tge.query_schedule().total.u128(), 25_000);
}

#[test]
fn should_not_set_schedule_overall_total() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(45_001),
        pools: vec![
            Pool {
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(25_000),
                accounts: vec![
                    Account::create(USER_INVESTOR_MIKE, 20_000, 1000, 10),
                    Account::create(USER_INVESTOR_JOHN, 5_000, 1000, 10),
                ],
            },
            Pool {
                name: "Minting Pool".to_string(),
                partial: false,
                total: Uint128(20_000),
                accounts: vec![
                    Account::create(USER_MP_RPT1, 11_000, 1000, 10),
                    Account::create(USER_MP_RPT2, 9_000, 1000, 10),
                ],
            },
        ],
    };
    // sum of pool totals is 45_000, while schedule total is 45_001
    tge.set_shedule(schedule).unwrap_err();
    //assert_eq!(tge.query_schedule().total.u128(), 25_000);
}

#[test]
fn should_set_schedule_multiple_pools() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(45_000),
        pools: vec![
            Pool {
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(25_000),
                accounts: vec![
                    Account::create(USER_INVESTOR_MIKE, 20_000, 1000, 10),
                    Account::create(USER_INVESTOR_JOHN, 5_000, 1000, 10),
                ],
            },
            Pool {
                name: "Minting Pool".to_string(),
                partial: false,
                total: Uint128(20_000),
                accounts: vec![
                    Account::create(USER_MP_RPT1, 11_000, 1000, 10),
                    Account::create(USER_MP_RPT2, 9_000, 1000, 10),
                ],
            },
        ],
    };
    // should we allow multiple RPTs inside the MiningPool ?
    tge.set_shedule(schedule).unwrap();
    assert_eq!(tge.query_schedule().total.u128(), 45_000);
}

#[test]
fn should_not_claim_before_launch() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(25_000),
        pools: vec![Pool {
            name: "Investors".to_string(),
            partial: false,
            total: Uint128(25_000),
            accounts: vec![
                Account::create(USER_INVESTOR_MIKE, 20_000, 1000, 10),
                Account::create(USER_INVESTOR_JOHN, 5_000, 1000, 10),
            ],
        }],
    };

    tge.set_shedule(schedule).unwrap();
    assert_eq!(tge.query_schedule().total.u128(), 25_000);

    tge.claim_for(USER_INVESTOR_MIKE, 5).unwrap_err();
    tge.launch().unwrap();
}

#[test]
fn should_support_different_schedule_intervals() {
    let mut tge = TGE::default();
    let mike = Account::create(USER_INVESTOR_MIKE, 800_000_000_000_000_000_000, 2000, 10);
    let john = Account::create(USER_INVESTOR_JOHN, 200_000_000_000_000_000_000, 1000, 12);

    let schedule = Schedule {
        total: Uint128(1_500_000_000_000_000_000_000),
        pools: vec![
            Pool {
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(1_000_000_000_000_000_000_000),
                accounts: vec![
                    // total amount will be distributed to the user during the 'duration' period
                    // on equidistant intervals.
                    // In this case, 800 split on (2000/10) = 200 intervals, 4 tokens per each
                    mike.clone(),
                    john.clone(),
                ],
            },
            Pool {
                name: "Minting Pool".to_string(),
                partial: false,
                total: Uint128(500_000_000_000_000_000_000),
                accounts: vec![Account::create(
                    USER_MP_RPT1,
                    500_000_000_000_000_000_000,
                    1000,
                    12,
                )],
            },
        ],
    };

    tge.set_shedule(schedule).unwrap();

    assert_eq!(
        tge.query_schedule().total.u128(),
        1_500_000_000_000_000_000_000
    );

    tge.launch().unwrap();

    // Mike's interval = 10
    // John's interval = 12
    let mike_actual_tokens_per_interval = 4 * TOKEN_DECIMALS;

    // claimed = amount (200e18) / potion size
    // portion size = duration / interval => 1000 / 12 = 83.3333
    // which is rounded to 83 as it's uint
    // so 200e18 / 83.3333333 would be 2.4 (correct)
    // but 200e18 / 83 = 2.409638554216867469 which is bad
    // remainder is 0.009638554216867469 which will be added to another portion
    let johns_rounded_intervals_count = john.duration / john.interval;
    // should be 2_409_638_554_216_867_469
    let john_actual_tokens_per_interval =
        john.amount.u128() / johns_rounded_intervals_count as u128;

    tge.claim_for(USER_INVESTOR_JOHN, john.interval - 1)
        .unwrap(); // 0
    assert_eq!(
        tge.query_balance(USER_INVESTOR_JOHN).u128(),
        john_actual_tokens_per_interval
    );

    tge.claim_for(USER_INVESTOR_MIKE, 2 * mike.interval + 1)
        .unwrap();
    assert_eq!(
        tge.query_balance(USER_INVESTOR_MIKE).u128(),
        mike_actual_tokens_per_interval * 3
    ); // 0, 10, 20

    tge.claim_for(USER_INVESTOR_JOHN, 4 * john.interval + 1)
        .unwrap();
    assert_eq!(
        tge.query_balance(USER_INVESTOR_JOHN).u128(),
        john_actual_tokens_per_interval * 5
    );

    tge.claim_for(
        USER_INVESTOR_JOHN,
        john.interval * (johns_rounded_intervals_count + 1),
    )
    .unwrap();
    assert_eq!(
        tge.query_balance(USER_INVESTOR_JOHN).u128(),
        john.amount.u128()
    );
}

#[test]
fn should_support_account_in_multiple_pools() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(45_000),
        pools: vec![
            Pool {
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(25_000),
                accounts: vec![
                    Account::create(USER_INVESTOR_MIKE, 20_000, 1000, 10),
                    Account::create(USER_INVESTOR_JOHN, 5_000, 1000, 10),
                ],
            },
            Pool {
                name: "Minting Pool".to_string(),
                partial: false,
                total: Uint128(20_000),
                accounts: vec![
                    Account::create(USER_INVESTOR_MIKE, 11_000, 1000, 10),
                    Account::create(USER_MP_RPT2, 9_000, 1000, 10),
                ],
            },
        ],
    };
    tge.set_shedule(schedule).unwrap();
    let _ = &tge.query_schedule();
    tge.launch().unwrap();
    tge.claim_for(USER_INVESTOR_MIKE, 1000).unwrap();
    let mikes_balance = tge.query_balance(USER_INVESTOR_MIKE);
    assert_eq!(mikes_balance.u128(), 31_000);
}
