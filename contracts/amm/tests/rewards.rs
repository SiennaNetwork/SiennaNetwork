use std::convert::TryInto;

use sienna_rewards::{
    ensemble::MockEnv,
    gov::handle::GovernanceHandle,
    gov::{
        poll_metadata::{PollMetadata, PollType},
        vote::VoteType,
    },
    handle::RewardsHandle,
    query::RewardsQuery,
    Handle, HumanAddr, Query, Response, Uint128,
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
fn test_deposit_rewards() {
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
}
#[test]
fn should_create_poll() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: PollType::SiennaRewards,
    };

    let env = MockEnv::new(sender.clone(), amm.rewards.to_owned().try_into().unwrap());

    //deposit some funds
    amm.deposit_lp_into_rewards(sender, Uint128(3600));

    //create poll
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll { meta }),
            env,
        )
        .unwrap();

    let poll_after = amm.get_poll(1, 1);
    assert_eq!(1, poll_after.id);
}

#[test]
fn should_cast_vote() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: PollType::SiennaRewards,
    };

    let env = MockEnv::new(sender.clone(), amm.rewards.to_owned().try_into().unwrap());

    //deposit some funds
    amm.deposit_lp_into_rewards(sender, Uint128(3600));

    //create poll
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll { meta }),
            env.clone(),
        )
        .unwrap();

    let vote = GovernanceHandle::Vote {
        poll_id: 1,
        choice: VoteType::Yes,
    };

    amm.ensemble
        .execute(&Handle::Governance(vote), env)
        .unwrap();
}

// amm.ensemble
//     .execute(
//         &Handle::Governance(GovernanceHandle::CreatePoll {
//             meta: PollMetadata {
//                 description: "tesekfekjfefeft".into(),
//                 title: "testtiiing".into(),
//                 poll_type: sienna_rewards::gov::poll_metadata::PollType::Other,
//             },
//         }),
//         MockEnv::new(
//             "user_b".to_string(),
//             amm.rewards.to_owned().try_into().unwrap(),
//         ),
//     )
//     .unwrap();
// amm.ensemble
//     .execute(
//         &Handle::Governance(GovernanceHandle::Vote {
//             choice: sienna_rewards::gov::vote::VoteType::Yes,
//             poll_id: 1,
//         }),
//         MockEnv::new(
//             "user_b".to_string(),
//             amm.rewards.to_owned().try_into().unwrap(),
//         ),
//     )
//     .unwrap();

// let result: Response = amm
//     .ensemble
//     .query(
//         amm.rewards.address,
//         Query::Governance(GovernanceQuery::Polls {
//             now: 1572402232,
//             page: 1,
//             take: 10,
//         }),
//     )
//     .unwrap();

// println!("{:?}", result)
