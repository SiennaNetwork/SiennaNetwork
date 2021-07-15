use fadroma::scrt::{
    cosmwasm_std::{Uint128, CanonicalAddr, StdResult, StdError, Storage, ReadonlyStorage},
    storage::{Readonly, Writable},
    utils::Uint256
};

macro_rules! error { ($info:expr) => { Err(StdError::generic_err($info)) }; }
