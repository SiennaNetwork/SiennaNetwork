use lend_shared::fadroma::{
    admin::{
        QueryMsg as AdminQuery,
        QueryResponse as AdminResponse
    },
    Uint256, StdError, Decimal256,
    testing::{mock_dependencies, mock_env}
};

use lend_shared::interfaces::interest_model::{
    HandleMsg, InitMsg, QueryMsg, QueryResponse
};

use crate::{init, handle, query, DefaultImpl};

#[test]
fn proper_initialization() {
    let ref mut deps = mock_dependencies(32, &[]);

    let msg = InitMsg {
        admin: Some("owner0000".into()),
        base_rate: Decimal256::percent(10),
        interest_multiplier: Decimal256::percent(10),
    };

    // we can just call .unwrap() to assert this was a success
    let res = init(deps, mock_env("addr0000", &[]), msg, DefaultImpl).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps, QueryMsg::Config {}, DefaultImpl).unwrap();

    match res {
        QueryResponse::Config { config } => {
            assert_eq!("0.1", &config.base_rate.to_string());
            assert_eq!("0.1", &config.interest_multiplier.to_string());
        },
        _ => panic!("Expected QueryResponse::Config")
    }

    let res = query(deps, QueryMsg::Admin(AdminQuery::Admin {}), DefaultImpl).unwrap();

    match res {
        QueryResponse::Admin(resp) => {
            match resp {
                AdminResponse::Admin { address } => {
                    assert_eq!("owner0000", address.as_str());
                }
            }
        },
        _ => panic!("Expected QueryResponse::Admin")
    }

    let query_msg = QueryMsg::BorrowRate {
        market_balance: Uint256::from(1000000u128),
        total_liabilities: Decimal256::from_uint256(500000u128),
        total_reserves: Decimal256::from_uint256(100000u128),
    };

    let res = query(deps, query_msg, DefaultImpl).unwrap();
    // utilization_ratio = 0.35714285714285714
    // borrow_rate = 0.035714285 + 0.1
    match res {
        QueryResponse::BorrowRate { borrow_rate } => {
            assert_eq!("0.135714285714285714", &borrow_rate.to_string());
        },
        _ => panic!("QueryResponse::BorrowRate")
    }


    let query_msg = QueryMsg::BorrowRate {
        market_balance: Uint256::zero(),
        total_liabilities: Decimal256::zero(),
        total_reserves: Decimal256::zero(),
    };

    let res = query(deps, query_msg, DefaultImpl).unwrap();
    match res {
        QueryResponse::BorrowRate { borrow_rate } => {
            assert_eq!("0.1", &borrow_rate.to_string());
        },
        _ => panic!("QueryResponse::BorrowRate")
    }
}

#[test]
fn update_config() {
    let ref mut deps = mock_dependencies(32, &[]);

    let msg = InitMsg {
        admin: Some("owner0000".into()),
        base_rate: Decimal256::percent(10),
        interest_multiplier: Decimal256::percent(10),
    };

    init(deps, mock_env("addr0000", &[]), msg, DefaultImpl).unwrap();

    // update owner
    let msg = HandleMsg::UpdateConfig {
        base_rate: None,
        interest_multiplier: None,
    };

    let res = handle(deps, mock_env("owner0000", &[]), msg, DefaultImpl).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(deps, QueryMsg::Config {}, DefaultImpl).unwrap();
    match res {
        QueryResponse::Config { config } => {
            assert_eq!("0.1", &config.base_rate.to_string());
            assert_eq!("0.1", &config.interest_multiplier.to_string());
        },
        _ => panic!("Expected QueryResponse::Config")
    }

    // Unauthorized err
    let msg = HandleMsg::UpdateConfig {
        base_rate: Some(Decimal256::percent(1)),
        interest_multiplier: Some(Decimal256::percent(1)),
    };

    let res = handle(deps, mock_env("unauth", &[]), msg, DefaultImpl);
    assert_eq!(res.unwrap_err(), StdError::unauthorized());
}
