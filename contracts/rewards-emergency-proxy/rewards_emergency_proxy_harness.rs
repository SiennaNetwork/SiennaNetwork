#![allow(unreachable_patterns)]

/// Unit testing harness for Sienna Rewards Emergency Proxy.

use fadroma::scrt::cosmwasm_std::{
    Uint128, HumanAddr, StdResult, StdError,
    Extern, testing::MockApi, MemoryStorage,
    Querier, QueryRequest, Empty, WasmQuery, QuerierResult,
    from_binary, to_binary, from_slice, SystemError};
use fadroma::scrt::callback::{
    ContractInstance as ContractLink};
use fadroma::scrt::harness::{
    Harness, InitFn, HandleFn, QueryFn, TxResult, assert_error};
use fadroma::scrt::snip20_api::mock::*;

use crate::{init, handle, query};
use crate::msg::{Init, Handle as TX, Query as QQ, Response};

pub use crate::rewards_math::{Amount, Time, Volume};
pub use fadroma::scrt::snip20_api::mock::Snip20;
pub use fadroma::scrt::harness::assert_fields;

pub struct RewardsEmergencyProxyMockQuerier {
    pub balance: Uint128
}

impl RewardsEmergencyProxyMockQuerier {
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

impl Querier for RewardsEmergencyProxyMockQuerier {
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

pub struct RewardsEmergencyProxyHarness <Q: Querier> {
    _now:  u128,
    _deps: Extern<MemoryStorage, MockApi, Q>,
    _lp_token:     ContractLink<HumanAddr>,
    _reward_token: ContractLink<HumanAddr>
}

impl <Q: Querier> Harness <Q, Init, TX, QQ, Response>
for RewardsEmergencyProxyHarness<Q> {
    type Deps = Extern<MemoryStorage, MockApi, Q>;
    fn deps       (&self)     -> &Self::Deps { &self._deps }
    fn deps_mut   (&mut self) -> &mut Self::Deps { &mut self._deps }
    fn get_init   (&mut self) -> InitFn<Self::Deps, Init> { init }
    fn get_handle (&mut self) -> HandleFn<Self::Deps, TX> { handle }
    fn get_query  (&self)     -> QueryFn<Self::Deps, QQ>  { query }
}

/// See https://docs.rs/cosmwasm-std/0.10.1/cosmwasm_std/testing/fn.mock_dependencies_with_balances.html
const ADDR_LEN: usize = 45;

impl RewardsEmergencyProxyHarness<RewardsMockQuerier> {

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

    pub fn init(&mut self, admin: &HumanAddr) -> StdResult<&mut Self> {
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
        Ok(self)
    }

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
            Ok(&mut *self)
        } else {
            unreachable!()
        }
    }

    pub fn claim (
        &mut self, user: &HumanAddr, amount: u128
    ) -> StdResult<&mut Self> {
        assert_eq!(self.tx_claim(user)?, (vec![
            Snip20::transfer(user.as_str(), &format!("{}", &amount)) ], 0, 0));
        self._deps.querier.decrement_balance(amount)?;
        Ok(self)
    }

    // private query and transaction helpers ------------------------------------------------------

    fn q_user_info (&self, address: &HumanAddr) -> StdResult<Response> {
        self.q(QQ::UserInfo {
            at: self.now(),
            address: address.clone(),
            key: "".into() })
    }

    fn tx_claim (&mut self, agent: &HumanAddr) -> TxResult {
        self.tx(self.now(), agent, TX::Claim {})
    }
}

