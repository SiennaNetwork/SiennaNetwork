use fadroma::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};


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
    C: Composable<S, A, Q>,
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
    C: Composable<S, A, Q>,
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
        core.set_ns(Self::TITLE, &poll_id.to_be_bytes(), &self.title)?;
        Ok(())
    }

    fn commit_description(&self, core: &mut C, poll_id: u64) -> StdResult<()> {
        core.set_ns(Self::DESCRIPTION, &poll_id.to_be_bytes(), &self.description)?;
        Ok(())
    }

    fn commit_poll_type(&self, core: &mut C, poll_id: u64) -> StdResult<()> {
        core.set_ns(Self::POLL_TYPE, &poll_id.to_be_bytes(), &self.poll_type)?;
        Ok(())
    }

    fn title(core: &C, poll_id: u64) -> StdResult<String> {
        Ok(core
            .get_ns(Self::TITLE, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err(format!(
                "failed to parse meta title from storage, id: {}",
                poll_id
            )))?)
    }

    fn description(core: &C, poll_id: u64) -> StdResult<String> {
        Ok(core
            .get_ns(Self::DESCRIPTION, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err(format!(
                "failed to parse meta description from storage, id: {}",
                poll_id
            )))?)
    }

    fn poll_type(core: &C, poll_id: u64) -> StdResult<PollType> {
        Ok(core
            .get_ns(Self::POLL_TYPE, &poll_id.to_be_bytes())?
            .ok_or(StdError::generic_err(format!(
                "failed to parse meta poll type from storage, id: {}",
                poll_id
            )))?)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PollType {
    SiennaRewards,
    SiennaSwapParameters,
    Other,
}
