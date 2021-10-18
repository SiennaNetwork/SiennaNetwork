//! Since there is a limited amount of rewards for each day,
//! they need to be distributed among the top liquidity providers.
//!
//! By locking funds, the user starts accruing a lifetime share of the pool
//! which entitles them to an equal percent of the total rewards,
//! which are distributed daily and the user can claim one per day.
//!
//! This lifetime share fluctuates as a result of the other users
//! locking and unlocking amounts of funds for different amounts of time.
//! If it remains constant or increases, users are guaranteed a new reward
//! every day. If they fall behind, they may be able to claim rewards
//! less frequently, and need to lock more tokens to restore their place
//! in the queue.

// Endgame mode: give exactly this many tokens to the contract to
// pay out remaining rewards.

#[cfg(browser)] #[macro_use] extern crate wasm_bindgen;
#[cfg(test)] #[macro_use] extern crate kukumba;
#[cfg(any(test, browser))] mod rewards_harness;
#[cfg(test)] mod rewards_test;
#[cfg(test)] mod rewards_test_2;

pub mod rewards_math;   use rewards_math::*;
pub mod rewards_algo;   use rewards_algo::*;
pub mod rewards_config; use rewards_config::*;

use fadroma::scrt::{
    cosmwasm_std::*,
    BLOCK_SIZE,
    callback::ContractInstance as ContractLink,
    contract::*,
    toolkit::snip20,
    snip20_api::ISnip20,
    vk::{ViewingKey,
         auth_handle, authenticate, AuthHandleMsg,
         DefaultHandleImpl as AuthHandle},
    admin::{DefaultHandleImpl as AdminHandle,
            admin_handle, AdminHandleMsg, load_admin,
            assert_admin, save_admin}};

pub fn init (
    deps: &mut Extern<impl Storage, impl Api, impl Querier>,
    env:  Env,
    msg:  Init
) -> StdResult<InitResponse> {
    Contract { env: Some(env), ..deps }.init(msg)
}

pub fn handle (
    deps: &mut Extern<impl Storage, impl Api, impl Querier>,
    env:  Env,
    msg:  Handle
) -> StdResult<HandleResponse> {
    Contract { env: Some(env), ..deps }.handle(msg)
}

pub fn query (
    deps: &Extern<impl Storage, impl Api, impl Querier>,
    msg:  Query
) -> StdResult<QueryResponse> {
    Contract { env: None, ..deps }.query(msg)
}

struct Contract <S, A, Q> {
    storage: S,
    api:     A,
    querier: Q,
    env:     Option<Env>
}

macro_rules! tx_ok {
    () => {
        Ok(HandleResponse::default())
    };
    ($($msg:expr),*) => {
        Ok(HandleResponse { messages: vec![$($msg),*], log: vec![], data: None })
    };
}

pub const DAY: Time = 86400; // seconds in 24 hours

message!(Init {
    admin:        Option<HumanAddr>,
    lp_token:     Option<ContractLink<HumanAddr>>,
    reward_token: ContractLink<HumanAddr>,
    viewing_key:  ViewingKey,
    ratio:        Option<Ratio>,
    threshold:    Option<Time>,
    cooldown:     Option<Time>
});

messages!(Handle {
    ChangeAdmin {}

    SetProvidedToken {}

    ChangeRatio {}
    ChangeThreshold {}
    ChangeCooldown {}

    ClosePool {}
    ReleaseSnip20 {}

    CreateViewingKey {}
    SetViewingKey {}

    Lock {}
    Retrieve {}
    Claim {}
});

impl <S: Storage, A: Api, Q: Querier> Contract<&mut S, A, Q> {

    pub fn init (self, msg: Init) -> StdResult<InitResponse> {
        let Init {
            admin,
            lp_token,
            reward_token,
            viewing_key,
            ratio,
            threshold,
            cooldown
        } = msg;

        // Contract has an admin who can do admin stuff.
        save_admin(self, &admin.unwrap_or(self.env.message.sender))?;

        // Contract accepts transactions in `lp_token`s.
        // The address of the `lp_token` can be provided later
        // to avoid a circular dependency during deployment.
        if let Some(lp_token) = lp_token {
            save_lp_token(&mut self.storage, &self.api, &lp_token)?;
        }

        // Contract distributes rewards in Reward Tokens.
        // For this, it must know its own balance in the `reward_token`s.
        // For that, it needs a reference to its own address+code_hash
        // and a viewing key in `reward_token`.

        save_reward_token(&mut self.storage, &self.api, &reward_token)?;

        let set_vk = ISnip20::attach(&reward_token).set_viewing_key(&viewing_key.0)?;

        save_viewing_key(&mut self.storage, &viewing_key)?;

        save_self_reference(&mut self.storage, &self.api, &ContractLink {
            address:   self.env.contract.address,
            code_hash: self.env.contract_code_hash
        })?;

        // Reward pool has configurable parameters:

        #[cfg(feature="pool_liquidity_ratio")]
        self.pool().set_created(&self.env.block.time)?;

        #[cfg(feature="global_ratio")]
        self.pool().configure_ratio(&ratio.unwrap_or((1u128.into(), 1u128.into())))?;

        #[cfg(feature="age_threshold")]
        self.pool().configure_threshold(&threshold.unwrap_or(DAY))?;

        #[cfg(feature="claim_cooldown")]
        self.pool().configure_cooldown(&cooldown.unwrap_or(DAY))?;

        // TODO remove global state from scrt-contract
        // define field! and addr_field! macros instead -
        // problem here is identifier concatenation
        // and making each field a module is ugly
        Ok(InitResponse { messages: vec![set_vk], log: vec![] })
    }

    fn pool <T> (self) -> Pool<T> {
        Pool::new(self.storage)
    }

    pub fn handle (self, msg: Handle) -> StdResult<HandleResponse> {
        Err(StdError::generic_err("not implemented"))
    }

    /// Set the contract admin.
    #[handle] pub fn change_admin (self, address: HumanAddr) {
        let msg = AdminHandleMsg::ChangeAdmin { address };
        admin_handle(self, self.env, msg, AdminHandle)
    }

    /// Set the active asset token.
    // Resolves circular reference when initializing the benchmark -
    // they need to know each other's addresses to use initial allowances
    #[handle] pub fn set_provided_token (self, address: HumanAddr, code_hash: String) {
        assert_admin(&self, &self.env)?;
        save_lp_token(&mut self.storage, &self.api, &ContractLink { address, code_hash })?;
        tx_ok!()
    }

    #[cfg(feature="global_ratio")]
    #[handle] pub fn change_ratio (self, numerator: Amount, denominator: Amount) {
        assert_admin(&self, &self.env)?;
        Pool::new(&mut self.storage)
            .configure_ratio(&(numerator.into(), denominator.into()))?;
        tx_ok!()
    }

    #[cfg(feature="age_threshold")]
    #[handle] pub fn change_threshold (self, threshold: Time) {
        assert_admin(&self, &self.env)?;
        Pool::new(&mut self.storage)
            .configure_threshold(&threshold)?;
        tx_ok!()
    }

    #[cfg(feature="claim_cooldown")]
    #[handle] pub fn change_cooldown (self, cooldown: Time) {
        assert_admin(&self, &self.env)?;
        Pool::new(&mut self.storage)
            .configure_cooldown(&cooldown)?;
        tx_ok!()
    }

    #[cfg(feature="pool_closes")]
    #[handle] pub fn close_pool (self, message: String) {
        assert_admin(&self, &self.env)?;
        Pool::new(&mut self.storage)
            .at(self.env.block.time)
            .close(message)?;
        tx_ok!()
    }

    // Snip20 tokens sent to this contract can be transferred
    // The goal is allow the contract to not act as burner for
    // snip20 tokens in case sent here. 
    #[handle] pub fn release_snip20  (
        self,
        snip20:    ContractLink<HumanAddr>,
        recipient: Option<HumanAddr>,
        key:       String
    ) -> StdResult<HandleResponse> {
        assert_admin(&self, &self.env)?;

        let recipient = recipient.unwrap_or(self.env.message.sender);

        let reward_token = load_reward_token(&self.storage, &self.api)?;

        // Update the viewing key if the supplied
        // token info for is the reward token
        if reward_token == snip20 {
            save_viewing_key(&mut self.storage, &ViewingKey(key.clone()))?
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

    // actions that are performed by users ----------------------------------------------------

    /// User can request a new viewing key for oneself.
    #[handle] pub fn create_viewing_key (self, entropy: String, padding: Option<String>) {
        let msg = AuthHandleMsg::CreateViewingKey { entropy, padding: None };
        auth_handle(self, self.env, msg, AuthHandle)
    }

    /// User can set own viewing key to a known value.
    #[handle] pub fn set_viewing_key (self, key: String, padding: Option<String>) {
        let msg = AuthHandleMsg::SetViewingKey { key, padding: None };
        auth_handle(self, self.env, msg, AuthHandle)
    }

    /// User can lock some liquidity provision tokens.
    #[handle] pub fn lock (self, amount: Amount) {
        // If the pool is closed, users can only retrieve all their liquidity tokens
        #[cfg(feature="pool_closes")]
        if let Some(closed_response) = self.close_handler()? {
            return Ok(closed_response)
        }

        tx_ok!(ISnip20::attach(&load_lp_token(&self.storage, &self.api)?).transfer_from(
            &self.env.message.sender,
            &self.env.contract.address,
            Pool::new(&mut self.storage)
                .at(self.env.block.time)
                .user(self.api.canonical_address(&self.env.message.sender)?)
                .lock_tokens(amount)?)?)
    }

    /// User can always get their liquidity provision tokens back.
    #[handle] pub fn retrieve (self, amount: Amount) {
        // If the pool is closed, users can only retrieve all their liquidity tokens
        #[cfg(feature="pool_closes")]
        if let Some(closed_response) = self.close_handler()? {
            return Ok(closed_response)
        }

        tx_ok!(ISnip20::attach(&load_lp_token(&self.storage, &self.api)?).transfer(
            &self.env.message.sender,
            Pool::new(&mut self.storage)
                .at(self.env.block.time)
                .user(self.api.canonical_address(&self.env.message.sender)?)
                .retrieve_tokens(amount)?)?)
    }

    /// User can receive rewards after having provided liquidity.
    #[handle] pub fn claim (self) {

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
        let reward = Pool::new(&mut self.storage)
            .at(self.env.block.time)
            .with_balance(reward_balance)
            .user(self.api.canonical_address(&self.env.message.sender)?)
            .claim_reward()?;

        // Add the reward to the response
        let reward_token_link = load_reward_token(&self.storage, &self.api)?;
        let reward_token      = ISnip20::attach(&reward_token_link);
        response.messages.push(reward_token.transfer(&self.env.message.sender, reward)?);
        response.log.push(LogAttribute { key: "reward".into(), value: reward.into() });

        Ok(response)
    }

}

messages!(Query {
    Admin {}
    PoolInfo { at: Time }
    /// Requires the user's viewing key.
    UserInfo { at: Time, address: HumanAddr, key: String }
    /// For Keplr integration
    TokenInfo {}
    /// For Keplr integration
    Balance {}
});

messages!(Response {

    /// Response from `Query::PoolInfo`
    PoolInfo {
        lp_token:         ContractLink<HumanAddr>,
        reward_token:     ContractLink<HumanAddr>,

        it_is_now:        Time,

        pool_last_update: Time,
        pool_lifetime:    Volume,
        pool_locked:      Amount,

        #[cfg(feature="pool_closes")]
        pool_closed:      Option<String>,

        pool_balance:     Amount,
        pool_claimed:     Amount,

        #[cfg(feature="age_threshold")]
        pool_threshold:   Time,

        #[cfg(feature="claim_cooldown")]
        pool_cooldown:    Time,

        #[cfg(feature="pool_liquidity_ratio")]
        pool_liquid:      Amount
    }

    /// Response from `Query::UserInfo`
    UserInfo {
        it_is_now:        Time,

        pool_last_update: Time,
        pool_lifetime:    Volume,
        pool_locked:      Amount,

        #[cfg(feature="pool_closes")]
        pool_closed:      Option<String>,

        user_last_update: Option<Time>,
        user_lifetime:    Volume,
        user_locked:      Amount,
        user_share:       Amount,
        user_earned:      Amount,
        user_claimed:     Amount,
        user_claimable:   Amount,

        #[cfg(feature="age_threshold")]
        user_age:         Time,

        #[cfg(feature="claim_cooldown")]
        user_cooldown:    Time
    }

    Admin {
        address: HumanAddr
    }

    /// Keplr integration
    TokenInfo {
        name:         String,
        symbol:       String,
        decimals:     u8,
        total_supply: Option<Amount>
    }

    /// Keplr integration
    Balance {
        amount: Amount
    }

});

impl <S: Storage, A: Api, Q: Querier> Contract<S, A, Q> {

    pub fn query (
        self,
        msg: Query
    ) -> StdResult<QueryResponse> {
        Err(StdError::generic_err("not implemented"))
    }

    #[query] pub fn admin (
        self
    ) -> StdResult<QueryResponse> {
        Ok(Response::Admin { address: load_admin(self)? })
    }

    #[query] pub fn pool_info (
        self,
        at: Time
    ) -> StdResult<QueryResponse> {

        let pool = Pool::new(self.storage)
            .at(at)
            .with_balance(self.load_reward_balance()?);

        let pool_last_update = pool.timestamp()?;

        if at < pool_last_update {
            return Err(StdError::generic_err("this contract does not store history")) }

        #[cfg(feature="pool_closes")]
        let pool_closed =
            if let Some((_, close_message)) = Pool::new(self.storage).closed()? {
                Some(close_message) }
            else {
                None };

        Ok(Response::PoolInfo {
            it_is_now: at,

            lp_token:     load_lp_token(self.storage, self.api)?,
            reward_token: load_reward_token(self.storage, self.api)?,

            #[cfg(feature="pool_closes")]
            pool_closed,

            pool_last_update,
            pool_lifetime:  pool.lifetime()?,
            pool_locked:    pool.locked()?,
            pool_claimed:   pool.claimed()?,
            pool_balance:   pool.balance(),

            #[cfg(feature="age_threshold")]
            pool_threshold: pool.threshold()?,

            #[cfg(feature="claim_cooldown")]
            pool_cooldown:  pool.cooldown()?,

            #[cfg(feature="pool_liquidity_ratio")]
            pool_liquid:    pool.liquidity_ratio()?,

            /* todo add balance/claimed/total in rewards token */
        })
    }

    #[query] pub fn user_info (
        self,
        at:      Time,
        address: HumanAddr,
        key:     String
    ) -> StdResult<QueryResponse> {
        let address = self.api.canonical_address(&address)?;

        authenticate(&self.storage, &ViewingKey(key), address.as_slice())?;

        let pool = Pool::new(&self.storage).at(at);
        let pool_last_update = pool.timestamp()?;
        if at < pool_last_update {
            return Err(StdError::generic_err("no time travel")) }
        let pool_lifetime = pool.lifetime()?;
        let pool_locked   = pool.locked()?;

        let reward_balance = self.load_reward_balance()?;
        let user = pool.with_balance(reward_balance).user(address);
        let user_last_update = user.timestamp()?;
        if let Some(user_last_update) = user_last_update {
            if at < user_last_update {
                return Err(StdError::generic_err("no time travel")) } }

        #[cfg(feature="pool_closes")]
        let pool_closed =
            if let Some((_, close_message)) = Pool::new(&self.storage).closed()? {
                Some(close_message) }
            else {
                None };

        Ok(Response::UserInfo {
            it_is_now: at,

            #[cfg(feature="pool_closes")]
            pool_closed,

            pool_last_update,
            pool_lifetime,
            pool_locked,

            user_last_update,
            user_lifetime:  user.lifetime()?,
            user_locked:    user.locked()?,
            user_share:     user.share(HUNDRED_PERCENT)?.low_u128().into(),
            user_earned:    user.earned()?,
            user_claimed:   user.claimed()?,
            user_claimable: user.claimable()?,

            #[cfg(feature="age_threshold")]
            user_age:       user.present()?,

            #[cfg(feature="claim_cooldown")]
            user_cooldown:  user.cooldown()?
        })
    }

    #[query] pub fn token_info (
        self
    ) -> StdResult<QueryResponse> {
        let lp_token      = load_lp_token(&self.storage, &self.api)?;
        let lp_token_info = ISnip20::attach(&lp_token).query(&self.querier).token_info()?;
        let lp_token_name = format!("Sienna Rewards: {}", lp_token_info.name);
        Ok(Response::TokenInfo {
            name:         lp_token_name,
            symbol:       "SRW".into(),
            decimals:     1,
            total_supply: None
        })
    }

    #[query] pub fn balance (
        self,
        address: HumanAddr,
        key:     String
    ) -> StdResult<QueryResponse> {
        let address = self.api.canonical_address(&address)?;
        authenticate(&self.storage, &ViewingKey(key), address.as_slice())?;
        Ok(Response::Balance {
            amount: Pool::new(&self.storage).user(address).locked()?
        })
    }

}

impl<S: Storage, A: Api, Q: Querier> Contract<S, A, Q> {

    pub fn load_reward_balance (self) -> StdResult<Uint128> {
        let reward_token_link  = load_reward_token(&self.storage, &self.api)?;
        let reward_token       = ISnip20::attach(&reward_token_link);
        let mut reward_balance = reward_token.query(&self.querier).balance(
            &load_self_reference(&self.storage, &self.api)?.address,
            &load_viewing_key(&self.storage)?.0)?;

        let lp_token_link = load_lp_token(&self.storage, &self.api)?;
        if lp_token_link == reward_token_link {
            let lp_balance = Pool::new(&self.storage).locked()?;
            reward_balance = (reward_balance - lp_balance)?; }

        Ok(reward_balance)
    }

    #[cfg(feature="pool_closes")]
    /// Returns either a "pool closed" HandleResponse
    /// (containing a LP Token transaction to return
    /// all of the user's locked LP the first time)
    /// or None if the pool isn't closed.
    pub fn close_handler (self) -> StdResult<Option<HandleResponse>> {
        Ok(if let Some((_, close_message)) = Pool::new(&*self.storage).closed()? {
            let mut messages = vec![];
            let mut log = vec![LogAttribute {
                key: "closed".into(), value: close_message }];
            let mut user = Pool::new(&mut *self.storage).at(self.env.block.time)
                .user(api.canonical_address(&self.env.message.sender)?);
            let locked = user.retrieve_tokens(
                user.locked()?)?;
            if locked > Amount::zero() {
                messages.push(
                    ISnip20::attach(&load_lp_token(&*self.storage, api)?)
                        .transfer(&self.env.message.sender, locked)?);
                log.push(LogAttribute {
                    key: "retrieved".into(), value: locked.into() });};
            Some(HandleResponse { messages, log, ..HandleResponse::default() })
        } else {
            None
        })
    }

}
