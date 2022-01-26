use std::str::FromStr;

use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::MockEnv,
        snip20_impl::msg::{HandleMsg as Snip20HandleMsg, InitialBalance},
        to_binary, Decimal256, Permit, Uint128, Uint256,
    },
    interfaces::{
        market::{self, MarketPermissions},
        overseer,
    },
};

use crate::setup::Lend;
use crate::ADMIN;

#[test]
fn deposit_and_mint() {
    let deposit_amount = Uint128(100_000);
    let exchange_rate = Decimal256::from_uint256(50_000u128).unwrap();
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();
    lend.prefund_user("borrower", Uint128(5 * one_token(6)), underlying_1.clone());

    let market = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(50),
            Some(exchange_rate),
        )
        .unwrap();

    // deposit should fail if insufficient funds
    let res = lend.ensemble.execute(
        &Snip20HandleMsg::Send {
            recipient: market.contract.address.clone(),
            recipient_code_hash: None,
            amount: deposit_amount,
            memo: None,
            padding: None,
            msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
        },
        MockEnv::new("borrower_2", underlying_1.clone()),
    );
    assert!(res.unwrap_err().to_string().contains("insufficient funds"));

    lend.ensemble
        .execute(
            &Snip20HandleMsg::Send {
                recipient: market.contract.address.clone(),
                recipient_code_hash: None,
                amount: deposit_amount,
                memo: None,
                padding: None,
                msg: Some(to_binary(&market::ReceiverCallbackMsg::Deposit {}).unwrap()),
            },
            MockEnv::new("borrower", underlying_1.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    "borrower",
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();

    assert_eq!(
        res.sl_token_balance,
        Uint256::from(deposit_amount)
            .decimal_div(exchange_rate)
            .unwrap()
    );
}

#[test]
fn redeem_basic() {
    let redeem_tokens = Uint128(10_000);
    let exchange_rate = Decimal256::from_uint256(50_000u128).unwrap();
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();
    lend.prefund_user("borrower", Uint128(5 * one_token(6)), underlying_1.clone());
    lend.prefund_user(ADMIN, Uint128(1000 * one_token(6)), underlying_1.clone());

    let market = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(50),
            Some(exchange_rate),
        )
        .unwrap();

    let res = lend.ensemble.execute(
        &market::HandleMsg::RedeemToken {
            burn_amount: Uint256::from(redeem_tokens),
        },
        MockEnv::new("borrower", market.contract.clone()),
    );
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("The protocol has an insufficient amount of the underlying asset"));

    // fund the market
    lend.ensemble
        .execute(
            &Snip20HandleMsg::Transfer {
                recipient: market.contract.address.clone(),
                amount: Uint128(1000 * one_token(6)),
                memo: None,
                padding: None,
            },
            MockEnv::new(ADMIN, underlying_1.clone()),
        )
        .unwrap();

    // lend.ensemble
    //     .execute(
    //         &market::HandleMsg::RedeemToken {
    //             burn_amount: Uint256::from(redeem_tokens),
    //         },
    //         MockEnv::new("borrower", market.contract.clone()),
    //     )
    //     .unwrap();
}
