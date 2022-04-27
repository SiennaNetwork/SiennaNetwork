use sienna_rewards::{
    fadroma::{
        ensemble::MockEnv,
        cosmwasm_std::Uint128
    },
    handle::RewardsHandle,
    Handle
};
use amm_shared::TokenType;

use crate::setup::{Amm, USERS, INITIAL_BALANCE};

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
    amm.ensemble.execute(
        &Handle::Rewards(RewardsHandle::Deposit { amount }),
        MockEnv::new(USERS[0], amm.rewards.clone())
    ).unwrap();

    amm.set_rewards_viewing_key(USERS[0], "whatever".into());

    let balance = amm.get_rewards_staked(USERS[0], "whatever".into());
    assert_eq!(100, balance.u128());

    let token = amm.get_rewards_config().lp_token.unwrap();
    let balance = amm.get_balance(USERS[0], TokenType::CustomToken {
        contract_addr: token.address,
        token_code_hash: token.code_hash
    });

    assert_eq!(balance, (INITIAL_BALANCE - amount).unwrap());
}
