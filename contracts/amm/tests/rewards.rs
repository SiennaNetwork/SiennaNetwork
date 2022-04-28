use amm_shared::TokenType;
use sienna_rewards::{
    fadroma::{cosmwasm_std::Uint128, ensemble::MockEnv},
    handle::RewardsHandle,
    Handle,
};

use crate::setup::{Amm, INITIAL_BALANCE, USERS};

#[test]
fn should_deposit_rewards() {
    let mut amm = Amm::new();
    amm.deposit_lp_into_rewards(USERS[0], Uint128(100));
    amm.set_rewards_viewing_key(USERS[0], "whatever".into());

    let balance = amm.get_rewards_staked(USERS[0], "whatever".into());
    assert_eq!(100, balance.u128());
}

#[test]
fn should_deposit_via_transfer() {
    let mut amm = Amm::new();

    let amount = Uint128(100);
    amm.ensemble
        .execute(
            &Handle::Rewards(RewardsHandle::Deposit { amount }),
            MockEnv::new(USERS[0], amm.rewards.clone()),
        )
        .unwrap();

    amm.set_rewards_viewing_key(USERS[0], "whatever".into());

    let balance = amm.get_rewards_staked(USERS[0], "whatever".into());
    assert_eq!(100, balance.u128());

    let token = amm.get_rewards_config().lp_token.unwrap();
    let balance = amm.get_balance(
        USERS[0],
        TokenType::CustomToken {
            contract_addr: token.address,
            token_code_hash: token.code_hash,
        },
    );

    assert_eq!(balance, (INITIAL_BALANCE - amount).unwrap());
}
#[test]
fn query_account_info_permitted() {
    let mut amm = Amm::new();
    amm.deposit_lp_into_rewards(USERS[0], Uint128(100));
    amm.set_rewards_viewing_key(USERS[0], "whatever".into());
    let time = amm.ensemble.block().time;

    let account = amm.query_account_info_permit(USERS[0], time);
    assert_eq!(100, account.staked.u128());
}
#[test]
fn query_balance_permitted() {
    let mut amm = Amm::new();
    amm.deposit_lp_into_rewards(USERS[0], Uint128(100));
    amm.set_rewards_viewing_key(USERS[0], "whatever".into());

    let balance = amm.query_balance_with_permit(USERS[0]);
    assert_eq!(100, balance.u128());
}
