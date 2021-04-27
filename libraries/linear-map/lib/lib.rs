//! Minimal KV store that passes `serde_json_wasm` serialization

use std::mem::replace;
use cosmwasm_std::{StdResult, Api, HumanAddr, CanonicalAddr};

/// Just a wrapped `Vec` with `get` and `insert` methods.
///
/// Acts as a KV map but serializes as an array of [K,V] pairs:
/// * new keys are appended to the end, existing keys are updated in place
/// * there is no check for keys being unique during deserialization
/// * in case of duplicate keys, it's the 1st instance of a given key that "counts".
///
/// It's like this because `serde_json_wasm` doesn't serialize maps (`HashMap`, `BTreeMap`).
/// This is true even in versions more recent than the default for SecretNetwork:
/// the `serialize_map` method contains a single `unreachable!()` panic. It's not immediately
/// obvious why this remains the case the case; perhaps iterating over of those is somehow more
/// expensive on a blockchain? In any case, in the absence of gas benchmarks it's pointless
/// to risk altering the default execution environment (of which `serde_json_wasm` is part),
/// even though there's no obvious reason why that wouldn't work.
#[derive(serde::Serialize,serde::Deserialize,Clone,Debug,PartialEq,schemars::JsonSchema)]
pub struct LinearMap<K, V>(pub Vec<(K, V)>);

impl <K: PartialEq, V> LinearMap<K, V> {
    pub fn new () -> Self { Self(Vec::new()) }
    pub fn get (&self, key: &K) -> Option<&V> {
        for (k, v) in self.0.iter() {
            if key == k {
                return Some(v)
            }
        }
        None
    }
    pub fn insert (&mut self, key: K, value: V) -> Option<V> {
        let mut found = None;
        for (i, (k, _)) in self.0.iter_mut().enumerate() {
            if key == *k {
                found = Some(i);
                break
            }
        }
        if let Some(index) = found {
            Some(replace(&mut self.0[index], (key, value)).1)
        } else {
            self.0.push((key, value));
            None
        }
    }
}

impl <V: Clone> LinearMap<HumanAddr, V> {
    pub fn canonize <A: Api> (&self, api: &A) -> StdResult<LinearMap<CanonicalAddr, V>> {
        let canonized: Result<Vec<_>,_> = self.0.iter().map(
            |(human, value)| match api.canonical_address(human) {
                Ok(canon) => Ok((canon, value.clone())),
                Err(e)    => Err(e)
            }).collect();
        Ok(LinearMap(canonized?))
    }
}

impl <V: Clone> LinearMap<CanonicalAddr, V> {
    pub fn humanize <A: Api> (&self, api: &A) -> StdResult<LinearMap<HumanAddr, V>> {
        let humanized: Result<Vec<_>,_> = self.0.iter().map(
            |(canon, value)| match api.human_address(canon) {
                Ok(human) => Ok((human, value.clone())),
                Err(e)    => Err(e)
            }).collect();
        Ok(LinearMap(humanized?))
    }
}
