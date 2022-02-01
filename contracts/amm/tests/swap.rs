use std::{
    convert::TryInto,
    str::FromStr
};

use amm_shared::{
    fadroma::{
        ContractLink,
        cosmwasm_std::{
            Decimal, HumanAddr, Uint128, StdError, coin, to_binary
        },
        ensemble::MockEnv,
    },
    TokenPairAmount, TokenTypeAmount,
    msg
};

use crate::setup::{Amm, USERS, INITIAL_BALANCE, NATIVE_DENOM, BURNER};

#[test]
fn pair_info() {
    let amm = Amm::new();

    for exchange in amm.get_pairs() {
        let result = amm.ensemble.query(
            exchange.contract.address,
            msg::exchange::QueryMsg::PairInfo
        ).unwrap();

        match result {
            msg::exchange::QueryMsgResponse::PairInfo {
                factory,
                pair,
                amount_0,
                amount_1,
                total_liquidity,
                ..
            } => {
                assert_eq!(factory, amm.factory);
                assert_eq!(pair, exchange.pair);
                assert!(amount_0.is_zero());
                assert!(amount_1.is_zero());
                assert!(total_liquidity.is_zero());
            }
        }
    }
}

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

#[test]
fn withdraw_liquidity() {
    let mut amm = Amm::new();

    let pair = amm.get_pairs().drain(..).next().unwrap();
    amm.increase_allowances(&pair);

    let amount = Uint128(5000000u128);

    let result = amm.ensemble.query(
        pair.contract.address.clone(),
        msg::exchange::QueryMsg::PairInfo
    ).unwrap();

    let lp_token = match result {
        msg::exchange::QueryMsgResponse::PairInfo { liquidity_token, .. } => {
            liquidity_token
        }
    };

    amm.ensemble.execute(
        &msg::exchange::HandleMsg::AddLiquidity {
            deposit: TokenPairAmount {
                pair: pair.pair.clone(),
                amount_0: amount,
                amount_1: amount
            },
            slippage_tolerance: Some(Decimal::from_str("0.5").unwrap())
        },
        MockEnv::new(USERS[0], pair.contract.clone())
    ).unwrap();

    amm.ensemble.execute(
        &msg::snip20::HandleMsg::Send {
            recipient: pair.contract.address.clone(),
            recipient_code_hash: None,
            padding: None,
            amount,
            msg: Some(to_binary(&msg::exchange::ReceiverCallbackMsg::RemoveLiquidity {
                recipient: USERS[0].into()
            }).unwrap()),
            memo: None
        },
        MockEnv::new(USERS[0], lp_token)
    ).unwrap();

    let balance_0 = amm.get_balance(USERS[0], pair.pair.0);
    let balance_1 = amm.get_balance(USERS[0], pair.pair.1);

    assert_eq!(balance_0, INITIAL_BALANCE);
    assert_eq!(balance_1, INITIAL_BALANCE);

    let result = amm.ensemble.query(
        pair.contract.address,
        msg::exchange::QueryMsg::PairInfo
    ).unwrap();

    match result {
        msg::exchange::QueryMsgResponse::PairInfo { amount_0, amount_1, .. } => {
            assert!(amount_0.is_zero());
            assert!(amount_1.is_zero());
        }
    }
}

#[test]
fn swap_native() {
    let mut amm = Amm::new();

    let pair = amm.get_pairs().drain(..).last().unwrap();
    amm.increase_allowances(&pair);

    let amount = Uint128(5000000u128);
    let swap_amount = Uint128(6000000u128);

    amm.ensemble.add_funds(
        USERS[0],
        vec![coin(amount.0, NATIVE_DENOM)]
    );

    amm.ensemble.execute(
        &msg::exchange::HandleMsg::AddLiquidity {
            deposit: TokenPairAmount {
                pair: pair.pair.clone(),
                amount_0: amount,
                amount_1: amount
            },
            slippage_tolerance: Some(Decimal::from_str("0.5").unwrap())
        },
        MockEnv::new(USERS[0], pair.contract.clone())
            .sent_funds(vec![coin(amount.0, NATIVE_DENOM)])
    ).unwrap();

    amm.ensemble.add_funds(
        USERS[1],
        vec![coin(swap_amount.0, NATIVE_DENOM)]
    );

    amm.ensemble.execute(
        &msg::exchange::HandleMsg::Swap {
            offer: TokenTypeAmount {
                token: amm_shared::TokenType::NativeToken { denom: NATIVE_DENOM.into() },
                amount: swap_amount
            },
            to: None,
            expected_return: None,
        },
        MockEnv::new(USERS[1], pair.contract.clone())
            .sent_funds(vec![coin(swap_amount.0, NATIVE_DENOM)])
    ).unwrap();

    let balance_after = amm.get_balance(USERS[1], pair.pair.1.clone());
    assert_eq!(balance_after, Uint128::zero());

    let balance_after = amm.get_balance(USERS[1], pair.pair.0);

    let return_amount = Uint128(2723548);

    assert_eq!((balance_after - INITIAL_BALANCE).unwrap(), return_amount);

    let burner_fee = amm.get_balance(BURNER, pair.pair.1);

    assert_eq!(burner_fee, Uint128(1200));

    let result = amm.ensemble.query(
        pair.contract.address.clone(),
        msg::exchange::QueryMsg::PairInfo
    ).unwrap();

    match result {
        msg::exchange::QueryMsgResponse::PairInfo { amount_0, amount_1, .. } => {
            assert_eq!((amount - return_amount).unwrap(), amount_0);
            assert_eq!((amount + swap_amount - burner_fee).unwrap(), amount_1);
        }
    };
}
