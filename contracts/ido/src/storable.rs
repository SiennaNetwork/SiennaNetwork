use fadroma::scrt::cosmwasm_std::{Api, Extern, Querier, StdResult, Storage};
use fadroma::scrt::storage::{
    load as storage_load, remove as storage_remove, save as storage_save,
};
use serde::{de::DeserializeOwned, Serialize};

/// Todo: Move this to Fadroma storage crate
///
/// Trait that will add storage options to your struct,
/// you will have to implement `key()` method that will let storage
/// know where to save or from where to load your data.
///
/// Optionally, you can define the namespace of your struct that
/// will namespace it when storing.
pub trait Storable: Serialize + DeserializeOwned {
    /// Static: namespace for self
    fn namespace() -> Vec<u8> {
        Vec::new()
    }

    /// Storage key used for saving Self
    fn key(&self) -> StdResult<Vec<u8>>;

    /// Concat of namespace and key
    fn concat_key(key: &[u8]) -> Vec<u8> {
        let mut ns = Self::namespace();
        ns.extend_from_slice(key);

        ns
    }

    /// Save Self in the storage
    fn save<S: Storage, A: Api, Q: Querier>(&self, deps: &mut Extern<S, A, Q>) -> StdResult<()> {
        let key = self.key()?;
        let key = key.as_slice();
        let key = Self::concat_key(&key);
        storage_save::<Self, _>(&mut deps.storage, &key.as_slice(), &self)
    }

    /// Remove Self from storage
    fn remove<S: Storage, A: Api, Q: Querier>(self, deps: &mut Extern<S, A, Q>) -> StdResult<()> {
        let key = self.key()?;
        let key = key.as_slice();
        let key = Self::concat_key(&key);
        storage_remove(&mut deps.storage, &key.as_slice());

        Ok(())
    }

    /// Static: Load Self from the storage
    fn load<S: Storage, A: Api, Q: Querier>(
        deps: &Extern<S, A, Q>,
        key: &[u8],
    ) -> StdResult<Option<Self>> {
        let key = Self::concat_key(key);

        // println!("{:?} {:?}", key, String::from(key.as_slice()));
        storage_load::<Self, _>(&deps.storage, key.as_slice())
    }

    /// Static: Save Self in the storage
    fn static_save<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        key: &[u8],
        item: &Self,
    ) -> StdResult<()> {
        let key = Self::concat_key(key);
        storage_save::<Self, _>(&mut deps.storage, &key.as_slice(), item)
    }

    /// Static: Remove Self from storage
    fn static_remove<S: Storage, A: Api, Q: Querier>(
        deps: &mut Extern<S, A, Q>,
        key: &[u8],
    ) -> StdResult<()> {
        let key = Self::concat_key(key);
        storage_remove(&mut deps.storage, &key.as_slice());

        Ok(())
    }
}
