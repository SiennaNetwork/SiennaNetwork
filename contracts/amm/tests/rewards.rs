use std::convert::TryInto;

use sienna_rewards::{
    ensemble::MockEnv,
    gov::handle::GovernanceHandle,
    gov::{
        poll_metadata::PollMetadata, query::GovernanceQuery,
        response::GovernanceResponse::VoteStatus, vote::VoteType,
    },
    handle::RewardsHandle,
    query::RewardsQuery,
    Handle, HumanAddr, Query, Response, Uint128,
};

use crate::setup::Amm;

#[test]
fn should_init() {
    let amm = Amm::new();

    let _result: Response = amm
        .ensemble
        .query(amm.rewards.address, Query::Rewards(RewardsQuery::Config))
        .unwrap();
}

#[test]
fn should_deposit_rewards() {
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
        poll_type: "Random type".to_string(),
    };

    let env = MockEnv::new(sender.clone(), amm.rewards.to_owned().try_into().unwrap());

    //deposit some funds
    amm.deposit_lp_into_rewards(sender, Uint128(3600));

    //create multiple poll
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll { meta: meta.clone() }),
            env.clone(),
        )
        .unwrap();

    let poll_after = amm.get_poll(1, 1);
    assert_eq!(1, poll_after.instance.id);
}
#[test]
fn should_paginate_polls() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: "Random type".to_string(),
    };

    let env = MockEnv::new(sender.clone(), amm.rewards.to_owned().try_into().unwrap());

    //deposit some funds
    amm.deposit_lp_into_rewards(sender, Uint128(3600));

    //create multiple poll
    for _ in 1..=51 {
        amm.ensemble
            .execute(
                &Handle::Governance(GovernanceHandle::CreatePoll { meta: meta.clone() }),
                env.clone(),
            )
            .unwrap();
    }

    let polls = amm.get_polls(6, 10, true, 1);
    assert_eq!(polls[0].id, 51);
    let poll_after = amm.get_poll(1, 1);
    assert_eq!(1, poll_after.instance.id);
}

#[test]
fn should_cast_vote() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: "Random type".to_string(),
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
        Response::Governance(VoteStatus { choice, power }) => {
            assert_eq!(choice, VoteType::Yes);
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
        poll_type: "Random type".to_string(),
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
        Response::Governance(VoteStatus { choice, power }) => {
            assert_eq!(choice, VoteType::No);
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
        Response::Governance(VoteStatus { choice, power }) => {
            assert_eq!(choice, VoteType::Yes);
            assert_eq!(power, Uint128(3600))
        }
        _ => panic!("invalid type for vote status returned."),
    }
}

#[test]
fn should_remove_vote() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that is longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: "Random type".to_string(),
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

    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::Unvote { poll_id: 1 }),
            env,
        )
        .unwrap();

    let vote: Result<Response, sienna_rewards::StdError> = amm.ensemble.query(
        amm.rewards.address.clone(),
        &Query::Governance(GovernanceQuery::VoteStatus {
            address: sender,
            key: "whatever".to_string(),
            poll_id: 1,
        }),
    );
    vote.unwrap_err();
}

#[test]
fn should_update_after_deposit() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that is longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: "Random type".to_string(),
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

    amm.deposit_lp_into_rewards(sender.clone(), Uint128(100));

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
        Response::Governance(VoteStatus { choice, power }) => {
            assert_eq!(choice, VoteType::No);
            assert_eq!(power, Uint128(3700))
        }
        _ => panic!("invalid type for vote status returned."),
    }
}

#[test]
fn should_not_withdraw() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that is longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: "Random type".to_string(),
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

    amm.ensemble
        .execute(
            &Handle::Rewards(RewardsHandle::Withdraw {
                amount: Uint128(200),
            }),
            env,
        )
        .unwrap_err();
}

#[test]
fn should_withdraw() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that is longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: "Random type".to_string(),
    };

    let env = MockEnv::new(sender.clone(), amm.rewards.to_owned().try_into().unwrap());

    //deposit some funds
    amm.deposit_lp_into_rewards(sender.clone(), Uint128(3800));

    amm.set_rewards_viewing_key(sender.clone(), "whatever".into());

    //create poll
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll { meta }),
            env.clone(),
        )
        .unwrap();

    amm.ensemble
        .execute(
            &Handle::Rewards(RewardsHandle::Withdraw {
                amount: Uint128(200),
            }),
            env,
        )
        .unwrap();
}
#[test]
fn should_be_closed() {
    let mut amm = Amm::new();
    let sender = HumanAddr::from("user_b");
    let meta = PollMetadata {
        description: "this is a description that is longer than 8 characters.".to_string(),
        title: "This is a title, no really".to_string(),
        poll_type: "Random type".to_string(),
    };

    let env = MockEnv::new(sender.clone(), amm.rewards.to_owned().try_into().unwrap());

    //deposit some funds
    amm.deposit_lp_into_rewards(sender.clone(), Uint128(3800));

    amm.set_rewards_viewing_key(sender.clone(), "whatever".into());

    //create poll
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll { meta: meta.clone() }),
            env.clone(),
        )
        .unwrap();

    let env = MockEnv::new("admin", amm.rewards.to_owned().try_into().unwrap());
    //close the poll
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::Close {
                reason: "Testing closing".into(),
            }),
            env.clone(),
        )
        .unwrap();

    let env = env.time(99999999);

    //deposit some funds
    //this should now fail
    amm.ensemble
        .execute(
            &Handle::Governance(GovernanceHandle::CreatePoll { meta }),
            env,
        )
        .unwrap_err();
}
