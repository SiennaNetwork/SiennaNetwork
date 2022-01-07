use fadroma::{
    schemars,
    admin,
    derive_contract::*,
};

#[interface(component(path = "admin"))]
pub trait Market { }
