use fadroma::{
    schemars,
    cosmwasm_std::{Storage, StdResult, StdError, Env},
    storage::{save, load},
    auth::ViewingKey
};

use serde::{Serialize, Deserialize};

/// A single viewing key that is shared throughout the
/// protocol so that private data can be viewied internally.
#[derive(Serialize, Deserialize, schemars::JsonSchema, Clone, Debug)]
pub struct MasterKey(ViewingKey);

impl MasterKey {
    const KEY: &'static[u8] = b"master_key";

    #[inline]
    pub fn new(env: &Env, seed: &[u8], entropy: &[u8]) -> Self {
        Self(ViewingKey::new(env, seed, entropy))
    }

    pub fn check(storage: &impl Storage, other: &Self) -> StdResult<()> {
        let key = Self::load(storage)?;

        if key.0.check_viewing_key(&other.0.to_hashed()) {
            Ok(())
        } else {
            Err(StdError::unauthorized())
        }
    }

    #[inline]
    pub fn save(&self, storage: &mut impl Storage) -> StdResult<()> {
        save(storage, Self::KEY, self)
    }

    #[inline]
    pub fn load(storage: &impl Storage) -> StdResult<Self> {
        Ok(load(storage, Self::KEY)?.unwrap())
    }
}
