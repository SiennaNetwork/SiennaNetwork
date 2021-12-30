use fadroma::*;
pub use fadroma::auth::vk::ViewingKey;

#[derive(Clone,Debug,PartialEq,serde::Serialize,serde::Deserialize,schemars::JsonSchema)]
#[serde(rename_all="snake_case")]
pub enum AuthHandle {
    NominateAdmin    { address: HumanAddr },
    BecomeAdmin      {},
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

pub const ADMIN:       &[u8] = b"/admin/current";
pub const NEXT_ADMIN:  &[u8] = b"/admin/next";
pub const VIEWING_KEY: &[u8] = b"/vk/";

pub fn err_no_admin <T> () -> StdResult<T> {
    Err(StdError::generic_err("This contract has no admin."))
}

pub trait Auth<S: Storage, A: Api, Q: Querier>: Composable<S, A, Q> {

    fn init (&mut self, env: &Env, admin: &Option<HumanAddr>) -> StdResult<()> {
        let admin = admin.as_ref().unwrap_or(&env.message.sender);
        self.set(ADMIN,      Some(self.api().canonical_address(&admin)))?;
        self.set(NEXT_ADMIN, Some(self.api().canonical_address(&admin)))?;
        Ok(())
    }

    fn handle (&mut self, env: Env, msg: AuthHandle) -> StdResult<HandleResponse> {
        match msg {
            AuthHandle::NominateAdmin { address } =>
                self.nominate_admin(env, address),
            AuthHandle::BecomeAdmin {} =>
                self.become_admin(env),
            AuthHandle::CreateViewingKey { entropy, .. } =>
                self.create_vk(env, entropy),
            AuthHandle::SetViewingKey { key, .. } => 
                self.set_vk(env, key.into())
        }
    }

    fn query (&self, msg: AuthQuery) -> StdResult<AuthResponse> {
        Ok(match msg {
            AuthQuery::Admin => AuthResponse::Admin {
                address: self.api().human_address(&self.load_admin()?)?
            }
        })
    }

    fn load_admin (&self) -> StdResult<CanonicalAddr> {
        if let Some(admin) = self.get::<CanonicalAddr>(ADMIN)? {
            Ok(admin)
        } else {
            err_no_admin()
        }
    }

    fn nominate_admin (&mut self, env: Env, address: HumanAddr) -> StdResult<HandleResponse> {
        self.assert_admin(&env)?;
        self.set(NEXT_ADMIN, Some(self.api().canonical_address(&address)?))?;
        Ok(HandleResponse::default())
    }

    fn become_admin (&mut self, env: Env) -> StdResult<HandleResponse> {
        if let Some(next_admin) = self.get::<CanonicalAddr>(NEXT_ADMIN)? {
            if next_admin == self.api().canonical_address(&env.message.sender)? {
                self.set(ADMIN, Some(next_admin))?;
                return Ok(HandleResponse::default())
            }
        }
        Err(StdError::unauthorized())
    }

    fn assert_admin (&self, env: &Env) -> StdResult<()> {
        if self.load_admin()? == self.api().canonical_address(&env.message.sender)? {
            Ok(())
        } else {
            Err(StdError::unauthorized())
        }
    }

    fn create_vk (&mut self, env: Env, entropy: String) -> StdResult<HandleResponse> {
        let prng_seed = [env.block.time.to_be_bytes(), env.block.height.to_be_bytes()];
        let key       = ViewingKey::new(&env, &prng_seed.concat(), &(entropy).as_ref());
        let address   = self.api().canonical_address(&env.message.sender)?;
        self.save_vk(address.as_slice(), &key)?;
        Ok(HandleResponse {
            messages: vec![],
            log:      vec![],
            data:     Some(to_binary(&AuthResponse::CreateViewingKey { key })?)
        })
    }

    fn set_vk (&mut self, env: Env, key: ViewingKey) -> StdResult<HandleResponse> {
        let address = self.api().canonical_address(&env.message.sender)?;
        self.save_vk(address.as_slice(), &key)?;
        Ok(HandleResponse::default())
    }

    fn check_vk (&self, provided_key: &ViewingKey, user_id: &[u8]) -> StdResult<()> {
        let stored_vk: Option<ViewingKey> = self.load_vk(user_id)?;
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
        self.set_ns(VIEWING_KEY, id, &key)
    }

    fn load_vk (&self, id: &[u8]) -> StdResult<Option<ViewingKey>> {
        self.get_ns(VIEWING_KEY, id)
    }

}

#[cfg(test)] mod test {
    use crate::*;

    #[test]
    fn test_vk_handle() {
        let ref mut deps = mock_dependencies(20, &[]);
        let user  = HumanAddr("user".into());

        let result  = Contract::handle(
            deps,
            mock_env(user.clone(), &[]),
            Handle::CreateViewingKey { entropy: "123".into(), padding: None }
        ).unwrap();
        let result: AuthResponse = from_binary(&result.data.unwrap()).unwrap();
        let created_vk = match result {
            AuthResponse::CreateViewingKey { key } => { key },
            _ => unimplemented!()
        };

        let user_canon = deps.api.canonical_address(&user).unwrap();
        let loaded_vk = Auth::load_vk(deps, user_canon.as_slice()).unwrap().unwrap();
        assert_eq!(created_vk, loaded_vk);

        let invalid_vk  = ViewingKey("invalid".into());
        let result = Auth::check_vk(deps, &invalid_vk, user_canon.as_slice());
        assert_eq!(result.unwrap_err(), StdError::unauthorized());

        let result = Auth::check_vk(deps, &created_vk, user_canon.as_slice());
        assert!(result.is_ok());

        let new_vk = String::from("new_vk");
        let request = Handle::SetViewingKey { key: new_vk.clone(), padding: None };
        Contract::handle(
            deps,
            mock_env(user.clone(), &[]).clone(),
            request
        ).unwrap();
        assert_eq!(ViewingKey(new_vk.clone()), Auth::load_vk(deps, user_canon.as_slice()).unwrap().unwrap());

        let result = Auth::check_vk(deps, &ViewingKey(new_vk), user_canon.as_slice());
        assert!(result.is_ok());
    }

    #[test]
    fn test_admin_handle() {
        let ref mut deps = mock_dependencies(20, &[]);

        let admin = HumanAddr::from("admin");
        let admin_canon = deps.api.canonical_address(&admin).unwrap();
        Composable::set(deps, ADMIN, admin_canon.clone()).unwrap();

        let result = Contract::handle(
            deps,
            mock_env("unauthorized", &[]),
            Handle::Auth(AuthHandle::NominateAdmin { address: HumanAddr::from("will fail") })
        ).unwrap_err();
        match result {
            StdError::Unauthorized { .. } => { },
            _ => panic!("Expected \"StdError::Unauthorized\", got: {:#?}", &result)
        };

        let new_admin = HumanAddr::from("new_admin");
        let new_admin_canon = deps.api.canonical_address(&new_admin).unwrap();
        let msg = AuthHandle::NominateAdmin { address: new_admin.clone() };
        Contract::handle(deps, mock_env(admin, &[]), Handle::Auth(msg)).unwrap();
        assert_eq!(
            Auth::load_admin(deps).unwrap(),
            admin_canon
        );

        let msg = AuthHandle::BecomeAdmin {};
        Contract::handle(deps, mock_env(new_admin, &[]), Handle::Auth(msg)).unwrap();
        assert_eq!(
            Auth::load_admin(deps).unwrap(),
            new_admin_canon
        );
    }

    #[test]
    fn test_auth_query() {
        let ref mut deps = mock_dependencies(10, &[]);
        assert_eq!(
            Contract::query(deps, Query::Auth(AuthQuery::Admin)),
            err_no_admin()
        );
        let admin = HumanAddr::from("admin");
        let admin_canon = deps.api.canonical_address(&admin).unwrap();
        Composable::set(deps, ADMIN, admin_canon).unwrap();
        match Contract::query(deps, Query::Auth(AuthQuery::Admin)).unwrap() {
            Response::Auth(AuthResponse::Admin { address }) =>
                assert_eq!(address, admin),
            _ => unimplemented!()
        };
    }
}
