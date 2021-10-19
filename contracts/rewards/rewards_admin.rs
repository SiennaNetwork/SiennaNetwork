use fadroma::scrt::cosmwasm_std::{
    HumanAddr, CanonicalAddr, StdResult, Extern, Env,
    Api, Querier, Storage, StdError, HandleResponse,
    Binary, to_binary
};
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};

const ADMIN_KEY: &[u8] = b"ltp5P6sFZT";

pub fn admin_handle(
    storage: &mut impl Storage,
    api:     &impl Api,
    env:     &Env,
    msg:     AdminHandleMsg,
    handle:  impl AdminHandle,
) -> StdResult<HandleResponse> {
    match msg {
        AdminHandleMsg::ChangeAdmin { address } => handle.change_admin(storage, api, env, address)
    }
}

pub fn admin_query<S: Storage, A: Api, Q: Querier>(
    storage: &mut impl Storage,
    api:     &impl Api,
    msg:     AdminQueryMsg,
    query:   impl AdminQuery,
) -> StdResult<Binary> {
    match msg {
        AdminQueryMsg::Admin => query.query_admin(storage, api)
    }
}

pub trait AdminHandle {
    fn change_admin(
        &self,
        storage: &mut impl Storage,
        api: &impl Api,
        env: &Env,
        address: HumanAddr,
    ) -> StdResult<HandleResponse> {
        assert_admin(storage, api, env)?;
        save_admin(storage, api, &address)?;
    
        Ok(HandleResponse::default())
    }
}

pub trait AdminQuery {
    fn query_admin(
        &self,
        storage: &impl Storage,
        api: &impl Api,
    )-> StdResult<Binary> {
        let address = load_admin(storage, api)?;
    
        to_binary(&AdminQueryResponse { 
            address
        })
    }
}

pub struct DefaultHandleImpl;

impl AdminHandle for DefaultHandleImpl { }

pub struct DefaultQueryImpl;

impl AdminQuery for DefaultQueryImpl { }

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AdminHandleMsg {
    ChangeAdmin {
        address: HumanAddr
    }
}

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AdminQueryMsg {
    Admin
}

#[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub struct AdminQueryResponse {
    pub address: HumanAddr
}

pub fn load_admin(
    storage: &impl Storage,
    api:     &impl Api
) -> StdResult<HumanAddr> {
    let result = storage.get(ADMIN_KEY);
    if let Some(bytes) = result {
        let admin = CanonicalAddr::from(bytes);
        api.human_address(&admin)
    } else {
        Ok(HumanAddr::default())
    }
}

pub fn save_admin(
    storage: &mut impl Storage,
    api:     &impl Api,
    address: &HumanAddr
) -> StdResult<()> {
    let admin = api.canonical_address(address)?;
    storage.set(ADMIN_KEY, &admin.as_slice());

    Ok(())
}

pub fn assert_admin(
    storage: &impl Storage,
    api: &impl Api,
    env: &Env
) -> StdResult<()> {
    let admin = load_admin(storage, api)?;

    if admin == env.message.sender {
        return Ok(());
    }

    Err(StdError::unauthorized())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::from_binary;
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn test_handle() {
        let ref mut deps = mock_dependencies(10, &[]);

        let admin = HumanAddr::from("admin");
        save_admin(deps.storage, deps.api, &admin).unwrap();

        let msg = AdminHandleMsg::ChangeAdmin { 
            address: HumanAddr::from("will fail")
        };

        let result = admin_handle(
            deps,
            mock_env("unauthorized", &[]),
            msg,
            DefaultHandleImpl
        ).unwrap_err();
        
        match result {
            StdError::Unauthorized { .. } => { },
            _ => panic!("Expected \"StdError::Unauthorized\"")
        };

        let new_admin = HumanAddr::from("new_admin");

        let msg = AdminHandleMsg::ChangeAdmin { 
            address: new_admin.clone()
        };

        admin_handle(
            deps,
            mock_env(admin, &[]),
            msg,
            DefaultHandleImpl
        ).unwrap();

        assert!(load_admin(deps.storage, deps.api).unwrap() == new_admin);
    }

    #[test]
    fn test_query() {
        let ref mut deps = mock_dependencies(10, &[]);

        let result = admin_query(deps, AdminQueryMsg::Admin, DefaultQueryImpl).unwrap();

        let response: AdminQueryResponse = from_binary(&result).unwrap();
        assert!(response.address == HumanAddr::default());

        let admin = HumanAddr::from("admin");
        save_admin(deps.storage, deps.api, &admin).unwrap();

        let result = admin_query(deps, AdminQueryMsg::Admin, DefaultQueryImpl).unwrap();

        let response: AdminQueryResponse = from_binary(&result).unwrap();
        assert!(response.address == admin);
    }
}

