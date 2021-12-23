use std::{
    convert::TryInto,
    str::FromStr
};

use amm_shared::{
    fadroma::{
        ContractLink,
        cosmwasm_std::{
            Decimal, HumanAddr, Uint128
        },
        ensemble::MockEnv,
    },
    TokenPairAmount,
    msg
};

use crate::setup::{Amm, USER};

#[test]
fn provide_liquidity_empty_pool() {
    let mut amm = Amm::new();
    let pair = amm.get_pairs().drain(0..1).next().unwrap();

    for token in pair.pair.into_iter() {
        let token: ContractLink<HumanAddr> = token.try_into().unwrap();

        amm.ensemble.execute(
            &msg::snip20::HandleMsg::IncreaseAllowance {
                spender: pair.contract.address.clone(),
                amount: Uint128(u128::MAX),
                expiration: None,
                padding: None
            },
            MockEnv::new(USER, token)
        ).unwrap();
    }

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
        MockEnv::new(USER, pair.contract.clone())
    ).unwrap();

    let result: msg::exchange::QueryMsgResponse = amm.ensemble.query(
        pair.contract.address,
        msg::exchange::QueryMsg::PairInfo
    ).unwrap();

    match result {
        msg::exchange::QueryMsgResponse::PairInfo { amount_0, amount_1, total_liquidity, .. } => {
            assert_eq!(amount_0, deposit_0);
            assert_eq!(amount_1, deposit_1);
            assert_eq!(total_liquidity, Uint128(400));
        }
    }
}
