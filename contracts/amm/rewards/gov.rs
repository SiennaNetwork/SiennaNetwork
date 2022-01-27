use fadroma::*;
use crate::{auth::Auth, errors};
use serde::{Serialize, Deserialize};
use schemars::JsonSchema;

// feature trait ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
pub trait Governance<S: Storage, A: Api, Q: Querier>:
    Composable<S, A, Q> // to compose with other modules
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

// init config api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
/// Governance configuration
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct GovernanceConfig {
    // ...
}
impl GovernanceConfig {
    pub const STORAGE_KEY: &'static[u8] = b"/gov/something";
    // ...
}
pub trait IGovernanceConfig <S, A, Q, C> where
    S: Storage, A: Api, Q: Querier, C: Governance<S, A, Q>
{
    /// Commit initial contract configuration to storage.
    fn initialize    (&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>>;
    // ...
}
impl<S, A, Q, C> IGovernanceConfig<S, A, Q, C> for GovernanceConfig where
    S: Storage, A: Api, Q: Querier, C: Governance<S, A, Q>
{
    fn initialize (&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        Ok(vec![])
    }
    // ...
}

// handle tx api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum GovernanceHandle {
    NewPoll {},
    Vote {}
    // ...
}
impl<S, A, Q, C> HandleDispatch<S, A, Q, C> for GovernanceHandle where
    S: Storage, A: Api, Q: Querier, C: Governance<S, A, Q>
{
    fn dispatch_handle (self, core: &mut C, env: Env) -> StdResult<HandleResponse> {
        Ok(match self {
            GovernanceHandle::NewPoll {} => HandleResponse::default(),
            GovernanceHandle::Vote {}    => HandleResponse::default()
            // ...
        })
    }
}

// query api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum GovernanceQuery {
    PollList   {},
    PollDetail {}
}
impl<S, A, Q, C> QueryDispatch<S, A, Q, C, GovernanceResponse> for GovernanceQuery where
    S: Storage, A: Api, Q: Querier, C: Governance<S, A, Q>
{
    fn dispatch_query (self, core: &C) -> StdResult<GovernanceResponse> {
        match self {
            GovernanceQuery::PollList {} =>
                GovernanceResponse::poll_list(core),
            GovernanceQuery::PollDetail {} =>
                GovernanceResponse::poll_detail(core)
        }
    }
}

// response api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone,Debug,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum GovernanceResponse {
    PollList  {},
    PollDetail(Poll)
}
pub trait IGovernanceResponse<S, A, Q, C>: Sized where
    S: Storage, A: Api, Q: Querier, C: Governance<S, A, Q>
{
    fn poll_list   (core: &C) -> StdResult<Self>;
    fn poll_detail (core: &C) -> StdResult<Self>;
}
impl<S, A, Q, C> IGovernanceResponse<S, A, Q, C> for GovernanceResponse where
    S: Storage, A: Api, Q: Querier, C: Governance<S, A, Q>
{
    fn poll_list   (core: &C) -> StdResult<Self> {
        Ok(GovernanceResponse::PollList{})
    }
    fn poll_detail (core: &C) -> StdResult<GovernanceResponse> {
        Ok(GovernanceResponse::PollDetail(Poll{}))
    }
}

// custom logic ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone,Debug,Default,PartialEq,Serialize,Deserialize,JsonSchema)]
#[serde(rename_all="snake_case")]
pub struct Poll { /* go wild here */ }
