use std::{
    convert::TryInto,
    str::FromStr
};

use amm_shared::{
    fadroma::{
        ContractLink,
        cosmwasm_std::{
            Decimal, HumanAddr, Uint128, StdError, to_binary
        },
        ensemble::MockEnv,
    },
    TokenPairAmount,
    msg
};

use crate::setup::{Amm, USERS};

#[test]
fn provide_liquidity_empty_pool() {
    let mut amm = Amm::new();
    let pair = amm.get_pairs().drain(0..1).next().unwrap();

    amm.increase_allowances(&pair);

    let deposit_0 = Uint128(800);
    let deposit_1 = Uint128(200);

    amm.ensemble.execute(
        &msg::exchange::HandleMsg::AddLiquidity {
            deposit: TokenPairAmount {
                pair: pair.pair,
                amount_0: deposit_0,
                amount_1: deposit_1
            },
            slippage_tolerance: Some(Decimal::from_str("0.5").unwrap())
        },
        MockEnv::new(USERS[0], pair.contract.clone())
    ).unwrap();

    let result: msg::exchange::QueryMsgResponse = amm.ensemble.query(
        pair.contract.address.clone(),
        msg::exchange::QueryMsg::PairInfo
    ).unwrap();

    match result {
        msg::exchange::QueryMsgResponse::PairInfo { amount_0, amount_1, total_liquidity, .. } => {
            assert_eq!(amount_0, deposit_0);
            assert_eq!(amount_1, deposit_1);
            assert_eq!(total_liquidity, Uint128(400));
        }
    }

    let balance = amm.get_lp_balance(USERS[0], pair.contract.address);
    assert_eq!(balance.u128(), 400u128);
}

#[test]
fn provide_liquidity_slippage_tolerance() {
    let mut amm = Amm::new();
    let pair = amm.get_pairs().drain(0..1).next().unwrap();

    amm.increase_allowances(&pair);

    amm.ensemble.execute(
        &msg::exchange::HandleMsg::AddLiquidity {
            deposit: TokenPairAmount {
                pair: pair.pair.clone(),
                amount_0: Uint128(8000),
                amount_1: Uint128(2000)
            },
            slippage_tolerance: Some(Decimal::from_str("0.5").unwrap())
        },
        MockEnv::new(USERS[0], pair.contract.clone())
    ).unwrap();

    let balance = amm.get_lp_balance(USERS[0], pair.contract.address.clone());
    assert_eq!(balance.u128(), 4000u128);

    let token_0: ContractLink<HumanAddr> = pair.pair.0.clone().try_into().unwrap();

    amm.ensemble.execute(&msg::snip20::HandleMsg::Send {
            recipient: pair.contract.address.clone(),
            recipient_code_hash: None,
            amount: Uint128(12000),
            memo: None,
            padding: None,
            msg: Some(to_binary(&msg::exchange::ReceiverCallbackMsg::Swap {
                expected_return: None,
                to: None
            }).unwrap()),
        },
        MockEnv::new(USERS[1], token_0)
    ).unwrap();

    let result: msg::exchange::QueryMsgResponse = amm.ensemble.query(
        pair.contract.address.clone(),
        msg::exchange::QueryMsg::PairInfo
    ).unwrap();

    match result {
        msg::exchange::QueryMsgResponse::PairInfo { amount_0, amount_1, total_liquidity, .. } => {
            assert_eq!(amount_0.u128(), 19998u128);
            assert_eq!(amount_1.u128(), 801u128);
            assert_eq!(total_liquidity, Uint128(4000));
        }
    }

    for i in 1..=8 {
        let result = amm.ensemble.execute(
            &msg::exchange::HandleMsg::AddLiquidity {
                deposit: TokenPairAmount {
                    pair: pair.pair.clone(),
                    amount_0: Uint128(80),
                    amount_1: Uint128(20)
                },
                slippage_tolerance: Some(Decimal::from_str(&format!("0.{}", i)).unwrap())
            },
            MockEnv::new(USERS[2], pair.contract.clone())
        ).unwrap_err();
    
        match result {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "Operation exceeds max slippage tolerance");
            }
            _ => panic!("Expecting StdError::GenericErr")
        }
    }

    amm.ensemble.execute(
        &msg::exchange::HandleMsg::AddLiquidity {
            deposit: TokenPairAmount {
                pair: pair.pair.clone(),
                amount_0: Uint128(80),
                amount_1: Uint128(20)
            },
            slippage_tolerance: Some(Decimal::from_str("0.9").unwrap())
        },
        MockEnv::new(USERS[2], pair.contract.clone())
    ).unwrap();

    let result: msg::exchange::QueryMsgResponse = amm.ensemble.query(
        pair.contract.address.clone(),
        msg::exchange::QueryMsg::PairInfo
    ).unwrap();

    match result {
        msg::exchange::QueryMsgResponse::PairInfo { amount_0, amount_1, total_liquidity, .. } => {
            assert_eq!(amount_0.u128(), 20078u128);
            assert_eq!(amount_1.u128(), 821u128);
            assert_eq!(total_liquidity, Uint128(4016));
        }
    }
}
