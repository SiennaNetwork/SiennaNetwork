use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::algo::{self, Account, IAccount};
use crate::auth::Auth;
use fadroma::*;

use super::poll_result::{IPollResult, PollResult};
use super::validator;
use super::{
    config::{GovernanceConfig, IGovernanceConfig},
    expiration::Expiration,
    governance::Governance,
    poll::{IPoll, Poll, PollStatus},
    poll_metadata::PollMetadata,
    vote::{IVote, Vote, VoteType},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceHandle {
    CreatePoll { meta: PollMetadata },
    Vote { variant: VoteType, poll_id: u64 },
    Unvote { poll_id: u64 },
    ChangeVote { variant: VoteType, poll_id: u64 },
    UpdateConfig { config: GovernanceConfig },
    Reveal { poll_id: u64 },
    AddCommitteeMember { member: HumanAddr },
    RemoveCommitteeMember { member: HumanAddr },
}
impl<S, A, Q, C> HandleDispatch<S, A, Q, C> for GovernanceHandle
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn dispatch_handle(self, core: &mut C, env: Env) -> StdResult<HandleResponse> {
        match self {
            GovernanceHandle::CreatePoll { meta } => {
                validator::validate_text_length(
                    &meta.title,
                    "Title",
                    GovernanceConfig::MIN_TITLE_LENGTH,
                    GovernanceConfig::MAX_TITLE_LENGTH,
                )?;
                validator::validate_text_length(
                    &meta.description,
                    "Description",
                    GovernanceConfig::MIN_DESC_LENGTH,
                    GovernanceConfig::MAX_DESC_LENGTH,
                )?;
                let account = algo::Account::from_env(core, &env)?;
                let threshold = GovernanceConfig::threshold(core)?;

                if account.staked < threshold.into() {
                    return Err(StdError::generic_err("Insufficient funds to create a poll"));
                };

                let id = Poll::create_id(core)?;
                let deadline = GovernanceConfig::deadline(core)?;
                let current_quorum = GovernanceConfig::quorum(core)?;
                let expiration = Expiration::AtTime(env.block.time + deadline);

                let poll = Poll {
                    creator: core.canonize(env.message.sender)?,
                    status: PollStatus::Active,
                    reveal_approvals: vec![],
                    expiration,
                    id,
                    metadata: meta,
                    current_quorum,
                };

                poll.store(core)?;

                Ok(HandleResponse {
                    data: Some(to_binary(&poll)?),
                    log: vec![
                        log("ACTION", "CREATE_POLL"),
                        log("POLL_ID", format!("{}", id)),
                        log("POLL_CREATOR", format!("{}", &poll.creator)),
                    ],
                    messages: vec![],
                })
            }
            GovernanceHandle::Vote { variant, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(&env.block) {
                    return Err(StdError::generic_err(
                        "Poll has expired. Voting is not possible anymore.",
                    ));
                }
                if let Ok(_) = Vote::get(core, env.message.sender.clone(), poll_id) {
                    return Err(StdError::generic_err(
                        "Already voted. Did you mean to update vote?",
                    ));
                }
                let account = Account::from_env(core, &env)?;

                let vote_power = account.staked;

                //save vote
                let vote = Vote {
                    variant,
                    vote_power,
                    voter: core.canonize(env.message.sender.clone())?,
                };

                vote.store(core, env.message.sender, poll_id)?;

                //(re)calculate result
                let mut result =
                    PollResult::get(core, poll_id).unwrap_or(PollResult::new(core, poll_id));

                match vote.variant {
                    VoteType::Yes => result.yes_votes += 1,
                    VoteType::No => result.no_votes += 1,
                }

                result.amassed_voting_power += vote_power;

                result.store(core)?;

                Ok(HandleResponse::default())
            }
            GovernanceHandle::ChangeVote { variant, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(&env.block) {
                    return Err(StdError::generic_err(
                        "Poll has expired. Voting is not possible anymore.",
                    ));
                }
                let mut vote = Vote::get(core, env.message.sender.clone(), poll_id)?;
                let mut result = PollResult::get(core, poll_id)?;

                //avoid transaction cost if the casted vote change is the same.
                if vote.variant == variant {
                    return Err(StdError::generic_err(
                        "Your vote is not changed. You tried to vote with the same variant. ",
                    ));
                }

                //update result based on new variant
                match variant {
                    VoteType::Yes => {
                        result.yes_votes += 1;
                        result.no_votes -= 1;
                    }
                    VoteType::No => {
                        result.yes_votes -= 1;
                        result.no_votes += 1;
                    }
                }

                //update original vote
                vote.variant = variant;

                //save everything
                vote.store(core, env.message.sender, poll_id)?;
                result.store(core)?;

                Ok(HandleResponse::default())
            }
            GovernanceHandle::Unvote { poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(&env.block) {
                    return Err(StdError::generic_err(
                        "Poll has expired. Unvoting is not possible anymore.",
                    ));
                }
                let vote = Vote::get(core, env.message.sender.clone(), poll_id)?;
                let mut result = PollResult::get(core, poll_id)?;

                //remove user's vote power from result
                let new_total = result
                    .amassed_voting_power
                    .u128()
                    .checked_sub(vote.vote_power.u128());

                // this should never panic, unless the vote is not properly updated.
                result.amassed_voting_power = Uint128::from(new_total.unwrap());

                //remove user's variant
                match vote.variant {
                    VoteType::Yes => result.yes_votes -= 1,
                    VoteType::No => result.no_votes -= 1,
                }
                result.store(core)?;

                Vote::remove(core, env.message.sender, poll_id)?;

                Ok(HandleResponse::default())
            }
            GovernanceHandle::Reveal { poll_id } => {
                // at least 2 out of 3 members must approve a reveal
                let sender = env.message.sender;
                let members = GovernanceConfig::committee(core)?;
                let approvals = Poll::reveal_approvals(core, poll_id)?;
                let is_member = members.contains(&sender);
                let already_approved = approvals.contains(&sender);
                if is_member && !already_approved {
                    Poll::approve_reveal(core, poll_id, &sender)?
                }
                Ok(HandleResponse::default())
            }
            _ => {
                Auth::assert_admin(core, &env)?;
                match self {
                    GovernanceHandle::UpdateConfig { config } => Ok(HandleResponse {
                        messages: config.store(core)?,
                        log: vec![],
                        data: None,
                    }),
                    GovernanceHandle::AddCommitteeMember { member } => {
                        GovernanceConfig::add_committee_member(core, member)?;
                        Ok(HandleResponse::default())
                    }
                    GovernanceHandle::RemoveCommitteeMember { member } => {
                        GovernanceConfig::remove_committee_member(core, member)?;
                        Ok(HandleResponse::default())
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
