use std::{rc::Rc, cell::{RefCell, RefMut}};

use fadroma::scrt::{
    cosmwasm_std::*,
    BLOCK_SIZE,
    callback::ContractInstance as ContractLink,
    toolkit::snip20,
    snip20_api::ISnip20,
    addr::{Humanize, Canonize},
    storage::{load, save},
};

use crate::{
    rewards_api::*,
    rewards_math::*,
    rewards_errors::*,
    rewards_pool::Pool,
    rewards_admin::{
        DefaultHandleImpl as AdminHandle,
        admin_handle, AdminHandleMsg, load_admin,
        assert_admin, save_admin
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

const POOL_SELF_REFERENCE: &[u8] = b"self";

const POOL_LP_TOKEN: &[u8] = b"lp_token";

const POOL_REWARD_TOKEN: &[u8] = b"reward_token";

const POOL_REWARD_TOKEN_VK: &[u8] = b"reward_token_vk";

pub struct Contract <S, A, Q, E> {
    storage: Rc<RefCell<S>>,
    api:     A,
    querier: Q,
    env:     E
}

type InitResult   = StdResult<InitResponse>;
type HandleResult = StdResult<HandleResponse>;

impl <S: Storage, A: Api, Q: Querier> Contract<&mut S, &mut A, &mut Q, Env> {

    pub fn init (&mut self, msg: Init) -> InitResult {

        self.save_self_reference()?;

        save_admin(
            *self.storage.borrow_mut(),
            self.api,
            &msg.admin.unwrap_or(self.env.message.sender.clone())
        );

        // Contract accepts transactions in `lp_token`s.
        // The address of the `lp_token` can be provided later
        // to avoid a circular dependency during deployment.
        if let Some(lp_token) = msg.lp_token {
            self.save_lp_token(&lp_token)?;
        }

        // Contract distributes rewards in Reward Tokens.
        // For this, it must know its own balance in the `reward_token`s.
        // For that, it needs a reference to its own address+code_hash
        // and a viewing key in `reward_token`.

        self.save_reward_token(&msg.reward_token)?;

        let set_vk = ISnip20::attach(&msg.reward_token).set_viewing_key(&msg.viewing_key.0)?;
        self.save_viewing_key(&msg.viewing_key)?;

        // Reward pool has configurable parameters:
        let mut pool = Pool::new(self.storage);

        #[cfg(feature="pool_liquidity_ratio")]
        pool.set_created(&self.env.block.time)?;

        #[cfg(feature="global_ratio")]
        pool.configure_ratio(&msg.ratio.unwrap_or((1u128.into(), 1u128.into())))?;

        #[cfg(feature="age_threshold")]
        pool.configure_threshold(&msg.threshold.unwrap_or(DAY))?;

        #[cfg(feature="claim_cooldown")]
        pool.configure_cooldown(&msg.cooldown.unwrap_or(DAY))?;

        // TODO remove global state from scrt-contract
        // define field! and addr_field! macros instead -
        // problem here is identifier concatenation
        // and making each field a module is ugly
        Ok(InitResponse { messages: vec![set_vk], log: vec![] })
    }

    pub fn save_lp_token (
        &mut self,
        link: &ContractLink<HumanAddr>
    ) -> StdResult<()> {
        save(*self.storage.borrow_mut(), POOL_LP_TOKEN, &link.canonize(self.api)?)
    }

    pub fn save_reward_token (
        &mut self,
        link: &ContractLink<HumanAddr>
    ) -> StdResult<()> {
        save(*self.storage.borrow_mut(), POOL_REWARD_TOKEN, &link.canonize(self.api)?)
    }

    pub fn save_viewing_key (
        &mut self,
        key: &ViewingKey
    ) -> StdResult<()> {
        save(*self.storage.borrow_mut(), POOL_REWARD_TOKEN_VK, &key)
    }

    fn save_self_reference (
        &mut self,
    ) -> StdResult<()> {
        let link = ContractLink {
            address:   self.env.contract.address.clone(),
            code_hash: self.env.contract_code_hash.clone()
        };
        save(*self.storage.borrow_mut(), POOL_SELF_REFERENCE, &link.canonize(self.api)?)
    }

    pub fn handle (&mut self, msg: Handle) -> HandleResult {
        match msg {
            Handle::ChangeAdmin { address } =>
                self.change_admin(address),
            Handle::SetProvidedToken { address, code_hash } =>
                self.set_provided_token(address, code_hash),
            Handle::ChangeRatio { numerator, denominator } =>
                self.change_ratio(numerator, denominator),
            _ => Err(StdError::generic_err("not implemented"))
        }
    }

    /// Set the contract admin.
    pub fn change_admin (&mut self, address: HumanAddr) -> HandleResult {
        let msg = AdminHandleMsg::ChangeAdmin { address };
        admin_handle(
            *self.storage.borrow_mut(),
            self.api,
            &self.env,
            msg,
            AdminHandle
        )
    }

    fn assert_admin (&mut self) -> StdResult<()> {
        assert_admin(*self.storage.borrow(), self.api, &self.env)
    }

    /// Set the active asset token.
    // Resolves circular reference when initializing the benchmark -
    // they need to know each other's addresses to use initial allowances
    pub fn set_provided_token (&mut self, address: HumanAddr, code_hash: String) -> HandleResult {
        self.assert_admin()?;
        self.save_lp_token(&ContractLink { address, code_hash })?;
        tx_ok!()
    }

    #[cfg(feature="global_ratio")]
    pub fn change_ratio (&mut self, numerator: Amount, denominator: Amount) -> HandleResult {
        self.assert_admin()?;
        Pool::new(self.storage).configure_ratio(&(numerator.into(), denominator.into()))?;
        tx_ok!()
    }

    #[cfg(feature="age_threshold")]
    pub fn change_threshold (&mut self, threshold: Time) -> HandleResult {
        self.assert_admin()?;
        Pool::new(self.storage).configure_threshold(&threshold)?;
        tx_ok!()
    }

    #[cfg(feature="claim_cooldown")]
    pub fn change_cooldown (&mut self, cooldown: Time) -> HandleResult {
        self.assert_admin()?;
        Pool::new(self.storage).configure_cooldown(&cooldown)?;
        tx_ok!()
    }

    #[cfg(feature="pool_closes")]
    pub fn close_pool (&mut self, message: String) -> HandleResult {
        self.assert_admin()?;
        Pool::new(self.storage).at(self.env.block.time).close(message)?;
        tx_ok!()
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

        let reward_token = load_reward_token(self.storage, self.api)?;

        // Update the viewing key if the supplied
        // token info for is the reward token
        if reward_token == snip20 {
            self.save_viewing_key(&ViewingKey(key.clone()))?
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

    fn auth_handle (&mut self, msg: AuthHandleMsg) -> HandleResult {
        auth_handle(
            self.storage,
            self.api,
            &self.env,
            msg,
            AuthHandle
        )
    }

    /// User can request a new viewing key for oneself.
    pub fn create_viewing_key (&mut self, entropy: String, padding: Option<String>) -> HandleResult {
        self.auth_handle(AuthHandleMsg::CreateViewingKey { entropy, padding })
    }

    /// User can set own viewing key to a known value.
    pub fn set_viewing_key (&mut self, key: String, padding: Option<String>) -> HandleResult {
        self.auth_handle(AuthHandleMsg::SetViewingKey { key, padding })
    }

    /// User can lock some liquidity provision tokens.
    pub fn lock (&mut self, amount: Amount) -> HandleResult {
        // If the pool is closed, users can only retrieve all their liquidity tokens
        #[cfg(feature="pool_closes")]
        if let Some(closed_response) = self.close_handler()? {
            return Ok(closed_response)
        }

        tx_ok!(ISnip20::attach(&load_lp_token(self.storage, self.api)?).transfer_from(
            &self.env.message.sender,
            &self.env.contract.address,
            Pool::new(self.storage)
                .at(self.env.block.time)
                .user(self.api.canonical_address(&self.env.message.sender)?)
                .lock_tokens(amount)?)?)
    }

    /// User can always get their liquidity provision tokens back.
    pub fn retrieve (&mut self, amount: Amount) -> HandleResult {
        // If the pool is closed, users can only retrieve all their liquidity tokens
        #[cfg(feature="pool_closes")]
        if let Some(closed_response) = self.close_handler()? {
            return Ok(closed_response)
        }

        tx_ok!(ISnip20::attach(&load_lp_token(self.storage, self.api)?).transfer(
            &self.env.message.sender,
            Pool::new(self.storage)
                .at(self.env.block.time)
                .user(self.api.canonical_address(&self.env.message.sender)?)
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
        let reward_balance = load_reward_balance(self.storage, self.api, self.querier)?;

        // Compute the reward portion for this user.
        // May return error if portion is zero.
        let reward = Pool::new(self.storage)
            .at(self.env.block.time)
            .with_balance(reward_balance)
            .user(self.api.canonical_address(&self.env.message.sender)?)
            .claim_reward()?;

        // Add the reward to the response
        let reward_token_link = load_reward_token(self.storage, self.api)?;
        let reward_token      = ISnip20::attach(&reward_token_link);
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

        let pool = Pool::new(self.storage);

        Ok(if let Some((_, close_message)) = pool.closed()? {

            let mut messages = vec![];

            let mut log = vec![
                LogAttribute { key: "closed".into(), value: close_message }
            ];

            let mut user = pool.at(self.env.block.time).user(
                self.api.canonical_address(&self.env.message.sender)?
            );

            let locked = user.retrieve_tokens(user.locked()?)?;

            if locked > Amount::zero() {

                messages.push(
                    ISnip20::attach(&load_lp_token(self.storage, self.api)?).transfer(
                        &self.env.message.sender,
                        locked
                    )?
                );

                log.push(
                    LogAttribute { key: "retrieved".into(), value: locked.into() }
                );
            };

            Some(HandleResponse { messages, log, ..HandleResponse::default() })

        } else {
            None
        })
    }

}

impl <S: Storage, A: Api, Q: Querier> Contract<&S, &A, &Q, ()> {

    pub fn query (
        self,
        msg: Query
    ) -> StdResult<Response> {
        match msg {
            Query::Admin    {}     => self.admin(),
            Query::PoolInfo { at } => self.pool_info(at),
            _ => Err(StdError::generic_err("not implemented"))
        }
    }

    pub fn admin (
        self
    ) -> StdResult<Response> {
        Ok(Response::Admin {
            address: load_admin(self.storage, self.api)? 
        })
    }

    pub fn pool_info (
        self,
        at: Time
    ) -> StdResult<Response> {

        let pool = Pool::new(self.storage).at(at)
            .with_balance(load_reward_balance(self.storage, self.api, self.querier)?);

        let pool_last_update = pool.timestamp()?;

        if at < pool_last_update {
            return Err(StdError::generic_err("this contract does not store history")) }

        #[cfg(feature="pool_closes")]
        let pool_closed =
            if let Some((_, close_message)) = pool.closed()? {
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

    pub fn user_info (
        self,
        at:      Time,
        address: HumanAddr,
        key:     String
    ) -> StdResult<Response> {
        let address = self.api.canonical_address(&address)?;

        authenticate(self.storage, &ViewingKey(key), address.as_slice())?;

        let pool = Pool::new(self.storage).at(at);
        let pool_last_update = pool.timestamp()?;
        if at < pool_last_update {
            return Err(StdError::generic_err("no time travel")) }
        let pool_lifetime = pool.lifetime()?;
        let pool_locked   = pool.locked()?;

        let reward_balance = load_reward_balance(self.storage, self.api, self.querier)?;
        let user = pool.with_balance(reward_balance).user(address);
        let user_last_update = user.timestamp()?;
        if let Some(user_last_update) = user_last_update {
            if at < user_last_update {
                return Err(StdError::generic_err("no time travel")) } }

        #[cfg(feature="pool_closes")]
        let pool_closed =
            if let Some((_, close_message)) = Pool::new(self.storage).closed()? {
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

    pub fn token_info (
        self
    ) -> StdResult<Response> {
        let lp_token      = load_lp_token(self.storage, self.api)?;
        let lp_token_info = ISnip20::attach(&lp_token).query(self.querier).token_info()?;
        let lp_token_name = format!("Sienna Rewards: {}", lp_token_info.name);
        Ok(Response::TokenInfo {
            name:         lp_token_name,
            symbol:       "SRW".into(),
            decimals:     1,
            total_supply: None
        })
    }

    pub fn balance (
        self,
        address: HumanAddr,
        key:     String
    ) -> StdResult<Response> {
        let address = self.api.canonical_address(&address)?;
        authenticate(self.storage, &ViewingKey(key), address.as_slice())?;
        Ok(Response::Balance {
            amount: Pool::new(self.storage).user(address).locked()?
        })
    }

}

fn load_reward_token (
    storage: &impl Storage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_REWARD_TOKEN)?;
    result.ok_or(missing_reward_token())
}

fn load_lp_token (
    storage: &impl Storage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_LP_TOKEN)?;
    result.ok_or(missing_lp_token())
}

fn load_reward_balance <S: Storage> (
    storage: &S,
    api:     &impl Api,
    querier: &impl Querier
) -> StdResult<Uint128> {
    let reward_token_link  = load_reward_token(storage, api)?;
    let reward_token       = ISnip20::attach(&reward_token_link);
    let mut reward_balance = reward_token.query(querier).balance(
        &load_self_reference(storage, api)?.address,
        &load_viewing_key(storage)?.0)?;

    let lp_token_link = load_lp_token(storage, api)?;
    if lp_token_link == reward_token_link {
        let lp_balance = Pool::new(storage).locked()?;
        reward_balance = (reward_balance - lp_balance)?;
    }

    Ok(reward_balance)
}

pub fn load_self_reference (
    storage: &impl Storage,
    api:     &impl Api
) -> StdResult<ContractLink<HumanAddr>> {
    let result: Option<ContractLink<CanonicalAddr>> = load(storage, POOL_SELF_REFERENCE)?;
    match result {
        Some(link) => Ok(link.humanize(api)?),
        None => missing_self_reference()
    }
}

pub fn load_viewing_key (
    storage: &impl Storage
) -> StdResult<ViewingKey> {
    let result: Option<ViewingKey> = load(storage, POOL_REWARD_TOKEN_VK)?;
    match result {
        Some(key) => Ok(key),
        None => missing_viewing_key()
    }
}
