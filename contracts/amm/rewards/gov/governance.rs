use fadroma::*;

use crate::auth::Auth;

use super::{
    config::{GovernanceConfig, IGovernanceConfig},
    handle::GovernanceHandle,
    query::GovernanceQuery,
    response::GovernanceResponse,
};

pub trait Governance<S: Storage, A: Api, Q: Querier>:
    Composable<S, A, Q>  // to compose with other modules
    + Auth<S, A, Q>     // to authenticate txs/queries
    + Sized             // to pass mutable self-reference to Total and Account
{
    /// Configure the governance module
    fn init (&mut self, env: &Env, mut config: GovernanceConfig) -> StdResult<Vec<CosmosMsg>> {
        // TODO make a require-feature macro instead of duplicating the same branch containing the magic string
        if cfg!(feature="gov") {
            config.initialize(self, env)
        } else {
            Err(StdError::generic_err("Governance disabled"))
        }
    }
    /// Handle transactions
    fn handle (&mut self, env: Env, msg: GovernanceHandle) -> StdResult<HandleResponse> {
        if cfg!(feature="gov") {
            msg.dispatch_handle(self, env)
        } else {
            Err(StdError::generic_err("Governance disabled"))
        }
    }
    /// Handle queries
    fn query (&self, msg: GovernanceQuery) -> StdResult<GovernanceResponse> {
        if cfg!(feature="gov") {
            msg.dispatch_query(self)
        } else {
            Err(StdError::generic_err("Governance disabled"))
        }
    }
}
