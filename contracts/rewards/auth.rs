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
        self.save_admin(&admin)?;
        self.save_next_admin(&admin)?;
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
                address: self.load_admin()?
            }
        })
    }

    fn nominate_admin (&mut self, env: Env, address: HumanAddr) -> StdResult<HandleResponse> {
        self.assert_admin(&env)?;
        self.save_next_admin(&address);
        Ok(HandleResponse::default())
    }

    fn become_admin (&mut self, env: Env) -> StdResult<HandleResponse> {
        if self.load_next_admin()? == env.message.sender {
            self.save_admin(&env.message.sender)?;
            Ok(HandleResponse::default())
        } else {
            Err(StdError::unauthorized())
        }
    }

    fn assert_admin (&self, env: &Env) -> StdResult<()> {
        if self.load_admin()? == env.message.sender {
            Ok(())
        } else {
            Err(StdError::unauthorized())
        }
    }

    fn load_admin (&self) -> StdResult<HumanAddr> {
        if let Some(bytes) = self.storage().get(ADMIN) {
            self.humanize(CanonicalAddr::from(bytes))
        } else {
            err_no_admin()
        }
    }

    fn load_next_admin (&self) -> StdResult<HumanAddr> {
        if let Some(bytes) = self.storage().get(NEXT_ADMIN) {
            self.humanize(CanonicalAddr::from(bytes))
        } else {
            err_no_admin()
        }
    }

    fn save_admin (&mut self, admin: &HumanAddr) -> StdResult<()> {
        let admin = self.api().canonical_address(admin)?;
        self.storage_mut().set(ADMIN, admin.as_slice());
        Ok(())
    }

    fn save_next_admin (&mut self, next_admin: &HumanAddr) -> StdResult<()> {
        let next_admin = self.api().canonical_address(next_admin)?;
        self.storage_mut().set(NEXT_ADMIN, next_admin.as_slice());
        Ok(())
    }

    fn create_vk (&mut self, env: Env, entropy: String) -> StdResult<HandleResponse> {
        let key = ViewingKey::new(
            &env,
            &[env.block.time.to_be_bytes(), env.block.height.to_be_bytes()].concat(),
            &(entropy).as_ref()
        );
        self.save_vk(&env.message.sender, &key)?;
        Ok(HandleResponse {
            messages: vec![],
            log:      vec![],
            data:     Some(to_binary(&AuthResponse::CreateViewingKey { key })?)
        })
    }

    fn set_vk (&mut self, env: Env, key: ViewingKey) -> StdResult<HandleResponse> {
        self.save_vk(&env.message.sender, &key)?;
        Ok(HandleResponse::default())
    }

    fn check_vk (&self, address: &HumanAddr, provided_vk: &ViewingKey) -> StdResult<()> {
        let stored_vk: Option<ViewingKey> = self.load_vk(address)?;
        if let Some(ref key) = stored_vk {
            if provided_vk.check_viewing_key(&key.to_hashed()) {
                Ok(())
            } else {
                Err(StdError::unauthorized())
            }
        } else {
            Err(StdError::unauthorized())
        }
    }

    fn save_vk (&mut self, address: &HumanAddr, key: &ViewingKey) -> StdResult<()> {
        let id = self.api().canonical_address(address)?;
        self.set_ns(VIEWING_KEY, id.as_slice(), &key)
    }

    fn load_vk (&self, address: &HumanAddr) -> StdResult<Option<ViewingKey>> {
        let id = self.api().canonical_address(address)?;
        self.get_ns(VIEWING_KEY, id.as_slice())
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
        let loaded_vk = Auth::load_vk(deps, &user).unwrap().unwrap();
        assert_eq!(created_vk, loaded_vk);

        let invalid_vk  = ViewingKey("invalid".into());
        let result = Auth::check_vk(deps, &user, &invalid_vk);
        assert_eq!(result.unwrap_err(), StdError::unauthorized());

        let result = Auth::check_vk(deps, &user, &created_vk);
        assert!(result.is_ok());

        let new_vk = String::from("new_vk");
        let request = Handle::SetViewingKey { key: new_vk.clone(), padding: None };
        Contract::handle(
            deps,
            mock_env(user.clone(), &[]).clone(),
            request
        ).unwrap();
        assert_eq!(ViewingKey(new_vk.clone()), Auth::load_vk(deps, &user).unwrap().unwrap());

        let result = Auth::check_vk(deps, &user, &ViewingKey(new_vk));
        assert!(result.is_ok());
    }

    #[test]
    fn test_admin_handle() {
        let ref mut deps = mock_dependencies(20, &[]);

        let admin = HumanAddr::from("admin");
        Auth::save_admin(deps, &admin).unwrap();

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
        let msg = AuthHandle::NominateAdmin { address: new_admin.clone() };
        Contract::handle(deps, mock_env(admin.clone(), &[]), Handle::Auth(msg)).unwrap();
        assert_eq!(Auth::load_admin(deps).unwrap(), admin);

        let msg = AuthHandle::BecomeAdmin {};
        Contract::handle(deps, mock_env(new_admin.clone(), &[]), Handle::Auth(msg)).unwrap();
        assert_eq!(Auth::load_admin(deps).unwrap(), new_admin);
    }

    #[test]
    fn test_auth_query() {
        let ref mut deps = mock_dependencies(10, &[]);
        assert_eq!(
            Contract::query(deps, Query::Auth(AuthQuery::Admin)),
            err_no_admin()
        );

        let admin = HumanAddr::from("admin");
        Auth::save_admin(deps, &admin).unwrap();

        match Contract::query(deps, Query::Auth(AuthQuery::Admin)).unwrap() {
            Response::Auth(AuthResponse::Admin { address }) =>
                assert_eq!(address, admin),
            _ => unimplemented!()
        };
    }
}
