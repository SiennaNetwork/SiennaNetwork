use amm_shared::Sender;
use fadroma::{Api, HumanAddr, Querier, QueryDispatch, StdResult, Storage};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    governance::Governance,
    response::{GovernanceResponse, IGovernanceResponse},
};
use crate::time_utils::Moment;
use crate::{
    auth::{Auth, AuthMethod},
    permit::Permit,
};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceQuery {
    Polls {
        take: u64,
        page: u64,
        asc: bool,
        now: Moment,
    },
    Poll {
        id: u64,
        now: Moment,
    },
    VoteStatus {
        poll_id: u64,
        address: HumanAddr,
        key: String,
    },
    WithPermit {
        query: QueryWithPermit,
        permit: Permit<GovernancePermissions>,
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
            GovernanceQuery::Polls {
                take,
                page,
                now,
                asc,
            } => GovernanceResponse::polls(core, take, page, asc, now),
            GovernanceQuery::Poll { id, now } => GovernanceResponse::poll(core, id, now),
            GovernanceQuery::Config {} => GovernanceResponse::config(core),
            GovernanceQuery::VoteStatus {
                poll_id,
                key,
                address,
            } => {
                let sender = Auth::authenticate(
                    core,
                    AuthMethod::ViewingKey {
                        address,
                        key: key.into(),
                    },
                    GovernancePermissions::VoteStatus,
                )?;
                let sender = Sender::from_human(&sender, core.api())?;
                GovernanceResponse::vote_status(core, poll_id, &sender)
            }
            GovernanceQuery::WithPermit { permit, query } => {
                let sender = Auth::authenticate(
                    core,
                    AuthMethod::Permit(permit),
                    GovernancePermissions::VoteStatus,
                )?;
                let sender = Sender::from_human(&sender, core.api())?;
                match query {
                    QueryWithPermit::VoteStatus { poll_id } => {
                        GovernanceResponse::vote_status(core, poll_id, &sender)
                    }
                }
            }
        }
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum QueryWithPermit {
    VoteStatus { poll_id: u64 },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum GovernancePermissions {
    VoteStatus,
}
