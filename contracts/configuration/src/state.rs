use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, CanonicalAddr, HumanAddr, ReadonlyStorage, Storage};
use cosmwasm_storage::{
    singleton, singleton_read, PrefixedStorage, ReadonlyPrefixedStorage, ReadonlySingleton,
    Singleton,
};
//pub static CONFIG_KEY: &[u8] = b"config";

pub const ADDRESSES_KEY: &[u8] = b"addresses";
pub const UINTS_KEY: &[u8] = b"uints";

pub fn get_address<S: ReadonlyStorage>(storage: &S, key: &[u8]) -> Option<Binary> {
    let store = ReadonlyPrefixedStorage::new(ADDRESSES_KEY, storage);
    let result = store.get(key)?;
    Some(Binary::from(result.as_slice()))
}

pub fn set_address<S: Storage>(storage: &mut S, key: &[u8], value: HumanAddr) {
    let mut store = PrefixedStorage::new(ADDRESSES_KEY, storage);
    store.set(key, value.as_str().as_bytes());
}

pub fn get_uint<S: ReadonlyStorage>(storage: &S, key: &[u8]) -> Option<Binary> {
    let store = ReadonlyPrefixedStorage::new(UINTS_KEY, storage);
    let result = store.get(key)?;
    Some(Binary::from(result.as_slice()))
}

pub fn set_uint<S: Storage>(storage: &mut S, key: &[u8], value: HumanAddr) {
    let mut store = PrefixedStorage::new(UINTS_KEY, storage);
    store.set(key, value.as_str().as_bytes());
}
