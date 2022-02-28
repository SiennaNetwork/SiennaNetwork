use lend_shared::{
    fadroma::{
        decimal::one_token,
        ensemble::MockEnv,
        snip20_impl::msg::QueryAnswer,
        snip20_impl::msg::{HandleMsg as Snip20HandleMsg, QueryMsg as Snip20QueryMsg},
        to_binary, Decimal256, Permit, Uint128, Uint256,
    },
    interfaces::{market, overseer},
};

use crate::setup::{Lend, ADMIN};

const BORROWER: &str = "borrower";

#[test]
fn deposit_and_mint() {
    let deposit_amount = Uint128(100_000);
    let exchange_rate = Decimal256::from_uint256(50_000u128).unwrap();
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();
    lend.prefund_user(BORROWER, Uint128(5 * one_token(6)), underlying_1.clone());

    let market = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(50),
            Some(exchange_rate),
            None,
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
            MockEnv::new(BORROWER, underlying_1.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BORROWER,
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

    let market = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(50),
            Some(exchange_rate),
            None,
        )
        .unwrap();

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    let res = lend.ensemble.execute(
        &market::HandleMsg::RedeemToken {
            burn_amount: Uint256::from(redeem_tokens),
        },
        MockEnv::new(BORROWER, market.contract.clone()),
    );
    assert!(res
        .unwrap_err()
        .to_string()
        .contains("The protocol has an insufficient amount of the underlying asset"));

    lend.prefund_and_deposit(
        BORROWER,
        Uint128(
            Uint256::from(redeem_tokens)
                .decimal_mul(exchange_rate)
                .unwrap()
                .clamp_u128()
                .unwrap(),
        ),
        market.contract.address.clone(),
    );

    lend.ensemble
        .execute(
            &market::HandleMsg::RedeemToken {
                burn_amount: Uint256::from(redeem_tokens),
            },
            MockEnv::new(BORROWER, market.contract.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BORROWER,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();

    assert_eq!(res.sl_token_balance, Uint256::from(0u128));
}

#[test]
fn redeem_underlying_basic() {
    let redeem_tokens = Uint128(10_000);
    let exchange_rate = Decimal256::from_uint256(50_000u128).unwrap();
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();

    let market = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(50),
            Some(exchange_rate),
            None,
        )
        .unwrap();

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    lend.prefund_and_deposit(
        BORROWER,
        Uint128(
            Uint256::from(redeem_tokens)
                .decimal_mul(exchange_rate)
                .unwrap()
                .clamp_u128()
                .unwrap(),
        ),
        market.contract.address.clone(),
    );

    lend.ensemble
        .execute(
            &market::HandleMsg::RedeemUnderlying {
                receive_amount: Uint256::from(redeem_tokens)
                    .decimal_mul(exchange_rate)
                    .unwrap(),
            },
            MockEnv::new(BORROWER, market.contract.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BORROWER,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();

    assert_eq!(res.sl_token_balance, Uint256::from(0u128));
}

#[test]
fn redeem_partial() {
    // deposit 50e18
    let deposit_amount = Uint128(50_000_000_000_000_000_000);
    // redeem partial: 250e8
    let redeem_amount = Uint128(250_000_000_00);
    // exchange rate 1e9
    let exchange_rate = Decimal256::from_uint256(1_000_000_000u128).unwrap();
    let mut lend = Lend::default();

    let underlying_1 = lend.new_underlying_token("ONE", 6).unwrap();

    let market = lend
        .whitelist_market(
            underlying_1.clone(),
            Decimal256::percent(50),
            Some(exchange_rate),
            None,
        )
        .unwrap();

    lend.ensemble
        .execute(
            &overseer::HandleMsg::Enter {
                markets: vec![market.contract.address.clone()],
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    lend.prefund_and_deposit(BORROWER, deposit_amount, market.contract.address.clone());

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BORROWER,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();

    // slToken balance should be 500e8
    assert_eq!(res.sl_token_balance, Uint256::from(500_000_000_00u128));

    // Redeem 250e8
    lend.ensemble
        .execute(
            &market::HandleMsg::RedeemToken {
                burn_amount: Uint256::from(redeem_amount),
            },
            MockEnv::new(BORROWER, market.contract.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend
        .ensemble
        .query(
            market.contract.address.clone(),
            &market::QueryMsg::Account {
                method: Permit::<market::MarketPermissions>::new(
                    BORROWER,
                    vec![market::MarketPermissions::AccountInfo],
                    vec![market.contract.address.clone()],
                    "balance",
                )
                .into(),
                block: None,
            },
        )
        .unwrap();

    assert_eq!(res.sl_token_balance, Uint256::from(redeem_amount));

    lend.ensemble
        .execute(
            &Snip20HandleMsg::SetViewingKey {
                key: "whatever".into(),
                padding: None,
            },
            MockEnv::new(BORROWER, underlying_1.clone()),
        )
        .unwrap();

    // underlying balance should be 25e18
    if let QueryAnswer::Balance { amount } = lend
        .ensemble
        .query(
            underlying_1.address.clone(),
            Snip20QueryMsg::Balance {
                address: BORROWER.into(),
                key: "whatever".into(),
            },
        )
        .unwrap()
    {
        assert_eq!(amount, Uint128(25 * one_token(18)));
    }
}
