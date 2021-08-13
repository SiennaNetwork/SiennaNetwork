#![allow(unreachable_patterns)]

/// Unit testing harness for Sienna Rewards.

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

pub use crate::{DAY, rewards_math::{Amount, Time, Volume}};
pub use fadroma::scrt::snip20_api::mock::Snip20;
pub use fadroma::scrt::harness::assert_fields;

// CosmosMsg::Wasm(WasmMsg::Execute { msg: the_actual_message_as_binary })
// loses type information. Thinking of genericizing it (hard, would require platform changes,
// would also enable custom non-JSON serializers) and/or adding a response builder macro
// (add_response_message!, add_response_log!, set_response_data!).

pub struct RewardsHarness <Q: Querier> {
    now: Time,
    _deps: Extern<MemoryStorage, MockApi, Q>,
    _lp_token: ContractLink<HumanAddr>
}

// trait fields WHEN?
impl <Q: Querier> Harness <Q, Init, TX, QQ, Response> for RewardsHarness<Q> {
    type Deps = Extern<MemoryStorage, MockApi, Q>;
    fn deps       (&self)     -> &Self::Deps { &self._deps }
    fn deps_mut   (&mut self) -> &mut Self::Deps { &mut self._deps }
    fn get_init   (&mut self) -> InitFn<Self::Deps, Init> { init }
    fn get_handle (&mut self) -> HandleFn<Self::Deps, TX> { handle }
    fn get_query  (&self)     -> QueryFn<Self::Deps, QQ>  { query }
}

/// See https://docs.rs/cosmwasm-std/0.10.1/cosmwasm_std/testing/fn.mock_dependencies_with_balances.html
const ADDR_LEN: usize = 45;

impl RewardsHarness<RewardsMockQuerier> {
    pub fn new () -> Self {
        Self {
            now: 0,
            _deps: Extern {
                storage: MemoryStorage::default(),
                api:     MockApi::new(ADDR_LEN),
                querier: RewardsMockQuerier { balance: 0u128.into() }
            },
            _lp_token: ContractLink {
                address:   "lp_token_address".into(),
                code_hash: "lp_token_hash".into(),
            }
        }
    }

    pub fn at (mut self, t: Time) -> Self {
        self.now = t;
        self
    }

    pub fn init_configured (mut self, admin: &HumanAddr) -> StdResult<Self> {
        let result = self.init(self.now, admin, Init {
            admin: None,
            lp_token:     Some(self.lp_token()),
            reward_token: ContractLink {
                address:   "reward_token_address".into(),
                code_hash: "reward_token_hash".into(),
            },
            viewing_key:  "".into(),
            ratio:        None,
            threshold:    None,
            cooldown:     None
        })?;
        assert_eq!(result,
            (vec![Snip20::set_viewing_key("")], 0, 0));
        Ok(self)
    }

    pub fn init_partial (mut self, admin: &HumanAddr) -> StdResult<Self> {
        let result = self.init(self.now, admin, Init {
            admin: None,
            lp_token:     None,
            reward_token: ContractLink {
                address:    "reward_token_address".into(),
                code_hash:  "reward_token_hash".into(),
            },
            viewing_key:  "".into(),
            ratio:        None,
            threshold:    None,
            cooldown:     None
        })?;
        assert_error!(self.q(QQ::PoolInfo { at: self.now })?,
            "missing liquidity provision token");
        assert_error!(self.q(QQ::UserInfo { at: self.now, address: admin.clone(), key: "".into() }),
            "missing liquidity provision token");
        Ok(self)
    }

    pub fn set_token_fails (
        self, badman: &HumanAddr, bad_addr: &str, bad_hash: &str
    ) -> StdResult<Self> {
        assert_eq!(self.tx_set_token(self.now, badman, bad_addr, bad_hash),
            Err(StdError::unauthorized()));
        assert_error!(self.q(QQ::PoolInfo { at: self.now })?,
            "missing liquidity provision token");
        Ok(self)
    }

    pub fn lp_token (&self) -> ContractLink<HumanAddr> {
        self._lp_token.clone()
    }

    pub fn tx_set_token (
        &mut self, height: Time, agent: &HumanAddr,
        address: &str, code_hash: &str
    ) -> TxResult {
        self.tx(height, agent, TX::SetProvidedToken {
            address:   HumanAddr::from(address),
            code_hash: code_hash.into(),
        })
    }

    pub fn pool (
        self, locked: u128, lifetime: u128, last_update: Time
    ) -> StdResult<Self> {
        if let Response::PoolInfo {
            pool_locked, pool_lifetime, pool_last_update, ..
        } = self.q(QQ::PoolInfo { at: self.now })? {
            assert_eq!(Amount::from(locked),   pool_locked);
            assert_eq!(Volume::from(lifetime), pool_lifetime);
            assert_eq!(last_update,            pool_last_update);
            Ok(self)
        } else {
            unreachable!()
        }
    }

    pub fn user (
        mut self, user: &HumanAddr,
        age: u64, locked: u128, lifetime: u128,
        earned: u128, claimed: u128, claimable: u128
    ) -> StdResult<Self> {
        if let Response::UserInfo {
            user_age, user_locked, user_lifetime,
            user_earned, user_claimed, user_claimable, ..
        } = self.q(QQ::UserInfo { at: self.now, address: user.clone(), key: "".into() })? {
            assert_eq!(age,   user_age);
            assert_eq!(Amount::from(locked),    user_locked);
            assert_eq!(Volume::from(lifetime),  user_lifetime);
            assert_eq!(Amount::from(earned),    user_earned);
            assert_eq!(Amount::from(claimed),   user_claimed);
            assert_eq!(Amount::from(claimable), user_claimable);
            Ok(self)
        } else {
            unreachable!()
        }
    }

    pub fn set_vk (mut self, agent: &HumanAddr, key: &str) -> StdResult<Self> {
        self.tx_set_vk(self.now, agent, key)?;
        Ok(self)
    }
    pub fn tx_set_vk (&mut self, height: Time, agent: &HumanAddr, key: &str) -> TxResult {
        self.tx(height, agent, TX::SetViewingKey { key: key.into(), padding: None })
    }

    pub fn lock (
        mut self, user: &HumanAddr, amount: u128
    ) -> StdResult<Self> {
        assert_eq!(
            self.tx_lock(self.now, user, amount.into())?,
            (vec![ Snip20::transfer_from("admin", "contract_addr", "1") ], 0, 0)
        );
        Ok(self)
    }
    pub fn tx_lock (&mut self, height: Time, agent: &HumanAddr, amount: u128) -> TxResult {
        self.tx(height, agent, TX::Lock { amount: amount.into() })
    }

    pub fn retrieve (
        mut self, user: &HumanAddr, amount: u128
    ) -> StdResult<Self> {
        assert_eq!(
            self.tx_retrieve(self.now, user, amount.into())?,
            (vec![ Snip20::transfer("admin", "1") ], 0, 0)
        );
        Ok(self)
    }
    pub fn tx_retrieve (&mut self, height: Time, agent: &HumanAddr, amount: u128) -> TxResult {
        self.tx(height, agent, TX::Retrieve { amount: amount.into() })
    }

    pub fn tx_claim (&mut self, height: Time, agent: &HumanAddr) -> TxResult {
        self.tx(height, agent, TX::Claim {})
    }

    pub fn fund (mut self, amount: u128) -> Self {
        self._deps = self._deps.change_querier(|q|RewardsMockQuerier {
            balance: q.balance + amount.into()
        });
        self
    }
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

//#[macro_export] macro_rules! test {

    //// pool info before lp token address is configured --------------------------------------------
    //($T:ident = $now:expr ; pool_unprepared -> {
        //locked:   $locked:expr,
        //lifetime: $lifetime:expr,
        //updated:  $updated:expr
    //} ) => { assert_fields!(
        //$T.q_pool_info($now as Time)? ;
        //Response::PoolInfo {
            //it_is_now: $now as Time,
            //lp_token:  None,
            //pool_last_update: $updated as Time,
            //pool_lifetime:    Volume::from($lifetime as u128),
            //pool_locked:      Amount::from($locked  as u128) } ); };

    //// pool info when lp token is configured ------------------------------------------------------
    //($T:ident = $now:expr ; pool -> {
        //locked:   $locked:expr,
        //lifetime: $lifetime:expr,
        //updated:  $updated:expr
    //}) => { assert_fields!(
        //$T.q_pool_info($now as Time)? ;
        //Response::PoolInfo {
            //it_is_now: $now as Time,
            //lp_token:  $T.lp_token(),
            //pool_last_update: $updated as Time,
            //pool_lifetime:    Volume::from($lifetime as u128),
            //pool_locked:      Amount::from($locked   as u128) } ); };

    //// user info ----------------------------------------------------------------------------------
    //($T:ident = $now:expr ; user($who:expr) -> {
        //age:       $age:expr,
        //locked:    $locked:expr,
        //lifetime:  $lifetime:expr,
        //earned:    $earned:expr,
        //claimed:   $claimed:expr,
        //claimable: $claimable:expr
    //} ) => { assert_fields!(
        //$T.q_user_info($now as Time, &$who)? ;
        //Response::UserInfo {
            //it_is_now: $now as Time,
            //// ignore pool fields
            ////assert_eq!(user_last_update, $updated as Time);
            //user_age: $age as Time,
            //user_locked:    Amount::from($locked    as u128),
            //user_lifetime:  Volume::from($lifetime  as u128),
            //user_earned:    Amount::from($earned    as u128),
            //user_claimed:   Amount::from($claimed   as u128),
            //user_claimable: Amount::from($claimable as u128) } ); };

    //// user actions -------------------------------------------------------------------------------

    //($T:ident = $now:expr ; $who:ident locks $amount:literal -> [
        //$($msg:expr),*
    //]) => {
        //assert_eq!(
            //$T.tx_lock($now, &$who, ($amount as u128).into())?,
            //(vec![ $($msg,)* ], 0, 0)
        //)
    //};

    //($T:ident = $now:expr ; $who:ident retrieves $amount:literal -> [
        //$($msg:expr),*
    //]) => {
        //assert_eq!(
            //$T.tx_retrieve($now, &$who, ($amount as u128).into())?,
            //(vec![ $($msg,)* ], 0, 0)
        //)
    //};

    //($T:ident = $now:expr ; $who:ident claims -> [
        //$($msg:expr),*
    //]) => {
        //assert_eq!(
            //$T.tx_claim($now, &$who)?,
            //(vec![ $($msg,)* ], 0, 0)
        //)
    //};

//}
