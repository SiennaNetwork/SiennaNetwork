use amm_shared::Sender;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use fadroma::*;

use crate::account::{Account, CloseSeal, IAccount};
use crate::auth::Auth;
use crate::errors::{self, poll_expired};

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
    /// Handles the creation of polls, metadata is the user input. The rest is determined automatically
    CreatePoll {
        meta: PollMetadata,
    },
    /// Handles adding votes to a poll
    Vote {
        choice: VoteType,
        poll_id: u64,
    },
    /// Handles removing votes from a poll
    Unvote {
        poll_id: u64,
    },

    /// Handles changing the choice for a poll
    ChangeVoteChoice {
        choice: VoteType,
        poll_id: u64,
    },

    /// Updates the configuration, the fields are optional so configuration can be partially updated by only setting
    /// the desired fields to update
    UpdateConfig {
        config: GovernanceConfig,
    },
    Close {
        reason: String,
    },
}
impl<S, A, Q, C> HandleDispatch<S, A, Q, C> for GovernanceHandle
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    /// Handles all of the governance transactions. For a detailed flow, check the governance documentation
    fn dispatch_handle(self, core: &mut C, env: Env) -> StdResult<HandleResponse> {
        match self {
            GovernanceHandle::CreatePoll { meta } => {
                if let Some((time, reason)) = core.get::<CloseSeal>(GovernanceConfig::CLOSED)? {
                    return errors::governance_closed(time, reason);
                }

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
                validator::validate_text_length(
                    &meta.poll_type,
                    "Poll type",
                    GovernanceConfig::MIN_POLL_TYPE_LENGTH,
                    GovernanceConfig::MAX_POLL_TYPE_LENGTH,
                )?;
                let account = Account::from_env(core, &env)?;
                let threshold = GovernanceConfig::threshold(core)?;

                if account.staked < threshold {
                    return Err(StdError::generic_err("Insufficient funds to create a poll"));
                };

                let deadline = GovernanceConfig::deadline(core)?;
                let current_quorum = GovernanceConfig::quorum(core)?;
                let expiration = Expiration::AtTime(env.block.time + deadline);

                let sender = Sender::from_human(&env.message.sender, core.api())?;

                let poll = Poll::new(core, &sender, expiration, meta, current_quorum)?;
                User::create_poll(core, &sender, &poll, env.block.time)?;

                Ok(HandleResponse {
                    data: Some(to_binary(&poll)?),
                    log: vec![],
                    messages: vec![],
                })
            }
            GovernanceHandle::Vote { choice, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(env.block.time) {
                    return poll_expired();
                }

                let account = Account::from_env(core, &env)?;
                let power = account.staked;
                let sender = Sender::from_human(&env.message.sender, core.api())?;
                User::add_vote(core, poll_id, &sender, choice, power, env.block.time)?;
                Ok(HandleResponse::default())
            }
            GovernanceHandle::ChangeVoteChoice { choice, poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(env.block.time) {
                    return poll_expired();
                }
                let sender = Sender::from_human(&env.message.sender, core.api())?;
                User::change_choice(core, poll_id, &sender, choice, env.block.time)?;
                Ok(HandleResponse::default())
            }
            GovernanceHandle::Unvote { poll_id } => {
                let expiration = Poll::expiration(core, poll_id)?;
                if expiration.is_expired(env.block.time) {
                    return poll_expired();
                }
                let sender = Sender::from_human(&env.message.sender, core.api())?;
                User::remove_vote(core, poll_id, &sender, env.block.time)?;
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
                    GovernanceHandle::Close { reason } => {
                        let seal: CloseSeal = (env.block.time, reason);
                        core.set(GovernanceConfig::CLOSED, seal)?;
                        Ok(HandleResponse::default())
                    }
                    _ => unreachable!(),
                }
            }
        }
    }
}
