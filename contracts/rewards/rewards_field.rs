use fadroma::scrt::cosmwasm_std::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{rc::Rc, cell::RefCell};

pub trait FieldFactory <S> {
    fn field <V> (self, key: &[u8]) -> Field<S, V>;
}

impl<S> FieldFactory<S> for Rc<RefCell<S>> {
    fn field <V> (self, key: &[u8]) -> Field<S, V> {
        Field::new(self.clone(), key.to_vec())
    }
}

pub struct Field <S, V> {
    storage:  Rc<RefCell<S>>,
    key:      Vec<u8>,
    value:    Option<V>,
    default:  Option<Box<fn()->StdResult<V>>>,
    required: Option<String>
}

impl<S, V> Field<S, V> {

    /// Define a new field
    pub fn new (storage: Rc<RefCell<S>>, key: Vec<u8>) -> Self {
        Self { storage, key, value: None, default: None, required: None }
    }

    /// Define a default value
    pub fn or (mut self, default: V) -> Self {
        let get_default = ||Ok(default);
        self.default = Some(Box::new(get_default));
        self
    }

    /// Define a default value
    pub fn or_else (mut self, get_default: fn()->StdResult<V>) -> Self {
        self.default = Some(Box::new(get_default));
        self
    }

    /// Define an error message for missing value with no default
    pub fn required (mut self, message: &str) -> Self {
        self.required = Some(message.to_string());
        self
    }

}

impl<S: ReadonlyStorage, V: DeserializeOwned> Field<S, V> {

    pub fn get (mut self) -> StdResult<V> {
        if let Some(value) = self.value {
            Ok(value)
        } else if let Some(data) = self.storage.borrow().get(&self.key) {
            let value = from_slice(&data)?;
            self.value = Some(value);
            Ok(value)
        } else if let Some(default) = self.default {
            self.value = Some(default()?);
            Ok(default)
        } else if let Some(message) = self.required {
            Err(StdError::generic_err(&message))
        } else {
            Err(StdError::generic_err("not in storage"))
        }
    }

}

impl<S: Storage, V: Serialize> Field<S, V> {

    pub fn set (mut self, value: &V) -> StdResult<()> {
        self.storage.borrow_mut().set(&self.key, &to_vec(value)?);
        self.value = Some(*value);
        Ok(())
    }

}
