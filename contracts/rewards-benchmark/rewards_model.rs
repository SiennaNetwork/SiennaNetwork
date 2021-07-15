pub use crate::rewards_math::*;
pub use crate::rewards_pool::*;
pub use crate::rewards_user::*;

use fadroma::scrt::{cosmwasm_std::CanonicalAddr, storage::{Readonly, Writable}};

/// Reward pool
pub struct Pool <S> {
    storage: S,
    now:     Option<Monotonic>
}

impl <S> Pool<S> {
    /// Create a new pool with a storage handle
    pub fn new (storage: S) -> Self {
        Self { storage, now: None }
    }
    /// Add the current time to the pool for operations that need it
    pub fn at (self, now: Monotonic) -> Self {
        Self { storage: self.storage, now: Some(now) }
    }
    /// Get an individual user from the pool
    pub fn user (&self, address: CanonicalAddr) -> User<S> {
        // variant: existed check here; new_user for first lock?
        User { pool: *self, address }
    }
}

impl<S: ReadonlyStorage> Readonly<S> for Pool<&S> {
    fn storage (&self) -> &S { self.storage }
}

impl<S: ReadonlyStorage> PoolReadonly<S> for Pool<&S> {
    fn now (&self) -> StdResult<Monotonic> {
        self.now.ok_or(StdError::generic_err("current time not set"))
    }
}

impl<S: Storage> Writable<S> for Pool<&S> {
    fn storage_mut (&mut self) -> &mut S { &mut *self.storage }
}

impl<S: Storage> PoolWritable<S> for Pool<&S> {}

pub struct User <S> {
    pool:    Pool<S>,
    address: CanonicalAddr
}

impl<S: ReadonlyStorage> Readonly<S> for User<&S> {
    fn storage (&self) -> &S { &self.pool.storage }
}

impl<S: ReadonlyStorage> UserReadonly<S> for User<&S> {
    fn pool (&self) -> &Pool<&S> {
        &self.pool
    }
    fn address (&self) -> &[u8] {
        self.address.as_slice()
    }
    // trait fields WHEN???
}

impl<S: ReadonlyStorage> Readonly<S> for User<&mut S> {
    fn storage (&self) -> &S { &self.pool.storage }
}

impl<S: Storage + ReadonlyStorage> Writable<S> for User<&mut S> {
    fn storage_mut (&mut self) -> &mut S { &mut self.pool.storage }
}

impl<S: ReadonlyStorage> UserReadonly<S> for User<&mut S> {
    fn pool (&self) -> &mut Pool<&S> {
        &self.pool
    }
    fn address (&self) -> &[u8] {
        self.address.as_slice()
    }
    // trait fields WHEN???
}

impl<S: Storage + ReadonlyStorage> UserWritable<S> for User<&mut S> {}
