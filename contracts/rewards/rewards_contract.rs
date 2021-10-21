use std::{rc::Rc, cell::RefCell};

use fadroma::scrt::{
    cosmwasm_std::*,
    BLOCK_SIZE,
    callback::ContractInstance as ContractLink,
    toolkit::snip20,
    snip20_api::ISnip20,
    addr::{Humanize, Canonize},
};

use crate::{
    rewards_api::*,
    rewards_math::*,
    rewards_field::*,
    rewards_pool::Pool,
    rewards_admin::{
        DefaultHandleImpl as AdminHandle,
        admin_handle, AdminHandleMsg, assert_admin,
    },
    rewards_vk::{
        ViewingKey,
        auth_handle, authenticate, AuthHandleMsg,
        DefaultHandleImpl as AuthHandle
    },
};

macro_rules! tx_ok {
    () => {
        Ok(HandleResponse::default())
    };
    ($($msg:expr),*) => {
        Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None })
    };
}

pub struct Contract <S: Storage, A: Api, Q: Querier, E> {
    deps: Rc<RefCell<Extern<S, A, Q>>>,
    env:  E,

    admin:          Field<S, A, Q, CanonicalAddr>,

    /// Used to check own balance.
    self_link:      Field<S, A, Q, ContractLink<CanonicalAddr>>,

    /// Contract accepts transactions in `lp_token`s.
    /// The address of the `lp_token` can be provided later
    /// to avoid a circular dependency during deployment.
    lp_token:       Field<S, A, Q, ContractLink<CanonicalAddr>>,

    /// Contract distributes rewards in Reward Tokens.
    /// For this, it must know its own balance in the `reward_token`s.
    /// For that, it needs a reference to its own address+code_hash
    /// and a viewing key in `reward_token`.
    reward_token:   Field<S, A, Q, ContractLink<CanonicalAddr>>,

    viewing_key:    Field<S, A, Q, ViewingKey>,
}

impl <S: Storage, A: Api, Q: Querier> Contract <S, A, Q, Env> {
    pub fn new_ro (
        deps: &Extern<S, A, Q>
    ) -> Contract<S, A, Q, ()> {
        let deps = Rc::new(RefCell::new(*deps));
        Contract {
            env: (),
            deps,
            admin:        deps.field(b"/admin"),
            self_link:    deps.field(b"/self"),
            lp_token:     deps.field(b"/lp_token"),
            reward_token: deps.field(b"/reward_token"),
            viewing_key:  deps.field(b"/viewing_key")
        }
    }

    pub fn new_rw (
        deps: &mut Extern<S, A, Q>,
        env:  Env
    ) -> Contract<S, A, Q, Env> {
        let deps = Rc::new(RefCell::new(*deps));
        Contract {
            env,
            deps,
            admin:        deps.field(b"/admin"),
            self_link:    deps.field(b"/self"),
            lp_token:     deps.field(b"/lp_token"),
            reward_token: deps.field(b"/reward_token"),
            viewing_key:  deps.field(b"/viewing_key")
        }
    }
}

type InitResult   = StdResult<InitResponse>;
type HandleResult = StdResult<HandleResponse>;

impl <S: Storage, A: Api, Q: Querier, E> Contract <S, A, Q, E> {

    fn load_reward_balance (&self) -> StdResult<Uint128> {
        let reward_token_link = self.reward_token.get()?;
        let reward_token = ISnip20::attach(&reward_token_link.humanize(&self.deps.api)?);
        let mut reward_balance = reward_token
            .query(&self.deps.querier)
            .balance(
                &self.self_link.get()?.humanize(&self.deps.api)?.address,
                &self.viewing_key.get()?.0
            )?;

        let lp_token_link = self.lp_token.get()?;
        if lp_token_link == reward_token_link {
            let lp_balance = Pool::new(self.deps).locked.get()?;
            reward_balance = (reward_balance - lp_balance)?;
        }

        Ok(reward_balance)
    }
}

impl <S: Storage, A: Api, Q: Querier> Contract<S, A, Q, Env> {

    pub fn init (&mut self, msg: Init) -> InitResult {
        self.self_link.set(&ContractLink {
            address:   self.env.contract.address.clone(),
            code_hash: self.env.contract_code_hash.clone()
        }.canonize(&self.deps.api)?);
        let admin = msg.admin.unwrap_or(self.env.message.sender.clone());
        self.admin.set(&self.deps.api.canonical_address(&admin)?)?;
        self.reward_token.set(&msg.reward_token.canonize(&self.deps.api)?);
        self.viewing_key.set(&msg.viewing_key)?;
        if let Some(lp_token) = msg.lp_token {
            self.save_lp_token(&lp_token)?;
        }
        self.save_initial_pool_config(&msg)?;
        Ok(InitResponse {
            log:      vec![],
            messages: vec![
                ISnip20::attach(&msg.reward_token).set_viewing_key(&msg.viewing_key.0)?
            ],
        })
    }

    fn save_lp_token (&mut self, link: &ContractLink<HumanAddr>) -> StdResult<()> {
        self.lp_token.set(&link.canonize(&self.deps.api)?)
    }

    fn save_initial_pool_config (&mut self, msg: &Init) -> StdResult<()> {
        // Reward pool has configurable parameters:
        let mut pool = Pool::new(self.deps);

        #[cfg(feature="pool_liquidity_ratio")]
        pool.created.set(&self.env.block.time)?;

        #[cfg(feature="global_ratio")]
        pool.global_ratio.set(&msg.ratio.unwrap_or((1u128.into(), 1u128.into())))?;

        #[cfg(feature="age_threshold")]
        pool.threshold.set(&msg.threshold.unwrap_or(DAY))?;

        #[cfg(feature="claim_cooldown")]
        pool.cooldown.set(&msg.cooldown.unwrap_or(DAY))?;

        Ok(())
    }

    pub fn handle (&mut self, msg: Handle) -> HandleResult {
        match msg {

            Handle::ChangeAdmin { address } => admin_handle(
                &mut *self.deps.borrow_mut(),
                &self.deps.api,
                &self.env,
                AdminHandleMsg::ChangeAdmin { address },
                AdminHandle
            ),

            Handle::SetProvidedToken { address, code_hash } => {
                self.assert_admin()?;
                self.save_lp_token(&ContractLink { address, code_hash })?;
                tx_ok!()
            },

            #[cfg(feature="global_ratio")]
            Handle::ChangeRatio { numerator, denominator } => {
                self.assert_admin()?;
                Pool::new(self.deps).global_ratio.set(&(numerator.into(), denominator.into()))?;
                tx_ok!()
            },

            #[cfg(feature="age_threshold")]
            Handle::ChangeThreshold { threshold } => {
                self.assert_admin()?;
                Pool::new(self.deps).threshold.set(&threshold)?;
                tx_ok!()
            },

            #[cfg(feature="claim_cooldown")]
            Handle::ChangeCooldown { cooldown } => {
                self.assert_admin()?;
                Pool::new(self.deps).cooldown.set(&cooldown)?;
                tx_ok!()
            },

            #[cfg(feature="pool_closes")]
            Handle::ClosePool { message } => {
                self.assert_admin()?;
                Pool::new(self.deps).at(self.env.block.time)?.close(message)?;
                tx_ok!()
            }

            Handle::ReleaseSnip20 { snip20, recipient, key } =>
                self.release_snip20(snip20, recipient, key),

            Handle::CreateViewingKey { entropy, padding } =>
                self.create_viewing_key(entropy, padding),
            Handle::SetViewingKey { key, padding } =>
                self.set_viewing_key(key, padding),

            Handle::Lock { amount } =>
                self.lock(amount),
            Handle::Retrieve { amount } =>
                self.retrieve(amount),
            Handle::Claim {} =>
                self.claim()

        }
    }

    fn assert_admin (&mut self) -> StdResult<()> {
        assert_admin(&mut *self.deps.borrow(), &self.deps.api, &self.env)
    }

    // Snip20 tokens sent to this contract can be transferred
    // The goal is allow the contract to not act as burner for
    // snip20 tokens in case sent here. 
    pub fn release_snip20 (
        &mut self,
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    ) -> HandleResult {
        self.assert_admin()?;

        let recipient = recipient.unwrap_or(self.env.message.sender.clone());

        let reward_token = self.reward_token.get()?.humanize(&self.deps.api)?;

        // Update the viewing key if the supplied
        // token info is for the reward token
        if reward_token == snip20 {
            self.viewing_key.set(&ViewingKey(key.clone()))?;
        }

        tx_ok!(
            snip20::increase_allowance_msg(
                recipient,
                Uint128(u128::MAX),
                Some(self.env.block.time + 86400000), // One day duration
                None,
                BLOCK_SIZE,
                snip20.code_hash.clone(),
                snip20.address.clone()
            )?,
            snip20::set_viewing_key_msg(
                key,
                None,
                BLOCK_SIZE,
                snip20.code_hash,
                snip20.address
            )?
        )
    }

    pub fn create_viewing_key (&mut self, entropy: String, padding: Option<String>) -> HandleResult {
        self.auth_handle(AuthHandleMsg::CreateViewingKey { entropy, padding })
    }

    pub fn set_viewing_key (&mut self, key: String, padding: Option<String>) -> HandleResult {
        self.auth_handle(AuthHandleMsg::SetViewingKey { key, padding })
    }

    fn auth_handle (&mut self, msg: AuthHandleMsg) -> HandleResult {
        auth_handle(
            &mut *self.deps.borrow(),
            &self.deps.api,
            &self.env,
            msg,
            AuthHandle
        )
    }

    /// User can lock some liquidity provision tokens.
    pub fn lock (&mut self, amount: Amount) -> HandleResult {
        // If the pool is closed, users can only retrieve all their liquidity tokens
        #[cfg(feature="pool_closes")]
        if let Some(closed_response) = self.close_handler()? {
            return Ok(closed_response)
        }

        tx_ok!(ISnip20::attach(&self.lp_token.get()?.humanize(&self.deps.api)?).transfer_from(
            &self.env.message.sender,
            &self.env.contract.address,
            Pool::new(self.deps)
                .at(self.env.block.time)?
                .user(self.deps.api.canonical_address(&self.env.message.sender)?)
                .lock_tokens(amount)?)?)
    }

    /// User can always get their liquidity provision tokens back.
    pub fn retrieve (&mut self, amount: Amount) -> HandleResult {
        // If the pool is closed, users can only retrieve all their liquidity tokens
        #[cfg(feature="pool_closes")]
        if let Some(closed_response) = self.close_handler()? {
            return Ok(closed_response)
        }

        tx_ok!(ISnip20::attach(&self.lp_token.get()?.humanize(&self.deps.api)?).transfer(
            &self.env.message.sender,
            Pool::new(self.deps)
                .at(self.env.block.time)?
                .user(self.deps.api.canonical_address(&self.env.message.sender)?)
                .retrieve_tokens(amount)?)?)
    }

    /// User can receive rewards after having provided liquidity.
    pub fn claim (&mut self) -> HandleResult {

        let mut response = HandleResponse { messages: vec![], log: vec![], data: None };

        // If the pool has been closed, also return the user their liquidity tokens
        #[cfg(feature="pool_closes")]
        if let Some(mut closed_response) = self.close_handler()? {
            response.messages.append(&mut closed_response.messages);
            response.log.append(&mut closed_response.log);
        }

        // Get the reward balance of the contract
        let reward_balance = self.load_reward_balance()?;

        // Compute the reward portion for this user.
        // May return error if portion is zero.
        let reward = Pool::new(self.deps)
            .at(self.env.block.time)?
            .with_balance(reward_balance)
            .user(self.deps.api.canonical_address(&self.env.message.sender)?)
            .claim_reward()?;

        // Add the reward to the response
        let reward_token = ISnip20::attach(&self.reward_token.get()?.humanize(&self.deps.api)?);
        response.messages.push(reward_token.transfer(&self.env.message.sender, reward)?);
        response.log.push(LogAttribute { key: "reward".into(), value: reward.into() });

        Ok(response)
    }

    #[cfg(feature="pool_closes")]
    /// Returns either a "pool closed" HandleResponse
    /// (containing a LP Token transaction to return
    /// all of the user's locked LP the first time)
    /// or None if the pool isn't closed.
    pub fn close_handler (&mut self) -> StdResult<Option<HandleResponse>> {
        let pool = Pool::new(self.deps);
        Ok(if let Some((_, close_message)) = pool.closed.get()? {
            let mut response = HandleResponse::default();
            response.log.push(LogAttribute {
                key:  "closed".into(),
                value: close_message
            });
            let mut user = pool
                .at(self.env.block.time)?
                .user(self.deps.api.canonical_address(&self.env.message.sender)?);
            let locked = user.retrieve_tokens(user.locked.get()?)?;
            if locked > Amount::zero() {
                response.messages.push(
                    ISnip20::attach(&self.lp_token.get()?.humanize(&self.deps.api)?).transfer(
                        &self.env.message.sender,
                        locked
                    )?
                );
                response.log.push(LogAttribute {
                    key:  "retrieved".into(),
                    value: locked.into()
                });
            };
            Some(response)
        } else {
            None
        })
    }

}

impl<S: Storage, A: Api, Q: Querier> Contract<S, A, Q, ()> {

    pub fn query (
        self,
        msg: Query
    ) -> StdResult<Response> {
        match msg {
            Query::Admin {} =>
                self.admin(),
            Query::PoolInfo { at } =>
                self.pool_info(at),
            Query::UserInfo { at, address, key } =>
                self.user_info(at, address, key),
            Query::TokenInfo {} =>
                self.token_info(),
            Query::Balance { address, key } =>
                self.balance(address, key),
        }
    }

    pub fn admin (self) -> StdResult<Response> {
        Ok(Response::Admin {
            address: self.deps.api.human_address(&self.admin.get()?)?
        })
    }

    pub fn pool_info (self, at: Time) -> StdResult<Response> {

        let balance = self.load_reward_balance()?;
        let pool = Pool::new(self.deps).at(at)?.with_balance(balance);
        let pool_last_update = pool.timestamp.get()?;
        if at < pool_last_update {
            return Err(StdError::generic_err("this contract does not store history"))
        }

        Ok(Response::PoolInfo {
            it_is_now: at,

            lp_token:       self.lp_token.get()?.humanize(&self.deps.api)?,
            reward_token:   self.reward_token.get()?.humanize(&self.deps.api)?,

            #[cfg(feature="pool_closes")]
            pool_closed:    self.close_message(&pool)?,

            pool_last_update,
            pool_lifetime:  pool.lifetime()?,
            pool_locked:    pool.locked.get()?,

            pool_claimed:   pool.claimed.get()?,
            pool_balance:   pool.balance(),

            #[cfg(feature="age_threshold")]
            pool_threshold: pool.threshold.get()?,

            #[cfg(feature="claim_cooldown")]
            pool_cooldown:  pool.cooldown.get()?,

            #[cfg(feature="pool_liquidity_ratio")]
            pool_liquid:    pool.liquidity_ratio()?,

            /* todo add balance/claimed/total in rewards token */
        })
    }

    pub fn user_info (
        self,
        at:      Time,
        address: HumanAddr,
        key:     String
    ) -> StdResult<Response> {
        let address = self.deps.api.canonical_address(&address)?;
        authenticate(&*self.deps.borrow(), &ViewingKey(key), address.as_slice())?;

        let pool = Pool::new(self.deps).at(at)?;
        let pool_last_update = pool.timestamp.get()?;
        if at < pool_last_update {
            return Err(StdError::generic_err("no time travel"))
        }

        let reward_balance = self.load_reward_balance()?;
        let user = pool.with_balance(reward_balance).user(address);
        let user_last_update = user.timestamp.get()?;
        if at < user_last_update {
            return Err(StdError::generic_err("no time travel"))
        }

        Ok(Response::UserInfo {
            it_is_now: at,

            pool_last_update,
            pool_lifetime:  pool.lifetime()?,
            pool_locked:    pool.locked.get()?,

            #[cfg(feature="pool_closes")]
            pool_closed:    self.close_message(&pool)?,

            user_last_update,
            user_lifetime:  user.lifetime()?,
            user_locked:    user.locked.get()?,
            user_share:     user.share(HUNDRED_PERCENT)?.low_u128().into(),
            user_earned:    user.earned()?,
            user_claimed:   user.claimed.get()?,
            user_claimable: user.claimable()?,

            #[cfg(feature="age_threshold")]
            user_age:       user.present()?,

            #[cfg(feature="claim_cooldown")]
            user_cooldown:  user.cooldown()?
        })
    }

    #[cfg(feature="pool_closes")]
    fn close_message (self, pool: &Pool<S, A, Q>) -> StdResult<Option<String>> {
        Ok(if let Some((_, close_message)) = pool.closed.get()? {
            Some(close_message)
        } else {
            None
        })
    }

    pub fn token_info (self) -> StdResult<Response> {
        let link = self.lp_token.get()?.humanize(&self.deps.api)?;
        let info = ISnip20::attach(&link).query(&self.querier).token_info()?;
        Ok(Response::TokenInfo {
            name:         format!("Sienna Rewards: {}", info.name),
            symbol:       "SRW".into(),
            decimals:     1,
            total_supply: None
        })
    }

    pub fn balance (self, address: HumanAddr, key: String) -> StdResult<Response> {
        let address = self.deps.api.canonical_address(&address)?;
        self.deps.borrow().authenticate(&ViewingKey(key), address.as_slice())?;
        Ok(Response::Balance {
            amount: Pool::new(self.deps).user(address).locked.get()?
        })
    }

}
