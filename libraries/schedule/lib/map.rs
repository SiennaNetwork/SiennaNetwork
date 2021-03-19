//! Minimal KV store that passes serialization

use std::mem::replace;

/// Just a wrapped `Vec` with `get` and `insert` methods
/// because `serde_json_wasm` doesn't serialize maps (even in v0.3)
///
/// This acts as a KV map but serializes as an array of [K,V] pairs:
/// new keys are appended to the end, but there is no check
/// for keys being unique during deserialization, and it's
/// the 1st instance of a given key that "counts".
#[derive(serde::Serialize,serde::Deserialize,Clone,Debug,PartialEq,schemars::JsonSchema)]
pub struct LinearMap<K, V>(Vec<(K, V)>);
impl<K: PartialEq, V> LinearMap<K, V> {
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
