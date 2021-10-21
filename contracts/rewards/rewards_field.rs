use fadroma::scrt::cosmwasm_std::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{rc::Rc, cell::RefCell};

pub trait FieldFactory <S: Storage, A: Api, Q: Querier> {
    fn field <V> (&self, key: &[u8]) -> Field<S, A, Q, V>;
}

impl<S: Storage, A: Api, Q: Querier> FieldFactory<S, A, Q>
for Rc<RefCell<Extern<S, A, Q>>> {
    fn field <V> (&self, key: &[u8]) -> Field<S, A, Q, V> {
        Field::new(self.clone(), key.to_vec())
    }
}

pub struct Field <S: Storage, A: Api, Q: Querier, V> {
    deps:     Rc<RefCell<Extern<S, A, Q>>>,
    key:      Vec<u8>,
    value:    Option<V>,
    default:  Option<V>,
    required: Option<String>
}

impl<S: Storage, A: Api, Q: Querier, V> Field<S, A, Q, V> {

    /// Define a new field
    pub fn new (deps: Rc<RefCell<S>>, key: Vec<u8>) -> Self {
        Self { deps, key, value: None, default: None, required: None }
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

impl<S: Storage, A: Api, Q: Querier, V: DeserializeOwned>
Field<S, A, Q, V> {

    pub fn get (mut self) -> StdResult<V> {
        if let Some(value) = self.value {
            Ok(value)
        } else if let Some(data) = self.deps.borrow().storage.get(&self.key) {
            let value = from_slice(&data)?;
            self.value = Some(value);
            Ok(value)
        } else if let Some(default) = self.default {
            self.value = Some(default);
            Ok(default)
        } else if let Some(message) = self.required {
            Err(StdError::generic_err(&message))
        } else {
            Err(StdError::generic_err("not in storage"))
        }
    }

}

impl<S: Storage, A: Api, Q: Querier, V: Serialize>
Field<S, A, Q, V> {

    pub fn set (mut self, value: &V) -> StdResult<()> {
        self.deps.borrow_mut().storage.set(&self.key, &to_vec(value)?);
        self.value = Some(*value);
        Ok(())
    }

}
