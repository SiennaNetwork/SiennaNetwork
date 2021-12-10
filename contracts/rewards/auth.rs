use fadroma::*;
pub use fadroma::ViewingKey;

pub const ADMIN_KEY:    &[u8] = b"ltp5P6sFZT";
pub const VIEWING_KEYS: &[u8] = b"XXzo7ZXRJ2";

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
    Admin { address: HumanAddr },
    CreateViewingKey { key: ViewingKey }
}

pub trait Auth<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q> {

    fn init (&mut self, env: &Env, admin: &Option<HumanAddr>) -> StdResult<()> {
        self.save_admin(&admin.as_ref().unwrap_or(&env.message.sender))?;
        Ok(())
    }

    fn handle (&mut self, env: Env, msg: AuthHandle) -> StdResult<HandleResponse> {
        match msg {
            AuthHandle::ChangeAdmin { address } =>
                self.change_admin(env, address),
            AuthHandle::CreateViewingKey { entropy, .. } =>
                self.create_vk(env, entropy),
            AuthHandle::SetViewingKey { key, .. } => 
                self.set_vk(env, key)
        }
    }

    fn query (&self, msg: AuthQuery) -> StdResult<AuthResponse> {
        Ok(match msg {
            AuthQuery::Admin => AuthResponse::Admin { address: self.load_admin()? }
        })
    }

    fn change_admin(&mut self, env: Env, address: HumanAddr) -> StdResult<HandleResponse> {
        self.assert_admin(&env)?;
        self.save_admin(&address)?;
        Ok(HandleResponse::default())
    }

    fn load_admin (&self) -> StdResult<HumanAddr> {
        self.get(ADMIN_KEY)?.map_or(
            Ok(HumanAddr::default()),
            |result|self.api().human_address(&result),
        )
    }

    fn save_admin(&mut self, address: &HumanAddr) -> StdResult<()> {
        let admin = self.api().canonical_address(address)?;
        self.set(ADMIN_KEY, Some(&admin))?;
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

    fn create_vk(&mut self, env: Env, entropy: String) -> StdResult<HandleResponse> {
        let prng_seed = [env.block.time.to_be_bytes(), env.block.height.to_be_bytes()];
        let key       = ViewingKey::new(&env, &prng_seed.concat(), &(entropy).as_ref());
        let address   = self.api().canonical_address(&env.message.sender)?;
        self.save_vk(address.as_slice(), &key)?;
        Ok(HandleResponse {
            messages: vec![],
            log: vec![],
            data: Some(to_binary(&AuthResponse::CreateViewingKey { key })?)
        })
    }

    fn set_vk(&mut self, env: Env, key: String) -> StdResult<HandleResponse> {
        let key = ViewingKey(key.to_string());
        let address = self.api().canonical_address(&env.message.sender)?;
        self.save_vk(address.as_slice(), &key)?;
        Ok(HandleResponse::default())
    }

    fn check_vk (
        &self,
        provided_key: &ViewingKey,
        storage_key: &[u8]
    ) -> StdResult<()> {
        let stored_vk: Option<ViewingKey> = self.load_vk(storage_key)?;
        if let Some(ref key) = stored_vk {
            if provided_key.check_viewing_key(&key.to_hashed()) {
                Ok(())
            } else {
                Err(StdError::unauthorized())
            }
        } else {
            Err(StdError::unauthorized())
        }
    }

    fn save_vk (&mut self, id: &[u8], key: &ViewingKey) -> StdResult<()> {
        self.set_ns(VIEWING_KEYS, id, &key)
    }

    fn load_vk (&self, id: &[u8]) -> StdResult<Option<ViewingKey>> {
        self.get_ns(VIEWING_KEYS, id)
    }

}
