
use sienna_rewards::{
    fadroma::{
        cosmwasm_std::Uint128
    }
};

use crate::setup::{Amm, USERS};

#[test]
fn should_deposit_rewards() {
    let mut amm = Amm::new();
    amm.deposit_lp_into_rewards(USERS[0], Uint128(100));
    amm.set_rewards_viewing_key(USERS[0], "whatever".into());

    let balance = amm.get_rewards_staked(USERS[0], "whatever".into());
    assert_eq!(100, balance.u128());
}
