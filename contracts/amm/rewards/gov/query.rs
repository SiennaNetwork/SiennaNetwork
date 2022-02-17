use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    governance::Governance,
    response::{GovernanceResponse, IGovernanceResponse},
    user::{IUser, User},
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceQuery {
    Polls {
        take: u64,
        page: u64,
    },
    Poll {
        id: u64,
    },
    VoteStatus {
        poll_id: u64,
        address: HumanAddr,
        key: String,
    },
    Config {},
}
impl<S, A, Q, C> QueryDispatch<S, A, Q, C, GovernanceResponse> for GovernanceQuery
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn dispatch_query(self, core: &C) -> StdResult<GovernanceResponse> {
        match self {
            GovernanceQuery::Polls { take, page } => {
                GovernanceResponse::polls(core, take, page, false)
            }
            GovernanceQuery::Poll { id } => GovernanceResponse::poll(core, id),
            GovernanceQuery::Config {} => GovernanceResponse::config(core),
            GovernanceQuery::VoteStatus {
                poll_id,
                key,
                address,
            } => {
                User::check_viewing_key(core, address.clone(), &key.into())?;
                GovernanceResponse::vote_status(core, poll_id, address)
            }
        }
    }
}
