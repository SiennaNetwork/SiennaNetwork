use crate::auth::Auth;
use core::fmt;
use cosmwasm_std::BlockInfo;
use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct GovernanceConfig {
    pub threshold: Option<u8>,
    pub quorum: Option<u8>,
    pub deadline: Option<u64>,
}
impl GovernanceConfig {
    //metadata configuration
    pub const MIN_TITLE_LENGTH: usize = 8;
    pub const MAX_TITLE_LENGTH: usize = 64;

    pub const MIN_DESC_LENGTH: usize = 8;
    pub const MAX_DESC_LENGTH: usize = 1024;

    //storage keys
    pub const THRESHOLD: &'static [u8] = b"/gov/threshold";
    pub const QUORUM: &'static [u8] = b"/gov/quorum";
    pub const DEADLINE: &'static [u8] = b"/gov/deadline";
    pub const VOTES: &'static [u8] = b"/gov/votes";
}
pub trait IGovernanceConfig<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    /// Commit initial contract configuration to storage.
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>>;
    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>>;
    fn threshold(core: &C) -> StdResult<u8>;
    fn quorum(core: &C) -> StdResult<u8>;
    fn deadline(core: &C) -> StdResult<u64>;
}
impl<S, A, Q, C> IGovernanceConfig<S, A, Q, C> for GovernanceConfig
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn initialize(&mut self, core: &mut C, env: &Env) -> StdResult<Vec<CosmosMsg>> {
        Ok(vec![])
    }

    fn store(&self, core: &mut C) -> StdResult<Vec<CosmosMsg>> {
        let GovernanceConfig {
            deadline,
            threshold,
            quorum,
        } = self;
        if let Some(deadline) = deadline {
            core.set(Self::DEADLINE, deadline)?;
        }
        if let Some(threshold) = threshold {
            core.set(Self::THRESHOLD, threshold)?;
        }
        if let Some(quorum) = quorum {
            core.set(Self::QUORUM, quorum)?;
        }
        Ok(vec![])
    }

    fn threshold(core: &C) -> StdResult<u8> {
        core.get::<u8>(Self::THRESHOLD)?
            .ok_or(StdError::generic_err("threshold not set"))
    }

    fn quorum(core: &C) -> StdResult<u8> {
        core.get::<u8>(Self::QUORUM)?
            .ok_or(StdError::generic_err("quorum not set"))
    }

    fn deadline(core: &C) -> StdResult<u64> {
        core.get::<u64>(Self::THRESHOLD)?
            .ok_or(StdError::generic_err("deadline not set"))
    }
}

// handle tx api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
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
                //here
                Ok(HandleResponse::default())
            }
            GovernanceHandle::ChangeVote { variant, poll_id } => Ok(HandleResponse::default()),
            GovernanceHandle::Unvote { poll_id } => Ok(HandleResponse::default()),
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

// query api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceQuery {
    Polls {},
    Poll {},
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
            GovernanceQuery::Polls {} => GovernanceResponse::polls(core),
            GovernanceQuery::Poll {} => GovernanceResponse::poll(core),
        }
    }
}

// response api ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceResponse {
    Polls {},
    Poll(Poll),
    Config(GovernanceConfig),
}
pub trait IGovernanceResponse<S, A, Q, C>: Sized
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(core: &C) -> StdResult<Self>;
    fn poll(core: &C) -> StdResult<Self>;
}
impl<S, A, Q, C> IGovernanceResponse<S, A, Q, C> for GovernanceResponse
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn polls(_core: &C) -> StdResult<Self> {
        Ok(GovernanceResponse::Polls {})
    }
    fn poll(core: &C) -> StdResult<GovernanceResponse> {
        Ok(GovernanceResponse::Poll(Poll {
            creator: core.canonize(HumanAddr::from("test"))?,
            id: 1,
            metadata: PollMetadata {
                description: "test".to_string(),
                poll_type: PollType::Other,
                title: "test".to_string(),
            },
            expiration: Expiration::AtTime(42),
            status: PollStatus::Active,
        }))
    }
}

// custom logic ~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Poll {
    pub id: u64,
    pub creator: CanonicalAddr,
    pub metadata: PollMetadata,
    pub expiration: Expiration,
    pub status: PollStatus,
}

impl Poll {
    pub const TOTAL: &'static [u8] = b"/gov/polls/total";

    pub const CREATOR: &'static [u8] = b"/gov/poll/creator";
    pub const EXPIRATION: &'static [u8] = b"/gov/poll/expiration";
    pub const STATUS: &'static [u8] = b"/gov/poll/status";
}

pub trait IPoll<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn create_id(core: &mut C) -> StdResult<u64>;

    fn store(&self, core: &mut C) -> StdResult<()>;
    fn get(core: &C, poll_id: u64) -> StdResult<Self>;

    fn creator(core: &C, poll_id: u64) -> StdResult<CanonicalAddr>;

    fn metadata(core: &C, poll_id: u64) -> StdResult<PollMetadata>;
    fn expiration(core: &C, poll_id: u64) -> StdResult<Expiration>;
    fn status(core: &C, poll_id: u64) -> StdResult<PollStatus>;

    fn commit_status(&self, core: &mut C) -> StdResult<()>;
}

impl<S, A, Q, C> IPoll<S, A, Q, C> for Poll
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn create_id(core: &mut C) -> StdResult<u64> {
        let total = core
            .get::<u64>(Self::TOTAL)?
            .ok_or(StdError::generic_err("can't find total id count"))?;
        let total = total
            .checked_add(1)
            .ok_or(StdError::generic_err("total integer overflow"))?;

        core.set(Self::TOTAL, total)?;

        Ok(total)
    }

    fn store(&self, core: &mut C) -> StdResult<()> {
        let Poll {
            creator,
            expiration,
            id,
            metadata,
            status,
        } = self;

        core.set_ns(Self::CREATOR, &self.id.to_be_bytes(), creator)?;
        core.set_ns(Self::EXPIRATION, &self.id.to_be_bytes(), expiration)?;
        core.set_ns(Self::STATUS, &self.id.to_be_bytes(), status)?;
        metadata.store(core, *id)?;

        Ok(())
    }

    fn get(core: &C, poll_id: u64) -> StdResult<Self> {
        let creator = Self::creator(core, poll_id)?;
        let expiration = Self::expiration(core, poll_id)?;
        let status = Self::status(core, poll_id)?;
        let metadata = Self::metadata(core, poll_id)?;

        Ok(Self {
            id: poll_id,
            creator,
            expiration,
            metadata,
            status,
        })
    }

    fn creator(core: &C, poll_id: u64) -> StdResult<CanonicalAddr> {
        Ok(core
            .get_ns::<CanonicalAddr>(Self::CREATOR, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll creator"))?)
    }
    fn metadata(core: &C, poll_id: u64) -> StdResult<PollMetadata> {
        PollMetadata::get(core, poll_id)
    }

    fn expiration(core: &C, poll_id: u64) -> StdResult<Expiration> {
        Ok(core
            .get_ns::<Expiration>(Self::EXPIRATION, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll expiration"))?)
    }

    fn status(core: &C, poll_id: u64) -> StdResult<PollStatus> {
        Ok(core
            .get_ns::<PollStatus>(Self::STATUS, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err("failed to parse poll expiration"))?)
    }

    fn commit_status(&self, core: &mut C) -> StdResult<()> {
        core.set_ns(Self::STATUS, &self.id.to_be_bytes(), self.status.clone())?;

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct PollMetadata {
    pub title: String,
    pub description: String,
    pub poll_type: PollType,
}
impl PollMetadata {
    pub const TITLE: &'static [u8] = b"/poll/meta/title/";
    pub const DESCRIPTION: &'static [u8] = b"/poll/meta/desc/";
    pub const POLL_TYPE: &'static [u8] = b"/poll/meta/type/";
}
pub trait IPollMetaData<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn store(&self, core: &mut C, poll_id: u64) -> StdResult<()>;
    fn get(core: &C, poll_id: u64) -> StdResult<Self>;
    fn commit_title(&self, core: &mut C, poll_id: u64) -> StdResult<()>;
    fn commit_description(&self, core: &mut C, poll_id: u64) -> StdResult<()>;
    fn commit_poll_type(&self, core: &mut C, poll_id: u64) -> StdResult<()>;

    fn title(core: &C, poll_id: u64) -> StdResult<String>;
    fn description(core: &C, poll_id: u64) -> StdResult<String>;
    fn poll_type(core: &C, poll_id: u64) -> StdResult<PollType>;
}
impl<S, A, Q, C> IPollMetaData<S, A, Q, C> for PollMetadata
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
{
    fn store(&self, core: &mut C, poll_id: u64) -> StdResult<()> {
        self.commit_title(core, poll_id)?;
        self.commit_description(core, poll_id)?;
        self.commit_poll_type(core, poll_id)?;

        Ok(())
    }

    fn get(core: &C, poll_id: u64) -> StdResult<Self> {
        Ok(PollMetadata {
            description: Self::description(core, poll_id)?,
            title: Self::title(core, poll_id)?,
            poll_type: Self::poll_type(core, poll_id)?,
        })
    }

    fn commit_title(&self, core: &mut C, poll_id: u64) -> StdResult<()> {
        core.set_ns(Self::TITLE, &poll_id.to_be_bytes(), self.title.as_bytes())?;
        Ok(())
    }

    fn commit_description(&self, core: &mut C, poll_id: u64) -> StdResult<()> {
        core.set_ns(
            Self::DESCRIPTION,
            &poll_id.to_be_bytes(),
            self.description.as_bytes(),
        )?;
        Ok(())
    }

    fn commit_poll_type(&self, core: &mut C, poll_id: u64) -> StdResult<()> {
        core.set_ns(
            Self::POLL_TYPE,
            &poll_id.to_be_bytes(),
            self.poll_type.clone(),
        )?;
        Ok(())
    }

    fn title(core: &C, poll_id: u64) -> StdResult<String> {
        core.get_ns(Self::TITLE, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err(
                "failed to parse meta title from storage",
            ))?
    }

    fn description(core: &C, poll_id: u64) -> StdResult<String> {
        core.get_ns(Self::DESCRIPTION, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err(
                "failed to parse meta description from storage",
            ))?
    }

    fn poll_type(core: &C, poll_id: u64) -> StdResult<PollType> {
        core.get_ns(Self::POLL_TYPE, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err(
                "failed to parse meta poll type from storage",
            ))?
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    SiennaRewards,
    SiennaSwapParameters,
    Other,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollStatus {
    Active,
    Failed,
    Passed,
}
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VoteType {
    Yes,
    No,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct Vote {
    pub variant: VoteType,
    pub vote_power: Uint128,
    pub voter: CanonicalAddr,
}

impl Vote {
    pub const VOTE: &'static [u8] = b"/gov/vote/";
}
pub trait IVote<S, A, Q, C>
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q>,
    Self: Sized,
{
    fn store(&self, core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()>;
    fn get(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Self>;
    fn build_key(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Vec<u8>>;
    fn get_all(core: &C, poll_id: u64) -> StdResult<Vec<Self>>;
}

impl<S, A, Q, C> IVote<S, A, Q, C> for Vote
where
    S: Storage,
    A: Api,
    Q: Querier,
    C: Governance<S, A, Q> + Storage,
    Self: Sized,
{
    fn store(&self, core: &mut C, address: HumanAddr, poll_id: u64) -> StdResult<()> {
        let key = Self::build_key(core, address, poll_id)?;

        core.set_ns(Self::VOTE, &key, self.clone())?;

        Ok(())
    }
    fn get_all<'a>(core: &C, poll_id: u64) -> StdResult<Vec<Self>> {
        let poll_id = poll_id.to_be_bytes().to_vec();
        let mut key = Vec::with_capacity(poll_id.as_slice().len() + Self::VOTE.len());
        key.extend_from_slice(Self::VOTE);
        key.extend_from_slice(poll_id.as_slice());

        //Requires static lifetime for the slice. 
        let store = IterableStorage::<Vote>::new(Self::VOTE);

        let votes = store.iter(core)?.map(|vote| vote).collect();
        votes
    }
    fn get(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Self> {
        let key = Self::build_key(core, address, poll_id)?;

        let vote = core
            .get_ns::<Vote>(Self::VOTE, &key)?
            .ok_or(StdError::generic_err(
                "can't find vote for user on that poll",
            ))?;
        Ok(vote)
    }

    fn build_key(core: &C, address: HumanAddr, poll_id: u64) -> StdResult<Vec<u8>> {
        let address = core.canonize(address)?;
        let poll_id = poll_id.to_be_bytes().to_vec();

        //TODO: Cleaner way to handle this?
        let len = address.as_slice().len() + poll_id.as_slice().len();

        let mut key = Vec::with_capacity(len);
        key.extend_from_slice(poll_id.as_slice());
        key.extend_from_slice(address.as_slice());

        Ok(key)
    }
}

//Expiration utility

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Expiration {
    AtTime(u64),
}

impl fmt::Display for Expiration {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Expiration::AtTime(time) => write!(f, "Expiration time {}", time),
        }
    }
}

impl Expiration {
    pub fn is_expired(&self, block: &BlockInfo) -> bool {
        match self {
            Expiration::AtTime(time) => block.time >= *time,
        }
    }
}
