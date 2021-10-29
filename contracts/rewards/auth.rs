use fadroma::*;
use fadroma::messages;
pub use fadroma::ViewingKey;

const ADMIN_KEY:    &[u8] = b"ltp5P6sFZT";
const VIEWING_KEYS: &[u8] = b"XXzo7ZXRJ2";

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum AuthHandle {
    ChangeAdmin      { address: HumanAddr },
    CreateViewingKey { entropy: String, padding: Option<String> },
    SetViewingKey    { key:     String, padding: Option<String> }
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum AuthQuery {
    Admin
}

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum AuthResponse {
    Admin {
        address: HumanAddr
    },
    CreateViewingKey {
        key: ViewingKey
    }
}

pub trait Auth<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q> {

    fn init (&mut self, env: &Env, admin: &Option<HumanAddr>) -> StdResult<()> {
        //self.set(b"/admin", &self.api().canonical_address(
            //&admin.unwrap_or(env.message.sender.clone())
        //)?)
        Ok(())
    }

    fn handle (&mut self, env: Env, msg: AuthHandle) -> StdResult<HandleResponse> {
        match msg {
            AuthHandle::ChangeAdmin { address } =>
                self.change_admin(env, address),
            AuthHandle::CreateViewingKey { entropy, .. } =>
                self.create_viewing_key(env, entropy),
            AuthHandle::SetViewingKey { key, .. } => 
                self.set_viewing_key(env, key)
        }
    }

    fn query (&self, msg: AuthQuery) -> StdResult<AuthResponse> {
        Ok(match msg {
            AuthQuery::Admin =>
                AuthResponse::Admin { address: self.load_admin()? }
        })
    }

    fn change_admin(&mut self, env: Env, address: HumanAddr) -> StdResult<HandleResponse> {
        self.assert_admin(&env)?;
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
        self.set(ADMIN_KEY, &admin.as_slice());
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

    fn create_viewing_key(&mut self, env: Env, entropy: String) -> StdResult<HandleResponse> {
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

    fn set_viewing_key(&mut self, env: Env, key: String) -> StdResult<HandleResponse> {
        let key = ViewingKey(key.to_string());
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

    fn save_viewing_key (&mut self, id: &[u8], viewing_key: &ViewingKey) -> StdResult<()> {
        self.set_ns(VIEWING_KEYS, id, &viewing_key)
    }

    fn load_viewing_key (&self, id: &[u8]) -> StdResult<Option<ViewingKey>> {
        self.get_ns(VIEWING_KEYS, id)
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use fadroma::*;
    use fadroma::testing::*;
    use super::Auth;

    #[test]
    fn test_vk_handle() {
        let ref mut deps = mock_dependencies(10, &[]);
        let sender  = HumanAddr("sender".into());
        let env     = mock_env(sender.clone(), &[]);
        let request = AuthHandle::CreateViewingKey { entropy: "123".into(), padding: None };
        let result  = Auth::handle(deps, env.clone(), request).unwrap();
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
        let request = AuthHandle::SetViewingKey { key: new_key.clone(), padding: None };
        Auth::handle(deps, env.clone(), request).unwrap();
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
        let result = Auth::handle(deps, mock_env("unauthorized", &[]), msg).unwrap_err();
        match result {
            StdError::Unauthorized { .. } => { },
            _ => panic!("Expected \"StdError::Unauthorized\"")
        };
        let new_admin = HumanAddr::from("new_admin");
        let msg = AuthHandle::ChangeAdmin { address: new_admin.clone() };
        Auth::handle(deps, mock_env(admin, &[]), msg).unwrap();
        assert!(Auth::load_admin(deps).unwrap() == new_admin);
    }

    #[test]
    fn test_auth_query() {
        let ref mut deps = mock_dependencies(10, &[]);
        match Auth::query(deps, AuthQuery::Admin).unwrap() {
            AuthResponse::Admin { address } => assert_eq!(address, HumanAddr::default()),
            _ => unimplemented!()
        };
        let admin = HumanAddr::from("admin");
        Auth::save_admin(deps, &admin).unwrap();
        match Auth::query(deps, AuthQuery::Admin).unwrap() {
            AuthResponse::Admin { address } => assert_eq!(address, admin),
            _ => unimplemented!()
        };
    }
}
