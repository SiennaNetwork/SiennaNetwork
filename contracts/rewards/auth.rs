pub use fadroma::scrt::{
    contract::{message, messages},
    cosmwasm_std::*,
    utils::{
        viewing_key::{ViewingKey, VIEWING_KEY_SIZE},
        storage::{ns_save, ns_load}
    },
};
use crate::core::ExternHook;

const ADMIN_KEY:    &[u8] = b"ltp5P6sFZT";
const VIEWING_KEYS: &[u8] = b"XXzo7ZXRJ2";

messages!(AuthHandle {
    ChangeAdmin      { address: HumanAddr }
    CreateViewingKey { entropy: String, padding: Option<String> }
    SetViewingKey    { key:     String, padding: Option<String> }
});

messages!(AuthQuery {
    Admin
});

messages!(AuthResponse {
    Admin {
        address: HumanAddr
    }
    CreateViewingKey {
        key: ViewingKey
    }
});

pub trait Auth<S: Storage, A: Api, Q: Querier>: ExternHook<S, A, Q> {

    fn init (&mut self, env: &Env, admin: &Option<HumanAddr>) -> StdResult<()> {
        let admin = admin.unwrap_or(env.message.sender.clone());
        self.admin.set(&self.api().canonical_address(&admin)?)
    }

    fn handle (&mut self, env: &Env, msg: &AuthHandle) -> StdResult<Option<HandleResponse>> {
        match msg {
            AuthHandle::ChangeAdmin { address } =>
                self.change_admin(env, address),
            AuthHandle::CreateViewingKey { entropy, .. } =>
                self.create_viewing_key(env, entropy),
            AuthHandle::SetViewingKey { key, .. } => 
                self.set_viewing_key(env, key)
        }
    }

    fn query (&self, msg: &Query) -> StdResult<Option<Binary>> {
        match msg {
            AuthQuery::Admin =>
                to_binary(&AuthResponse::Admin { address: self.load_admin()? })
        }
    }

    fn change_admin(&self, env: &Env, address: HumanAddr,) -> StdResult<HandleResponse> {
        self.assert_admin(env)?;
        self.save_admin(&address)?;
        Ok(HandleResponse::default())
    }

    fn load_admin (&self) -> StdResult<HumanAddr> {
        let result = self.storage().get(ADMIN_KEY);
        if let Some(bytes) = result {
            let admin = CanonicalAddr::from(bytes);
            self.api().human_address(&admin)
        } else {
            Ok(HumanAddr::default())
        }
    }

    fn save_admin(&mut self, address: &HumanAddr) -> StdResult<()> {
        let admin = self.api().canonical_address(address)?;
        self.storage().set(ADMIN_KEY, &admin.as_slice());
        Ok(())
    }

    fn assert_admin(&self, env: &Env) -> StdResult<()> {
        let admin = self.load_admin()?;
        if admin == env.message.sender {
            Ok(())
        } else {
            Err(StdError::unauthorized())
        }
    }

    fn create_viewing_key(&self, env: &Env, entropy: String) -> StdResult<HandleResponse> {
        let prng_seed = [ 
            env.block.time.to_be_bytes(),
            env.block.height.to_be_bytes() 
        ].concat();

        let key = ViewingKey::new(&env, &prng_seed, &(entropy).as_ref());
        let address = self.api().canonical_address(&env.message.sender)?;
        self.save_viewing_key(address.as_slice(), &key)?;
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&AuthResponse::CreateViewingKey { key })?)
        })
    }

    fn set_viewing_key(&self, env: &Env, key: String) -> StdResult<HandleResponse> {
        let key = ViewingKey(key);
        let address = self.api().canonical_address(&env.message.sender)?;
        self.save_viewing_key(address.as_slice(), &key)?;
        Ok(HandleResponse::default())
    }

    fn check_viewing_key (
        &self,
        provided_key: &ViewingKey,
        storage_key: &[u8]
    ) -> StdResult<()> {
        let stored_vk: Option<ViewingKey> = self.load_viewing_key(storage_key)?;
        if let Some(key) = stored_vk {
            if provided_key.check_viewing_key(&key.to_hashed()) {
                return Ok(());
            }
        }
        provided_key.check_viewing_key(&[0u8; VIEWING_KEY_SIZE]);
        return Err(StdError::unauthorized());
    }

    fn save_viewing_key(&mut self, key: &[u8], viewing_key: &ViewingKey) -> StdResult<()> {
        ns_save(self.storage(), VIEWING_KEYS, key, &viewing_key)
    }

    fn load_viewing_key(&self, key: &[u8],) -> StdResult<Option<ViewingKey>> {
        ns_load(self.storage(), VIEWING_KEYS, key)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{from_binary, HumanAddr};
    use cosmwasm_std::testing::{mock_dependencies, mock_env};

    #[test]
    fn test_vk_handle() {
        let ref mut deps = mock_dependencies(10, &[]);

        let sender = HumanAddr("sender".into());
        let sender_canonical = deps.api.canonical_address(&sender).unwrap();
        let env = mock_env(sender, &[]);

        let result = auth_handle(
            deps,
            env.clone(),
            AuthHandle::CreateViewingKey { entropy: "123".into(), padding: None },
            DefaultHandleImpl
        ).unwrap();

        let result: HandleAnswer = from_binary(&result.data.unwrap()).unwrap();
        let created_vk = match result {
            HandleAnswer::CreateViewingKey { key } => {
                key
            }
        };
        
        assert_eq!(created_vk, load_viewing_key(deps, sender_canonical.as_slice()).unwrap().unwrap());

        let auth_result = authenticate(&deps.storage, &ViewingKey("invalid".into()), sender_canonical.as_slice());
        assert_eq!(auth_result.unwrap_err(), StdError::unauthorized());

        let auth_result = authenticate(&deps.storage, &created_vk, sender_canonical.as_slice());
        assert!(auth_result.is_ok());

        let new_key = String::from("new_key");

        auth_handle(
            deps,
            env.clone(),
            AuthHandle::SetViewingKey { key: new_key.clone(), padding: None },
            DefaultHandleImpl
        ).unwrap();

        assert_eq!(ViewingKey(new_key.clone()), load_viewing_key(deps, sender_canonical.as_slice()).unwrap().unwrap());

        let auth_result = authenticate(&deps.storage, &ViewingKey(new_key), sender_canonical.as_slice());
        assert!(auth_result.is_ok());
    }

    #[test]
    fn test_admin_handle() {
        let ref mut deps = mock_dependencies(10, &[]);

        let admin = HumanAddr::from("admin");
        save_admin(deps.storage, deps.api, &admin).unwrap();

        let msg = AuthHandle::ChangeAdmin { 
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

        let msg = AuthHandle::ChangeAdmin { 
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
    fn test_auth_query() {
        let ref mut deps = mock_dependencies(10, &[]);

        let result = admin_query(deps, AuthQuery::Admin, DefaultQueryImpl).unwrap();

        let response: AdminQueryResponse = from_binary(&result).unwrap();
        assert!(response.address == HumanAddr::default());

        let admin = HumanAddr::from("admin");
        save_admin(deps.storage, deps.api, &admin).unwrap();

        let result = admin_query(deps, AuthQuery::Admin, DefaultQueryImpl).unwrap();

        let response: AdminQueryResponse = from_binary(&result).unwrap();
        assert!(response.address == admin);
    }
}
