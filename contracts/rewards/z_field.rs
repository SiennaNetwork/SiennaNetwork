use fadroma::*;
use serde::{Serialize, de::DeserializeOwned};

//pub struct Field <V> {
    //key:      Vec<u8>,
    //value:    Option<V>,
    //default:  Option<V>,
    //required: Option<String>
//}

//impl<V> Field<V> {

    ///// Define a new field
    //pub fn new (key: &[u8]) -> Self {
        //Self { key: key.to_vec(), value: None, default: None, required: None }
    //}

    ///// Define a default value
    //pub fn or (mut self, default: V) -> Self {
        //self.default = Some(default);
        //self
    //}

    ///// Define an error message for missing value with no default
    //pub fn required (mut self, message: &str) -> Self {
        //self.required = Some(message.to_string());
        //self
    //}

//}

//impl<V: Serialize + DeserializeOwned> Field<V> {

    //pub fn get <S: Storage> (mut self, storage: &S) -> StdResult<V> {
        //if let Some(value) = self.value {
            //Ok(value)
        //} else if let Some(data) = storage.get(&self.key) {
            //let value = from_slice(&data)?;
            //self.value = Some(value);
            //Ok(self.value.unwrap())
        //} else if let Some(default) = self.default {
            //self.value = Some(default);
            //Ok(self.value.unwrap())
        //} else if let Some(message) = self.required {
            //Err(StdError::generic_err(&message))
        //} else {
            //Err(StdError::generic_err("not in storage"))
        //}
    //}

    //pub fn set <S: Storage> (mut self, storage: &mut S, value: V) -> StdResult<()> {
        //storage.set(&self.key, &to_vec(&value)?);
        //self.value = Some(value);
        //Ok(())
    //}

//}
