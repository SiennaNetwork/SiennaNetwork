#![allow(unreachable_patterns)]

/// Unit testing harness for Sienna Rewards.

use fadroma::*;
use fadroma::testing::*;
use fadroma::scrt_contract_harness::*;
use crate::*;

macro_rules! assert_error {
    ($response:expr, $msg:expr) => { assert_eq!($response, Err(StdError::generic_err($msg))) }
}

macro_rules! assert_fields {
    ($instance:expr ; $variant:path {
        $($var:ident: $expected:expr),+
    }) => { {
        let mut tw = tabwriter::TabWriter::new(std::io::stdout());
        write!(&mut tw, "field\texpected\tactual\t\n");
        $(
            write!(&mut tw, "{}\t", stringify!($var));
            write!(&mut tw, "{:?}\t", $expected);
            write!(&mut tw, "{:?}\t\n", (if $var == $expected {
                yansi::Paint::green
            } else {
                yansi::Paint::red
            })(format!("{}", &$var)));
        )+;
    }; }
}

pub struct RewardsMockQuerier {
    pub balance: Uint128
}

impl RewardsMockQuerier {
    fn mock_query_dispatch (
        &self,
        _contract: &ContractLink<HumanAddr>,
        msg:       &Snip20Query
    ) -> Snip20QueryAnswer {
        match msg {
            Snip20Query::Balance { .. } => {
                //if contract != self.reward_token {
                    //panic!("MockSnip20Querier: Expected balance query for {:?}", self.reward_token)
                //}
                Snip20QueryAnswer::Balance { amount: self.balance }
            },

            _ => unimplemented!()
        }
    }
    pub fn increment_balance (&mut self, amount: u128) {
        self.balance = self.balance + amount.into();
    }
    pub fn decrement_balance (&mut self, amount: u128) -> StdResult<()> {
        self.balance = (self.balance - amount.into())?;
        Ok(())
    }
}

impl Querier for RewardsMockQuerier {
    fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = match from_slice(bin_request) {
            Ok(v) => v,
            Err(e) => {
                let error = format!("Parsing query request: {}", e);
                let request = bin_request.into();
                return Err(SystemError::InvalidRequest { error, request })
            }
        };
        match request {
            QueryRequest::Wasm(WasmQuery::Smart { callback_code_hash, contract_addr, msg }) => {
                Ok(to_binary(&self.mock_query_dispatch(&ContractLink {
                    code_hash: callback_code_hash,
                    address: contract_addr
                }, &from_binary(&msg).unwrap())))
            },
            _ => panic!("MockSnip20Querier: Expected WasmQuery::Smart.")
        }
    }
}

pub struct RewardsHarness <Q: Querier> {
    _now:  u128,
    _deps: Extern<MemoryStorage, MockApi, Q>,
    _lp_token:     ContractLink<HumanAddr>,
    _reward_token: ContractLink<HumanAddr>
}

// trait fields WHEN?
impl <Q: Querier> Harness <Q, Init, Handle, Query, Response> for RewardsHarness<Q> {
    type Deps = Extern<MemoryStorage, MockApi, Q>;
    fn deps       (&self)     -> &Self::Deps { &self._deps }
    fn deps_mut   (&mut self) -> &mut Self::Deps { &mut self._deps }
    fn get_init   (&mut self) -> InitFn<Self::Deps, Init> { crate::init }
    fn get_handle (&mut self) -> HandleFn<Self::Deps, Handle> { crate::handle }
    fn get_query  (&self)     -> QueryFn<Self::Deps, Query>  { crate::query }
}

/// See https://docs.rs/cosmwasm-std/0.10.1/cosmwasm_std/testing/fn.mock_dependencies_with_balances.html
const ADDR_LEN: usize = 45;

impl RewardsHarness<RewardsMockQuerier> {

    pub fn new () -> Self {
        Self {
            _now:  0,
            _deps: Extern {
                storage:   MemoryStorage::default(),
                api:       MockApi::new(ADDR_LEN),
                querier:   RewardsMockQuerier { balance: 0u128.into() } },
            _lp_token:     ContractLink {
                address:   "lp_token_address".into(),
                code_hash: "lp_token_hash".into(), },
            _reward_token: ContractLink {
                address:   "reward_token_address".into(),
                code_hash: "reward_token_hash".into(), } } }

    // mocked external state ----------------------------------------------------------------------

    pub fn at (&mut self, t: u128) -> &mut Self {
        self._now = t;
        self }

    pub fn after (&mut self, t: u128) -> &mut Self {
        self._now += t;
        self }

    fn now (&self) -> u64 {
        self._now as u64 }

    pub fn fund <'a> (&'a mut self, amount: u128) -> &'a mut Self {
        self._deps.querier.increment_balance(amount);
        self }

    // init and provide LP token address ----------------------------------------------------------

    pub fn init_configured (&mut self, admin: &HumanAddr) -> StdResult<&mut Self> {
        let result = self.init(self.now(), admin, Init {
            admin:         None,
            lp_token:      Some(self.lp_token()),
            reward_token:  self.reward_token(),
            viewing_key:   "".into(),
            ratio:         None,
            threshold:     None,
            cooldown:      None })?;
        assert_eq!(result,
            (vec![Snip20::set_viewing_key("")], 0, 0));
        Ok(self) }

    pub fn init_partial (&mut self, admin: &HumanAddr) -> StdResult<&mut Self> {
        let result = self.init(self.now(), admin, Init {
            admin:         None,
            lp_token:      None,
            reward_token:  self.reward_token(),
            viewing_key:   "".into(),
            ratio:         None,
            threshold:     None,
            cooldown:      None })?;
        assert_eq!(result,
            (vec![Snip20::set_viewing_key("")], 0, 0));
        assert_error!(self.q_pool_info(),
            "missing liquidity provision token");
        Ok(self) }

    pub fn set_token (
        &mut self, admin: &HumanAddr, addr: &str, hash: &str
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_set_token(admin, addr, hash)?,
            (vec![], 0, 0));
        Ok(self) }

    pub fn set_token_fails (
        &mut self, badman: &HumanAddr, bad_addr: &str, bad_hash: &str
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_set_token(badman, bad_addr, bad_hash),
            Err(StdError::unauthorized()));
        assert_error!(self.q_pool_info(),
            "missing liquidity provision token");
        Ok(self) }

    pub fn set_ratio (
        &mut self, admin: &HumanAddr, num: u128, denom: u128
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_set_ratio(admin, num, denom)?,
            (vec![], 0, 0));
        Ok(self) }

    pub fn lp_token (&self) -> ContractLink<HumanAddr> {
        self._lp_token.clone() }

    pub fn reward_token (&self) -> ContractLink<HumanAddr> {
        self._reward_token.clone() }

    // pool and user status -----------------------------------------------------------------------

    pub fn pool (
        &mut self, locked: u128, lifetime: u128, last_update: u128
    ) -> StdResult<&mut Self> {
        if let Response::PoolInfo {
            pool_locked, pool_lifetime, pool_last_update, ..
        } = self.q_pool_info()? {
            assert_eq!(Volume::from(lifetime), pool_lifetime, "lifetime");
            assert_eq!(Amount::from(locked),   pool_locked,   "locked");
            assert_eq!(last_update as u64, pool_last_update,  "last_update");
            Ok(&mut *self) }
        else {
            unreachable!() } }

    pub fn user (
        &mut self, user: &HumanAddr,
        age: u128, locked: u128, lifetime: u128,
        earned: u128, claimed: u128, claimable: u128
    ) -> StdResult<&mut Self> {
        if let Response::UserInfo {
            user_age, user_locked, user_lifetime,
            user_earned, user_claimed, user_claimable, ..
        } = self.q_user_info(user)? {
            assert_eq!(Amount::from(claimable), user_claimable, "claimable");
            assert_eq!(Amount::from(claimed),   user_claimed,   "claimed");
            assert_eq!(Amount::from(earned),    user_earned,    "earned");
            assert_eq!(Volume::from(lifetime),  user_lifetime,  "lifetime");
            assert_eq!(Amount::from(locked),    user_locked,    "locked");
            assert_eq!(age as u64,              user_age,       "age");
            Ok(&mut *self) }
        else {
            unreachable!() } }

    // user actions -------------------------------------------------------------------------------

    pub fn set_vk (
        &mut self, agent: &HumanAddr, key: &str
    ) -> StdResult<&mut Self> {
        self.tx_set_vk(agent, key)?;
        Ok(self) }

    pub fn lock (
        &mut self, user: &HumanAddr, amount: u128
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_lock(user, amount.into())?, (vec![
            Snip20::transfer_from(user.as_str(), "contract_addr", &format!("{}", &amount))], 0, 0));
        Ok(self) }

    pub fn retrieve (
        &mut self, user: &HumanAddr, amount: u128
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_retrieve(user, amount.into())?, (vec![
            Snip20::transfer(user.as_str(), &format!("{}", &amount))], 0, 0));
        Ok(self) }

    pub fn retrieve_too_much (
        &mut self, user: &HumanAddr, amount: u128, msg: &str
    ) -> StdResult<&mut Self> {
        assert_eq!(
            self.tx_retrieve(user, amount.into()),
            Err(StdError::generic_err(msg)));
        Ok(self) }

    pub fn claim (
        &mut self, user: &HumanAddr, amount: u128
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_claim(user)?, (vec![
            Snip20::transfer(user.as_str(), &format!("{}", &amount)) ], 0, 0));
        self._deps.querier.decrement_balance(amount)?;
        Ok(self) }

    pub fn claim_must_wait (
        &mut self, user: &HumanAddr, expected_msg: &str
    ) -> StdResult<&mut Self> {
        match self.tx_claim(user) {
            Err(StdError::GenericErr { msg, .. }) => assert_eq!(msg, expected_msg),
            x => panic!("expected error, got: {:#?}", &x)
        }
        Ok(self) }

    pub fn close (
        &mut self, user: &HumanAddr  
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_close(user)?, (vec![], 0, 0));
        Ok(self) }

    // private query and transaction helpers ------------------------------------------------------

    fn q_pool_info (&self) -> StdResult<Response> {
        self.q(Query::PoolInfo {
            at: self.now() }) }

    fn q_user_info (&self, address: &HumanAddr) -> StdResult<Response> {
        self.q(Query::UserInfo {
            at: self.now(),
            address: address.clone(),
            key: "".into() }) }

    fn tx_set_token (
        &mut self, agent: &HumanAddr, address: &str, code_hash: &str
    ) -> TxResult {
        self.tx(self.now(), agent, Handle::SetProvidedToken {
            address:   HumanAddr::from(address),
            code_hash: code_hash.into(), }) }

    fn tx_set_ratio (
        &mut self, agent: &HumanAddr, num: u128, denom: u128
    ) -> TxResult {
        self.tx(self.now(), agent, Handle::ChangeRatio {
            numerator:   num.into(),
            denominator: denom.into(), }) }

    fn tx_set_vk (&mut self, agent: &HumanAddr, key: &str) -> TxResult {
        self.tx(self.now(), agent, Handle::SetViewingKey {
            key: key.into(),
            padding: None }) }

    pub fn tx_lock (&mut self, agent: &HumanAddr, amount: u128) -> TxResult {
        self.tx(self.now(), agent, Handle::Lock {
            amount: amount.into() }) }

    fn tx_retrieve (&mut self, agent: &HumanAddr, amount: u128) -> TxResult {
        self.tx(self.now(), agent, Handle::Retrieve {
            amount: amount.into() }) }

    fn tx_claim (&mut self, agent: &HumanAddr) -> TxResult {
        self.tx(self.now(), agent, Handle::Claim {}) }

    fn tx_close (&mut self, agent: &HumanAddr) -> TxResult {
        self.tx(self.now(), agent, Handle::ClosePool { message: "closed".into() }) }
}

pub struct Snip20;

impl Snip20 {
    pub fn set_viewing_key (key: &str) -> String {
        format!(
            "{{\"set_viewing_key\":{{\"key\":\"{}\",\"padding\":null}}}}",
            key
        ).into()
    }
    pub fn transfer_from (owner: &str, recipient: &str, amount: &str) -> String {
        format!(
            "{{\"transfer_from\":{{\"owner\":\"{}\",\"recipient\":\"{}\",\"amount\":\"{}\",\"padding\":null}}}}",
            owner, recipient, amount
        ).into()
    }
    pub fn transfer (recipient: &str, amount: &str) -> String {
        format!(
            "{{\"transfer\":{{\"recipient\":\"{}\",\"amount\":\"{}\",\"padding\":null}}}}",
            recipient, amount
        ).into()
    }
}

#[derive(serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Snip20Query {
    Balance {}
}

#[derive(serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Snip20QueryAnswer {
    Balance { amount: Uint128 }
}
