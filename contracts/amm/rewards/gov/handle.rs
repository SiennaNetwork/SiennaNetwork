
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

use crate::algo::{Account, IAccount};
use crate::auth::Auth;
use fadroma::*;

use super::{governance::{Governance}, vote::{VoteType, IVote, Vote}, poll_metadata::PollMetadata, config::{GovernanceConfig, IGovernanceConfig}, expiration::Expiration, poll::{Poll, IPoll, PollStatus}};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceHandle {
    CreatePoll { meta: PollMetadata },
    Vote { variant: VoteType, poll_id: u64 },
    Unvote { poll_id: u64 },
    ChangeVote { variant: VoteType, poll_id: u64 },
    UpdateConfig { config: GovernanceConfig },
    Reveal { poll_id: u64} 
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
                let id = Poll::create_id(core)?;
                let deadline = GovernanceConfig::deadline(core)?;
                let expiration = Expiration::AtTime(env.block.time + deadline);

                let poll = Poll {
                    creator: core.canonize(env.message.sender)?,
                    expiration,
                    id,
                    metadata: meta,
                    status: PollStatus::Active,
                };

                poll.store(core)?;

                Ok(HandleResponse::default())
            }
            GovernanceHandle::Vote { variant, poll_id } => {
                if let Ok(_) = Vote::get(core, env.message.sender.clone(), poll_id) {
                    return Err(StdError::generic_err(
                        "Already voted. Did you mean to update vote?",
                    ));
                }
                let account = Account::from_env(core, &env)?;

                let vote_power = account.staked;

                let vote = Vote {
                    variant,
                    vote_power,
                    voter: core.canonize(env.message.sender.clone())?,
                };
                
                vote.store(core, env.message.sender, poll_id)?;

                Ok(HandleResponse::default())
            }
            GovernanceHandle::ChangeVote { variant, poll_id } => Ok(HandleResponse::default()),
            GovernanceHandle::Unvote { poll_id } => Ok(HandleResponse::default()),
            GovernanceHandle::Reveal { poll_id} => {
                // not implemented
                Ok(HandleResponse::default())
            },
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