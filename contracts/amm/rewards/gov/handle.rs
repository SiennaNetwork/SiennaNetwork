use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use fadroma::*;

use crate::account::{Account, IAccount};
use crate::auth::Auth;
use crate::errors::poll_expired;
use crate::time_utils::Moment;

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
    Vote { choice: VoteType, poll_id: u64 },
    Unvote { poll_id: u64 },
    ChangeVoteChoice { choice: VoteType, poll_id: u64 },
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
        let now: Moment = env.block.time;
        let sender = env.message.sender.clone();
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
                let account = Account::from_env(core, &env)?;
                let threshold = GovernanceConfig::threshold(core)?;

                if account.staked < threshold {
                    return Err(StdError::generic_err("Insufficient funds to create a poll"));
                };

                let deadline = GovernanceConfig::deadline(core)?;
                let current_quorum = GovernanceConfig::quorum(core)?;
                let current_time = env.block.time;
                let expiration = Expiration::AtTime(current_time + deadline);
                let creator = core.canonize(sender.clone())?;

                let poll = Poll::new(core, creator, expiration, meta, current_quorum)?;
                User::create_poll(core, sender, &poll, current_time)?;

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
            GovernanceHandle::Vote { choice, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(now) {
                    return poll_expired();
                }

                let account = Account::from_env(core, &env)?;
                let power = account.staked; // Uint128(200);

                User::add_vote(core, poll_id, sender, choice, power, now)?;
                Ok(HandleResponse::default())
            }
            GovernanceHandle::ChangeVoteChoice { choice, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(now) {
                    return poll_expired();
                }
                User::change_choice(core, poll_id, sender, choice, now)?;
                Ok(HandleResponse::default())
            }
            GovernanceHandle::Unvote { poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(now) {
                    return poll_expired();
                }
                User::remove_vote(core, poll_id, sender, now)?;
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
