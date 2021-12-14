#![cfg(test)]
use crate::*;
use fadroma::{testing::*, auth::vk_auth::{authenticate, load_viewing_key}};

#[test]
fn test_vk_handle() {
    let ref mut deps = mock_dependencies(10, &[]);
    let sender  = HumanAddr("sender".into());
    let env     = mock_env(sender.clone(), &[]);
    let request = Handle::CreateViewingKey { entropy: "123".into(), padding: None };
    let result  = Contract::handle(deps, env.clone(), request).unwrap();
    let result: AuthResponse = from_binary(&result.data.unwrap()).unwrap();
    let created_vk = match result {
        AuthResponse::CreateViewingKey { key } => { key },
        _ => unimplemented!()
    };
    let sender_canonical = deps.api.canonical_address(&sender).unwrap();
    assert_eq!(created_vk, load_viewing_key(deps, sender_canonical.as_slice()).unwrap().unwrap());
    let auth_result = authenticate(&deps.storage, &ViewingKey("invalid".into()), sender_canonical.as_slice());
    assert_eq!(auth_result.unwrap_err(), StdError::unauthorized());
    let auth_result = authenticate(&deps.storage, &created_vk, sender_canonical.as_slice());
    assert!(auth_result.is_ok());
    let new_key = String::from("new_key");
    let request = Handle::SetViewingKey { key: new_key.clone(), padding: None };
    Contract::handle(deps, env.clone(), request).unwrap();
    assert_eq!(ViewingKey(new_key.clone()), load_viewing_key(deps, sender_canonical.as_slice()).unwrap().unwrap());
    let auth_result = authenticate(&deps.storage, &ViewingKey(new_key), sender_canonical.as_slice());
    assert!(auth_result.is_ok());
}

#[test]
fn test_admin_handle() {
    let ref mut deps = mock_dependencies(10, &[]);
    let admin = HumanAddr::from("admin");
    Auth::save_admin(deps, &admin).unwrap();
    let msg = AuthHandle::ChangeAdmin { address: HumanAddr::from("will fail") };
    let result = Contract::handle(deps, mock_env("unauthorized", &[]), Handle::Auth(msg)).unwrap_err();
    match result {
        StdError::Unauthorized { .. } => { },
        _ => panic!("Expected \"StdError::Unauthorized\", got: {:#?}", &result)
    };
    let new_admin = HumanAddr::from("new_admin");
    let msg = AuthHandle::ChangeAdmin { address: new_admin.clone() };
    Contract::handle(deps, mock_env(admin, &[]), Handle::Auth(msg)).unwrap();
    assert!(Auth::load_admin(deps).unwrap() == new_admin);
}

#[test]
fn test_auth_query() {
    let ref mut deps = mock_dependencies(10, &[]);
    match Contract::query(deps, Query::Auth(AuthQuery::Admin)).unwrap() {
        Response::Auth(AuthResponse::Admin { address }) =>
            assert_eq!(address, HumanAddr::default()),
        _ => unimplemented!()
    };
    let admin = HumanAddr::from("admin");
    Auth::save_admin(deps, &admin).unwrap();
    match Contract::query(deps, Query::Auth(AuthQuery::Admin)).unwrap() {
        Response::Auth(AuthResponse::Admin { address }) =>
            assert_eq!(address, admin),
        _ => unimplemented!()
    };
}
