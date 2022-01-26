use std::str::FromStr;

use lend_shared::{
    fadroma::{
        ensemble::{ContractHarness, MockDeps, MockEnv},
        from_binary,
        snip20_impl::msg::HandleMsg as Snip20HandleMsg,
        to_binary, Binary, Composable, Decimal256, Env, HandleResponse, HumanAddr, InitResponse,
        Permit, StdError, StdResult, Uint128, Uint256, ViewingKey,
    },
    interfaces::{market::{self, MarketPermissions}, overseer},
};

use crate::setup::Lend;
use crate::ADMIN;

#[test]
fn deposit() {
    let deposit_amount = Uint128(100_000);
    let exchange_rate = Decimal256::from_uint256(50_000u128).unwrap();
    let mut lend = Lend::default();

    let market = lend
        .whitelist_market(lend.underlying_token_one.clone(), Decimal256::percent(50), Some(exchange_rate))
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
        MockEnv::new("borrower_2", lend.underlying_token_one.clone()),
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
            MockEnv::new("borrower", lend.underlying_token_one.clone()),
        )
        .unwrap();

    let res: market::AccountInfo = lend.ensemble.query(
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
    ).unwrap();

    assert_eq!(res.sl_token_balance, Uint256::from(2u128));

}
