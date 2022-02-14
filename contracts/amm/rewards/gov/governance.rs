use fadroma::*;

use crate::{ auth::Auth};

use super::{config::{GovernanceConfig, IGovernanceConfig}, handle::GovernanceHandle, query::GovernanceQuery, response::GovernanceResponse};


pub trait Governance<S: Storage, A: Api, Q: Querier>:
    Composable<S, A, Q>  // to compose with other modules
    + Auth<S, A, Q>     // to authenticate txs/queries
    + Sized             // to pass mutable self-reference to Total and Account
{
    /// Configure the rewards module
    fn init (&mut self, env: &Env, mut config: GovernanceConfig) -> StdResult<Vec<CosmosMsg>> {
        config.initialize(self, env)
    }
    /// Handle transactions
    fn handle (&mut self, env: Env, msg: GovernanceHandle) -> StdResult<HandleResponse> {
        msg.dispatch_handle(self, env)
    }
    /// Handle queries
    fn query (&self, msg: GovernanceQuery) -> StdResult<GovernanceResponse> {
        msg.dispatch_query(self)
    }
}

