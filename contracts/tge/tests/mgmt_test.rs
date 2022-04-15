#![allow(dead_code)]
use fadroma::{
    admin,
    cosmwasm_std::{HumanAddr, StdError, Uint128},
    ensemble::MockEnv,
};
use sienna_mgmt as mgmt;
use sienna_schedule::{Account, Pool, Schedule};

use crate::setup::{ADMIN, TGE};

#[test]
fn init() {
    let tge = TGE::new(false);

    let admin: HumanAddr = tge
        .ensemble
        .query(
            tge.mgmt.address.clone(),
            mgmt::QueryMsg::Admin(admin::QueryMsg::Admin {}),
        )
        .unwrap();

    assert_eq!(admin.0, ADMIN);

    let config: mgmt::ConfigResponse = tge
        .ensemble
        .query(tge.mgmt.address.clone(), mgmt::QueryMsg::Config {})
        .unwrap();

    assert_eq!(config.launched, None);
    assert_eq!(config.token, tge.token);

    let expected = Schedule::<HumanAddr>::new(&[Pool::partial("TEST", 25, &[])]);
    let schedule: Schedule<HumanAddr> = tge
        .ensemble
        .query(tge.mgmt.address, mgmt::QueryMsg::Schedule {})
        .unwrap();

    assert_eq!(schedule, expected);
}

#[test]
fn configure() {
    let mut tge = TGE::new(false);

    let original = sienna_schedule::Schedule::new(&[Pool::full("original", &[])]);
    tge.ensemble
        .execute(
            &mgmt::HandleMsg::Configure {
                schedule: original.clone(),
            },
            tge.get_mgmt_env_as_admin(),
        )
        .unwrap();

    let schedule: Schedule<HumanAddr> = tge
        .ensemble
        .query(tge.mgmt.address.clone(), mgmt::QueryMsg::Schedule {})
        .unwrap();

    assert_eq!(schedule, original);

    let err = tge
        .ensemble
        .execute(
            &mgmt::HandleMsg::Configure {
                schedule: original.clone(),
            },
            tge.get_mgmt_env("humpty dumpty"),
        )
        .unwrap_err();

    assert_eq!(err, StdError::unauthorized());

    let src = include_str!("../../../settings/schedule.json");
    let prod: Schedule<HumanAddr> = serde_json::from_str(src).unwrap();

    tge.ensemble
        .execute(
            &mgmt::HandleMsg::Configure {
                schedule: prod.clone(),
            },
            tge.get_mgmt_env_as_admin(),
        )
        .unwrap();

    let schedule: Schedule<HumanAddr> = tge
        .ensemble
        .query(tge.mgmt.address.clone(), mgmt::QueryMsg::Schedule {})
        .unwrap();

    assert_eq!(schedule, prod);

    tge.ensemble
        .execute(&mgmt::HandleMsg::Launch {}, tge.get_mgmt_env_as_admin())
        .unwrap();
}

#[test]
fn launch() {
    let mut tge = TGE::new(false);

    let launch_time = 1000;

    tge.ensemble.block().time = launch_time;

    let err = tge
        .ensemble
        .execute(&mgmt::HandleMsg::Launch {}, tge.get_mgmt_env("rando"))
        .unwrap_err();

    assert_eq!(err, StdError::unauthorized());

    tge.ensemble
        .execute(&mgmt::HandleMsg::Launch {}, tge.get_mgmt_env_as_admin())
        .unwrap();

    let config: mgmt::ConfigResponse = tge
        .ensemble
        .query(tge.mgmt.address.clone(), mgmt::QueryMsg::Config {})
        .unwrap();

    assert_eq!(config.launched, Some(launch_time));
    assert_eq!(config.token, tge.token);

    let schedule = sienna_schedule::Schedule::new(&[Pool::full("", &[])]);
    let err = tge
        .ensemble
        .execute(
            &mgmt::HandleMsg::Configure { schedule },
            tge.get_mgmt_env_as_admin(),
        )
        .unwrap_err();

    assert_eq!(err, StdError::generic_err(mgmt::MGMTError!(UNDERWAY)));

    let err = tge
        .ensemble
        .execute(&mgmt::HandleMsg::Launch {}, tge.get_mgmt_env_as_admin())
        .unwrap_err();

    assert_eq!(err, StdError::generic_err(mgmt::MGMTError!(UNDERWAY)));
}

#[test]
fn claim() {
    let mut tge = TGE::new(false);

    let claimant = "claimant";
    let launch_time = 1000;
    let interval = 180;
    let amount = Uint128(25);

    tge.add_account(
        "TEST".into(),
        Account {
            name: claimant.into(),
            address: claimant.into(),
            amount,
            cliff: Uint128::zero(),
            start_at: 0,
            interval,
            duration: 0,
        },
    )
    .unwrap();

    for addr in ["rando", claimant] {
        let err = tge
            .ensemble
            .execute(&mgmt::HandleMsg::Claim {}, tge.get_mgmt_env(addr))
            .unwrap_err();

        assert_eq!(err, StdError::generic_err(mgmt::MGMTError!(PRELAUNCH)));
    }

    tge.ensemble.block().time = launch_time;

    tge.ensemble
        .execute(&mgmt::HandleMsg::Launch {}, tge.get_mgmt_env_as_admin())
        .unwrap();

    let err = tge
        .ensemble
        .execute(&mgmt::HandleMsg::Claim {}, tge.get_mgmt_env("rando"))
        .unwrap_err();

    assert_eq!(err, StdError::generic_err(mgmt::MGMTError!(NOTHING)));

    tge.ensemble.block().time = launch_time + interval;

    tge.ensemble
        .execute(&mgmt::HandleMsg::Claim {}, tge.get_mgmt_env(claimant))
        .unwrap();

    let balance = tge.query_balance(claimant);
    assert_eq!(balance, amount);

    let mut history: mgmt::HistoryResponse = tge
        .ensemble
        .query(
            tge.mgmt.address,
            mgmt::QueryMsg::History {
                pagination: mgmt::Pagination {
                    start: 0,
                    limit: 50,
                },
            },
        )
        .unwrap();

    assert_eq!(history.total, 1);
    assert_eq!(history.entries.len(), 1);

    let entry = history.entries.pop().unwrap();
    assert_eq!(entry.claimant.0, claimant);
    assert_eq!(entry.timestamp, launch_time + interval);
    assert_eq!(entry.amount, amount);
}

#[test]
fn prod_schedule_simulation() {
    let mut tge = TGE::new(false);

    let launch_time = 2;

    let src = include_str!("../../../settings/schedule.json");
    let schedule: Schedule<HumanAddr> = serde_json::from_str(src).unwrap();

    tge.ensemble
        .execute(
            &mgmt::HandleMsg::Configure {
                schedule: schedule.clone(),
            },
            tge.get_mgmt_env_as_admin(),
        )
        .unwrap();

    tge.ensemble.block().time = launch_time;

    tge.ensemble
        .execute(&mgmt::HandleMsg::Launch {}, tge.get_mgmt_env_as_admin())
        .unwrap();

    for pool in schedule.pools.iter() {
        for account in pool.accounts.iter() {
            let address = account.address.clone();
            let portion_count = account.portion_count();
            let portion_size = account.portion_size();
            let remainder = account.remainder();

            /*
            println!(
                "\naccount: {} {} {} {}",
                &account.name, account.start_at, account.interval, account.duration
            );
            println!(
                "amounts: {} = {} + {} * {} + {}",
                account.amount, account.cliff, portion_count, portion_size, remainder
            );
            */

            assert_eq!(
                account.cliff.u128() + portion_count as u128 * portion_size + remainder,
                account.amount.u128(),
                "(cliff + portions + remainder) should equal account total"
            );

            if account.start_at > 0 {
                //funds are not unlocked before `start_at`
                let progress: mgmt::ProgressResponse = tge
                    .ensemble
                    .query(
                        tge.mgmt.address.clone(),
                        mgmt::QueryMsg::Progress {
                            address: address.clone(),
                            time: account.start_at - 1,
                        },
                    )
                    .unwrap();

                assert_eq!(progress.unlocked, Uint128::zero());
                assert_eq!(progress.claimed, Uint128::zero());
            }

            let progress: mgmt::ProgressResponse = tge
                .ensemble
                .query(
                    tge.mgmt.address.clone(),
                    mgmt::QueryMsg::Progress {
                        address: address.clone(),
                        time: account.start_at + account.interval,
                    },
                )
                .unwrap();

            if account.cliff > Uint128::zero() {
                // cliff
                assert_eq!(progress.unlocked, account.cliff);
                assert_eq!(progress.claimed, Uint128::zero());
            } else {
                // first portion
                assert_eq!(progress.unlocked, Uint128::from(account.portion_size()));
                assert_eq!(progress.claimed, Uint128::zero());
            }

            let progress: mgmt::ProgressResponse = tge
                .ensemble
                .query(
                    tge.mgmt.address.clone(),
                    mgmt::QueryMsg::Progress {
                        address,
                        time: account.start_at + account.duration + account.interval,
                    },
                )
                .unwrap();

            assert_eq!(progress.unlocked, account.amount);
            assert_eq!(progress.claimed, Uint128::zero());
        }
    }

    let mut num_txs = 0;

    for pool in schedule.pools {
        for account in pool.accounts {
            let time = account.end() + account.interval;

            tge.ensemble.block().time = time;
            tge.ensemble.block().height = time / 5;

            tge.ensemble
                .execute(
                    &mgmt::HandleMsg::Claim {},
                    MockEnv::new(account.address.clone(), tge.mgmt.clone()),
                )
                .unwrap();

            let balance = tge.query_balance(&account.address.0);
            assert_eq!(balance, account.amount);

            num_txs += 1;
        }
    }

    let history: mgmt::HistoryResponse = tge
        .ensemble
        .query(
            tge.mgmt.address,
            mgmt::QueryMsg::History {
                pagination: mgmt::Pagination {
                    start: 0,
                    limit: 50,
                },
            },
        )
        .unwrap();

    assert_eq!(history.total, num_txs);
}
