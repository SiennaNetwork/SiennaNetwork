use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use fadroma::*;

use crate::auth::Auth;
use crate::errors::poll_expired;

use super::poll::UpdateResultDto;
use super::user::{IUser, User};
use super::validator;
use super::{
    config::{GovernanceConfig, IGovernanceConfig},
    expiration::Expiration,
    governance::Governance,
    poll::{IPoll, Poll},
    poll_metadata::PollMetadata,
    vote::VoteType,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceHandle {
    CreatePoll { meta: PollMetadata },
    Vote { variant: VoteType, poll_id: u64 },
    Unvote { poll_id: u64 },
    ChangeVote { variant: VoteType, poll_id: u64 },
    UpdateConfig { config: GovernanceConfig },
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
                // let account = Account::from_env(core, &env)?;
                let staked: Uint128 = Uint128(3600);
                let threshold = GovernanceConfig::threshold(core)?;

                if staked < threshold.into() {
                    return Err(StdError::generic_err("Insufficient funds to create a poll"));
                };

                let deadline = GovernanceConfig::deadline(core)?;
                let current_quorum = GovernanceConfig::quorum(core)?;
                let expiration = Expiration::AtTime(env.block.time + deadline);
                let creator = core.canonize(env.message.sender)?;

                let poll = Poll::new(core, creator, expiration, meta, current_quorum)?;

                poll.store(core)?;

                Ok(HandleResponse {
                    data: Some(to_binary(&poll)?),
                    log: vec![
                        log("ACTION", "CREATE_POLL"),
                        log("POLL_ID", format!("{}", &poll.id)),
                        log("POLL_CREATOR", format!("{}", &poll.creator)),
                    ],
                    messages: vec![],
                })
            }
            GovernanceHandle::Vote { variant, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(env.block.time) {
                    return poll_expired();
                }

                // let account = Account::from_env(core, &env)?;
                let power = Uint128(200);

                Poll::update_result(
                    core,
                    poll_id,
                    env.message.sender.clone(),
                    UpdateResultDto::AddVote { power, variant },
                )?;

                User::append_active_poll(core, env.message.sender, poll_id, env.block.time)?;

                Ok(HandleResponse::default())
            }
            GovernanceHandle::ChangeVote { variant, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(env.block.time) {
                    return poll_expired();
                }
                Poll::update_result(
                    core,
                    poll_id,
                    env.message.sender,
                    UpdateResultDto::ChangeVoteVariant { variant },
                )?;

                Ok(HandleResponse::default())
            }
            GovernanceHandle::Unvote { poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(env.block.time) {
                    return poll_expired();
                }

                Poll::update_result(
                    core,
                    poll_id,
                    env.message.sender.clone(),
                    UpdateResultDto::RemoveVote {},
                )?;

                User::remove_active_poll(core, env.message.sender, poll_id, env.block.time)?;

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
                    _ => unreachable!(),
                }
            }
        }
    }
}
