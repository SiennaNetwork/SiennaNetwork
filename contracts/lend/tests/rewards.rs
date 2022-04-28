use lend_shared::{
    fadroma::{
        cosmwasm_std::{
            Uint128, Env, Binary, InitResponse,
            HandleResponse, StdResult, StdError,
            from_binary
        },
        ensemble::{ContractHarness, MockDeps, MockEnv},
        snip20_impl::msg as snip20,
        Decimal256, Uint256, ContractLink, one_token
    }
};
use sienna_rewards as rewards;

use crate::setup::{Lend, ADMIN};

struct Rewards;

impl ContractHarness for Rewards {
    fn init(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<InitResponse> {
        rewards::init(deps, env, from_binary(&msg)?)
    }
    fn handle(&self, deps: &mut MockDeps, env: Env, msg: Binary) -> StdResult<HandleResponse> {
        rewards::handle(deps, env, from_binary(&msg)?)
    }
    fn query(&self, deps: &MockDeps, msg: Binary) -> StdResult<Binary> {
        rewards::query(deps, from_binary(&msg)?)
    }
}

#[test]
fn deposit_to_rewards() {
    let mut lend = Lend::default();
    let joe = "joe";

    let market_token = lend.new_underlying_token("SSCRT", 6).unwrap();
    let reward_token = lend.new_underlying_token("SIENNA", 18).unwrap();

    let market = lend
        .whitelist_market(market_token, Decimal256::percent(90), None, None)
        .unwrap()
        .contract;

    let amount = Uint128(500 * one_token(6));

    lend.prefund_and_deposit(joe, amount, market.address.clone());

    let rewards = lend.ensemble.register(Box::new(Rewards));
    let rewards = lend.ensemble
        .instantiate(
            rewards.id,
            &sienna_rewards::Init {
                admin: Some(ADMIN.into()),
                config: sienna_rewards::config::RewardsConfig {
                    bonding: None,
                    lp_token: Some(market.clone()),
                    reward_token: Some(reward_token),
                    reward_vk: Some("whatever".to_string()),
                    timekeeper: Some(ADMIN.into()),
                },
                governance_config: None,
            },
            MockEnv::new(
                ADMIN,
                ContractLink {
                    address: "rewards".into(),
                    code_hash: rewards.code_hash,
                },
            ),
        )
        .unwrap();

    let invalid_deposit = amount + Uint128(1);
    let err = lend.ensemble.execute(
        &snip20::HandleMsg::Send {
            recipient: rewards.address.clone(),
            recipient_code_hash: Some(rewards.code_hash.clone()),
            amount: invalid_deposit,
            msg: None,
            memo: None,
            padding: None,
        },
        MockEnv::new(joe, market.clone()),
    )
    .unwrap_err();

    assert_eq!(err, StdError::generic_err(format!(
        "insufficient funds: balance={}, required={}",
        amount,
        invalid_deposit
    )));

    lend.ensemble.execute(
        &snip20::HandleMsg::Send {
            recipient: rewards.address.clone(),
            recipient_code_hash: Some(rewards.code_hash.clone()),
            amount,
            msg: None,
            memo: None,
            padding: None,
        },
        MockEnv::new(joe, market.clone()),
    )
    .unwrap();

    let info = lend.account_info(joe, market.address.clone());
    
    assert_eq!(info.borrow_balance, Uint256::zero());
    assert_eq!(info.sl_token_balance, Uint256::zero());

    lend.ensemble.execute(
        &rewards::Handle::Rewards(
            rewards::handle::RewardsHandle::Withdraw { amount }
        ),
        MockEnv::new(joe, rewards)
    ).unwrap();

    let info = lend.account_info(joe, market.address);
    
    assert_eq!(info.borrow_balance, Uint256::zero());
    assert_eq!(info.sl_token_balance, amount.into());
}
