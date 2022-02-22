use std::convert::TryInto;

use sienna_rewards::{
    ensemble::MockEnv,
    gov::handle::GovernanceHandle,
    gov::{poll_metadata::PollMetadata, query::GovernanceQuery},
    handle::RewardsHandle,
    query::RewardsQuery,
    Handle, Query, Response, Uint128,
};

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
    let mut amm = Amm::new();
    amm.ensemble
        .execute(
            &Handle::Rewards(RewardsHandle::Deposit {
                amount: Uint128(3600),
            }),
            MockEnv::new(
                "user_b".to_string(),
                amm.rewards.to_owned().try_into().unwrap(),
            ),
        )
        .unwrap();
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll {
                meta: PollMetadata {
                    description: "tesekfekjfefeft".into(),
                    title: "testtiiing".into(),
                    poll_type: sienna_rewards::gov::poll_metadata::PollType::Other,
                },
            }),
            MockEnv::new(
                "user_b".to_string(),
                amm.rewards.to_owned().try_into().unwrap(),
            ),
        )
        .unwrap();
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::Vote {
                choice: sienna_rewards::gov::vote::VoteType::Yes,
                poll_id: 1,
            }),
            MockEnv::new(
                "user_b".to_string(),
                amm.rewards.to_owned().try_into().unwrap(),
            ),
        )
        .unwrap();

    let result: Response = amm
        .ensemble
        .query(
            amm.rewards.address,
            Query::Governance(GovernanceQuery::Polls {
                now: 1572402232,
                page: 1,
                take: 10,
            }),
        )
        .unwrap();

    println!("{:?}", result)
}
