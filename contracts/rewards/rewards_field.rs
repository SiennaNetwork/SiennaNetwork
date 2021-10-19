use fadroma::scrt::cosmwasm_std::*;
use serde::{Serialize, de::DeserializeOwned};
use std::{rc::Rc, cell::{RefCell, RefMut}};

pub trait FieldFactory <S> {
    fn field <V> (self, key: &[u8]) -> Field<S, V>;
}

impl<S> FieldFactory<S> for Rc<RefCell<S>> {
    fn field <V> (self, key: &[u8]) -> Field<S, V> {
        Field::new(self.clone(), key.to_vec())
    }
}

pub struct Field <S, V> {
    storage: Rc<RefCell<S>>,
    key:     Vec<u8>,
    value:   Option<V>
}

impl<S, V> Field<S, V> {
    pub fn new (storage: Rc<RefCell<S>>, key: Vec<u8>) -> Self {
        Self { storage, key, value: None }
    }
}

impl<S: ReadonlyStorage, V: Copy + DeserializeOwned> Field<S, V> {
    pub fn value (mut self) -> StdResult<V> {
        match self.value {
            Some(value) => Ok(value),
            None => {
                match self.storage.borrow().get(&self.key) {
                    Some(data) => {
                        let value = from_slice(&data)?;
                        self.value = Some(value);
                        Ok(value)
                    },
                    None => Err(StdError::generic_err("not in storage"))
                }
            }
        }
    }
    pub fn value_or_default (mut self, default: V) -> StdResult<V> {
        match self.value {
            Some(value) => Ok(value),
            None => {
                match self.storage.borrow().get(&self.key) {
                    Some(data) => {
                        let value = from_slice(&data)?;
                        self.value = Some(value);
                        Ok(value)
                    },
                    None => {
                        self.value = default;
                        Ok(default)
                    }
                }
            }
        }
    }
    pub fn value_or_err (mut self, message: &str) -> StdResult<V> {
        match self.value {
            Some(value) => Ok(value),
            None => {
                match self.storage.borrow().get(&self.key) {
                    Some(data) => {
                        let value = from_slice(&data)?;
                        self.value = Some(value);
                        Ok(value)
                    },
                    None => Err(StdError::generic_err(message))
                }
            }
        }
    }

}

impl<S: ReadonlyStorage + Storage, V: Serialize> Field<S, V> {
    pub fn store (mut self, value: &V) -> StdResult<()> {
        {
            let mut storage: RefMut<_> = self.storage.borrow_mut();
            storage.set(&self.key, &to_vec(value)?);
        }
        self.value = Some(value);
        Ok(())
    }
}
