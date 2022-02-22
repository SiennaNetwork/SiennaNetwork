use amm_shared::msg::rewards::{Query, RewardsQuery};
use sienna_rewards::Response;

use crate::setup::Amm;

#[test]
fn rewards_init() {
    let amm = Amm::new();

    let _result: Response = amm
        .ensemble
        .query(amm.rewards.address, Query::Rewards(RewardsQuery::Config))
        .unwrap();
}

#[test]
fn deposit_rewards() {
    let _amm = Amm::new();
}
