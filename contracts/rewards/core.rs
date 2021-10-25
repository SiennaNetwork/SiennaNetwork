use fadroma::*;
use serde::{Serialize, de::DeserializeOwned};

pub trait Composable<S: Storage, A: Api, Q: Querier> {
    fn storage (self) -> S;
    fn set <V: Serialize> (&mut self, key: &[u8], value: V) -> StdResult<()> {
        self.storage().set(key, &to_vec(&value)?);
        Ok(())
    }
    fn set_ns <V: Serialize> (&self, ns: &[u8], key: &[u8], value: V) -> StdResult<()> {
        self.set(&concat(ns, key), value)
    }
    fn get <V: DeserializeOwned> (&self, key: &[u8]) -> StdResult<V> {
        if let Some(data) = self.storage().get(key) {
            Ok(from_slice(&data)?)
        } else {
            Err(StdError::generic_err(format!("{:?}: not found in storage", &key)))
        }
    }
    fn get_ns <V: DeserializeOwned> (&self, ns: &[u8], key: &[u8]) -> StdResult<V> {
        self.get(&concat(ns, key))
    }

    fn api (self) -> A;
    fn humanize <V: Humanize<U>, U: Canonize<V>> (&self, value: V) -> StdResult<U> {
        value.humanize(&self.api())
    }
    fn canonize <V: Canonize<U>, U: Humanize<V>> (&self, value: V) -> StdResult<U> {
        value.canonize(&self.api())
    }

    fn querier (self) -> Q;
}

impl<S: Storage, A: Api, Q: Querier> Composable<S, A, Q> for Extern<S, A, Q> {
    fn storage (self) -> S { self.storage }
    fn api     (self) -> A { self.api }
    fn querier (self) -> Q { self.querier }
}

pub struct Field <V> {
    key:      Vec<u8>,
    value:    Option<V>,
    default:  Option<V>,
    required: Option<String>
}

impl<V> Field<V> {

    /// Define a new field
    pub fn new (key: &[u8]) -> Self {
        Self { key: key.to_vec(), value: None, default: None, required: None }
    }

    /// Define a default value
    pub fn or (mut self, default: V) -> Self {
        self.default = Some(default);
        self
    }

    /// Define an error message for missing value with no default
    pub fn required (mut self, message: &str) -> Self {
        self.required = Some(message.to_string());
        self
    }

}

impl<V: Serialize + DeserializeOwned> Field<V> {

    pub fn get <S: Storage> (mut self, storage: &S) -> StdResult<V> {
        if let Some(value) = self.value {
            Ok(value)
        } else if let Some(data) = storage.get(&self.key) {
            let value = from_slice(&data)?;
            self.value = Some(value);
            Ok(self.value.unwrap())
        } else if let Some(default) = self.default {
            self.value = Some(default);
            Ok(self.value.unwrap())
        } else if let Some(message) = self.required {
            Err(StdError::generic_err(&message))
        } else {
            Err(StdError::generic_err("not in storage"))
        }
    }

    pub fn set <S: Storage> (mut self, storage: &mut S, value: V) -> StdResult<()> {
        storage.set(&self.key, &to_vec(&value)?);
        self.value = Some(value);
        Ok(())
    }

}
