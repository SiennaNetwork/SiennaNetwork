use fadroma::scrt::cosmwasm_std::*;
use serde::{Serialize, de::DeserializeOwned};

pub trait Composable<S: Storage, A: Api, Q: Querier> {
    fn storage (self) -> S;

    fn set <V: Serialize> (&mut self, key: &[u8], value: V) -> StdResult<()> {
        self.storage().set(key, &to_vec(&value)?);
        Ok(())
    }

    fn get <V: DeserializeOwned> (&self, key: &[u8]) -> StdResult<Option<V>> {
        if let Some(data) = self.storage().get(key) {
            Ok(Some(from_slice(&data)?))
        } else {
            Ok(None)
        }
    }

    fn api (self) -> A;

    fn humanize <T> (&self, value: T) -> T { value }
    fn canonize <T> (&self, value: T) -> T { value }

    fn querier (self) -> Q;
}

impl<S: Storage, A: Api, Q: Querier> Composable<S, A, Q> for Extern<S, A, Q> {
    fn storage (self) -> S { self.storage }
    fn api     (self) -> A { self.api }
    fn querier (self) -> Q { self.querier }
}
