use fadroma::{ensemble::MockEnv, ContractLink, HumanAddr, Uint128, CanonicalAddr};
use sienna_mgmt::ConfigResponse;
use sienna_rpt::LinearMap;
use sienna_schedule::{Account, Schedule, Pool};

use crate::setup::{ADMIN, TGE, DEFAULT_EPOCH_START};

const USER_INVESTOR_MIKE: &str = "Mike";
const USER_INVESTOR_JOHN: &str = "John";
const USER_MP_RPT1: &str = "RTP1";
const USER_MP_RPT2: &str = "RTP2";
const TOKEN1: &str = "secret1TOKEN1";
const TOKEN2: &str = "secret1TOKEN2";

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
            &sienna_rpt::HandleMsg::SetDistribution {
                distribution: updated_distribution.clone(),
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
            &sienna_rpt::HandleMsg::SetDistribution {
                distribution: updated_distribution.clone(),
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
            &sienna_rpt::HandleMsg::SetDistribution {
                distribution: updated_distribution.clone(),
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
            )
            .time(1978897420),
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
        pools: vec![
            Pool{
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(20_000),
                accounts: vec![
                    Account {
                        name: USER_INVESTOR_MIKE.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_MIKE),
                        amount: Uint128(2_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    },
                    Account {
                        name: USER_INVESTOR_JOHN.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_JOHN),
                        amount: Uint128(5_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    }
                ]
            }
        ]
    };
    // wrong sum of totals inside pool
    tge.set_shedule(schedule).unwrap_err();
    //assert_eq!(tge.query_schedule().total.u128(), 25_000);
    
}


#[test]
fn should_set_schedule_snigle_pool() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(25_000),
        pools: vec![
            Pool{
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(25_000),
                accounts: vec![
                    Account {
                        name: USER_INVESTOR_MIKE.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_MIKE),
                        amount: Uint128(20_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    },
                    Account {
                        name: USER_INVESTOR_JOHN.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_JOHN),
                        amount: Uint128(5_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    }
                ]
            }
        ]
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
            Pool{
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(25_000),
                accounts: vec![
                    Account {
                        name: USER_INVESTOR_MIKE.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_MIKE),
                        amount: Uint128(20_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    },
                    Account {
                        name: USER_INVESTOR_JOHN.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_JOHN),
                        amount: Uint128(5_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    }
                ]
            }, 
            Pool{
                name: "Minting Pool".to_string(),
                partial: false,
                total: Uint128(20_000),
                accounts: vec![
                    Account {
                        name: USER_MP_RPT1.to_string(),
                        address: HumanAddr::from(USER_MP_RPT1),
                        amount: Uint128(11_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    },
                    Account {
                        name: USER_MP_RPT2.to_string(),
                        address: HumanAddr::from(USER_MP_RPT2),
                        amount: Uint128(9_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    }
                ]
            }, 
            
        ]
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
            Pool{
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(25_000),
                accounts: vec![
                    Account {
                        name: USER_INVESTOR_MIKE.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_MIKE),
                        amount: Uint128(20_000),
                        cliff: Uint128(0),
                        duration: 1000, 
                        interval: 10,
                        start_at: 0
                    },
                    Account {
                        name: USER_INVESTOR_JOHN.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_JOHN),
                        amount: Uint128(5_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    }
                ]
            }, 
            Pool{
                name: "Minting Pool".to_string(),
                partial: false,
                total: Uint128(20_000),
                accounts: vec![
                    Account {
                        name: USER_MP_RPT1.to_string(),
                        address: HumanAddr::from(USER_MP_RPT1),
                        amount: Uint128(11_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    }, 
                    Account {
                        name: USER_MP_RPT2.to_string(),
                        address: HumanAddr::from(USER_MP_RPT2),
                        amount: Uint128(9_000),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 10,
                        start_at: 0
                    }
                ]
            }, 
            
        ]
    };
    // should we allow multiple RPTs inside the MiningPool ?
    tge.set_shedule(schedule).unwrap();
    assert_eq!(tge.query_schedule().total.u128(), 45_000);
}


#[test]
fn should_support_different_schedule_intervals() {
    let mut tge = TGE::default();

    let schedule = Schedule {
        total: Uint128(1_500),
        pools: vec![
            Pool{
                name: "Investors".to_string(),
                partial: false,
                total: Uint128(1_000),
                accounts: vec![
                    Account {
                        name: USER_INVESTOR_MIKE.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_MIKE),

                        // total amount will be distributed to the user during the 'duration' period
                        // on equidistant intervals.
                        // In this case, 800 split on (2000/10) = 200 intervals, 4 tokens per each
                        amount: Uint128(800),
                        cliff: Uint128(0),
                        duration: 2000,
                        interval: 10,
                        start_at: 0
                    },
                    Account {
                        name: USER_INVESTOR_JOHN.to_string(),
                        address: HumanAddr::from(USER_INVESTOR_JOHN),
                        amount: Uint128(200),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 12,
                        start_at: 0
                    }
                ]
            }, 
            Pool{
                name: "Minting Pool".to_string(),
                partial: false,
                total: Uint128(500),
                accounts: vec![
                    Account {
                        name: USER_MP_RPT1.to_string(),
                        address: HumanAddr::from(USER_MP_RPT1),
                        amount: Uint128(500),
                        cliff: Uint128(0),
                        duration: 1000,
                        interval: 14,
                        start_at: 0
                    },
                ]
            }, 
            
        ]
    };
    tge.set_shedule(schedule).unwrap();

    assert_eq!(tge.query_schedule().total.u128(), 1_500);

    tge.ensemble.execute(
        &sienna_mgmt::HandleMsg::Launch {}, 
        tge.get_mgmt_env_as_admin().time(DEFAULT_EPOCH_START)
    ).unwrap();

    // User Mike 
    let mut actual_tokens_per_interval = 4;
    
    tge.ensemble.execute(
        &sienna_mgmt::HandleMsg::Claim {}, 
        tge.get_mgmt_env(USER_INVESTOR_MIKE).time(DEFAULT_EPOCH_START + 21)
    ).unwrap();
        
    assert_eq!(tge.query_balance(USER_INVESTOR_MIKE).u128(), actual_tokens_per_interval * 3); // 0, 10, 20

    tge.ensemble.execute(
        &sienna_mgmt::HandleMsg::Claim {}, 
        tge.get_mgmt_env(USER_INVESTOR_MIKE).time(DEFAULT_EPOCH_START + 21 + 10)
    ).unwrap();
    
    assert_eq!(tge.query_balance(USER_INVESTOR_MIKE).u128(), actual_tokens_per_interval * 4); 

    // User John 
    let mut actual_tokens_per_interval = 2.4; // 200 / 1000 * 12;
    
    tge.ensemble.execute(
        &sienna_mgmt::HandleMsg::Claim {}, 
        tge.get_mgmt_env(USER_INVESTOR_JOHN).time(DEFAULT_EPOCH_START + 11)
    ).unwrap();
        
    assert_eq!(tge.query_balance(USER_INVESTOR_JOHN).u128(), actual_tokens_per_interval * 1); 

    tge.ensemble.execute(
        &sienna_mgmt::HandleMsg::Claim {}, 
        tge.get_mgmt_env(USER_INVESTOR_JOHN).time(DEFAULT_EPOCH_START + 11 + 60)
    ).unwrap();
    
    assert_eq!(tge.query_balance(USER_INVESTOR_JOHN).u128(), actual_tokens_per_interval * 5); 


}
    // different intervals
    // claim before launch
    // claim before interval 
    // claim after X intervals
