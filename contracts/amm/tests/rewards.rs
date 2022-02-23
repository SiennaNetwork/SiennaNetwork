use std::convert::TryInto;

use sienna_rewards::{
    ensemble::MockEnv,
    gov::handle::GovernanceHandle,
    gov::{
        poll_metadata::{PollMetadata, PollType},
        query::GovernanceQuery,
        response::GovernanceResponse::VoteStatus,
        vote::VoteType,
    },
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
    amm.deposit_lp_into_rewards("user_b".into(), Uint128(100));
    amm.set_rewards_viewing_key("user_b".into(), "whatever".into());

    let balance = amm.get_rewards_staked("user_b".into(), "whatever".into());
    assert_eq!(100, balance.u128());
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
    amm.deposit_lp_into_rewards(sender.clone(), Uint128(3600));
    amm.set_rewards_viewing_key(sender.clone(), "whatever".into());

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

    let vote: Response = amm
        .ensemble
        .query(
            amm.rewards.address.clone(),
            &Query::Governance(GovernanceQuery::VoteStatus {
                address: sender,
                key: "whatever".to_string(),
                poll_id: 1,
            }),
        )
        .unwrap();
    match vote {
        Response::Governance(VoteStatus { variant, power }) => {
            assert_eq!(variant, VoteType::Yes);
            assert_eq!(power, Uint128(3600))
        }
        _ => panic!("invalid type for vote status returned."),
    }
}

#[test]
fn should_change_choice() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that is longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: PollType::SiennaRewards,
    };

    let env = MockEnv::new(sender.clone(), amm.rewards.to_owned().try_into().unwrap());

    //deposit some funds
    amm.deposit_lp_into_rewards(sender.clone(), Uint128(3600));

    amm.set_rewards_viewing_key(sender.clone(), "whatever".into());

    //create poll
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll { meta }),
            env.clone(),
        )
        .unwrap();

    let vote = GovernanceHandle::Vote {
        poll_id: 1,
        choice: VoteType::No,
    };

    //vote
    amm.ensemble
        .execute(&Handle::Governance(vote.clone()), env.clone())
        .unwrap();

    //query the status
    let vote: Response = amm
        .ensemble
        .query(
            amm.rewards.address.clone(),
            &Query::Governance(GovernanceQuery::VoteStatus {
                address: sender.clone(),
                key: "whatever".to_string(),
                poll_id: 1,
            }),
        )
        .unwrap();
    match vote {
        Response::Governance(VoteStatus { variant, power }) => {
            assert_eq!(variant, VoteType::No);
            assert_eq!(power, Uint128(3600))
        }
        _ => panic!("invalid type for vote status returned."),
    }

    let change_vote = GovernanceHandle::ChangeVoteChoice {
        choice: VoteType::Yes,
        poll_id: 1,
    };
    amm.ensemble
        .execute(&Handle::Governance(change_vote), env.clone())
        .unwrap();

    let vote: Response = amm
        .ensemble
        .query(
            amm.rewards.address.clone(),
            &Query::Governance(GovernanceQuery::VoteStatus {
                address: sender,
                key: "whatever".to_string(),
                poll_id: 1,
            }),
        )
        .unwrap();
    match vote {
        Response::Governance(VoteStatus { variant, power }) => {
            assert_eq!(variant, VoteType::Yes);
            assert_eq!(power, Uint128(3600))
        }
        _ => panic!("invalid type for vote status returned."),
    }
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
