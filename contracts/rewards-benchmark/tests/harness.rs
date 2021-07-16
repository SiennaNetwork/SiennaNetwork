#![allow(unreachable_patterns)]

use fadroma::scrt::{
    cosmwasm_std::{
        Uint128, HumanAddr, StdResult,
        Extern, testing::MockApi, MemoryStorage,
        Querier, QueryRequest, Empty, WasmQuery, QuerierResult,
        from_binary, to_binary, from_slice, SystemError,
    },
    callback::{ContractInstance as ContractLink},
    harness::{Harness, InitFn, HandleFn, QueryFn},
    //snip20 // todo work around circular dep ( via more reexports :( )
};
pub use fadroma::scrt::harness::TxResult;

use sienna_rewards_benchmark::{
    init, handle, query,
    msg::{Init, Handle as TX, Query as QQ, Response}
};

#[macro_export] macro_rules! assert_error {
    ($response:expr, $msg:expr) => { assert_eq!($response, Err(StdError::generic_err($msg))) }
}

// CosmosMsg::Wasm(WasmMsg::Execute { msg: the_actual_message_as_binary })
// loses type information. Thinking of genericizing it (hard, would require platform changes,
// would also enable custom non-JSON serializers) and/or adding a response builder macro
// (add_response_message!, add_response_log!, set_response_data!).

pub struct RewardsHarness <Q: Querier> {
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

    pub fn lp_token (&self) -> ContractLink<HumanAddr> {
        self._lp_token.clone()
    }

    pub fn init_configured (&mut self, height: u64, agent: &HumanAddr) -> TxResult {
        self.init(height, agent, Init {
            admin: None,
            lp_token:     Some(self.lp_token()),
            reward_token: ContractLink {
                address:   "reward_token_address".into(),
                code_hash: "reward_token_hash".into(),
            },
            viewing_key:  "".into(),
            ratio:        None,
            threshold:    None
        })
    }
    pub fn init_partial (&mut self, height: u64, agent: &HumanAddr) -> TxResult {
        self.init(height, agent, Init {
            admin: None,
            lp_token:     None,
            reward_token: ContractLink {
                address:    "reward_token_address".into(),
                code_hash:  "reward_token_hash".into(),
            },
            viewing_key:  "".into(),
            ratio:        None,
            threshold:    None
        })
    }

    pub fn tx_set_token (
        &mut self, height: u64, agent: &HumanAddr,
        address: &str, code_hash: &str
    ) -> TxResult {
        self.tx(height, agent, TX::SetProvidedToken {
            address:   HumanAddr::from(address),
            code_hash: code_hash.into(),
        })
    }
    pub fn tx_set_vk (&mut self, height: u64, agent: &HumanAddr, key: &str) -> TxResult {
        self.tx(height, agent, TX::SetViewingKey { key: key.into(), padding: None })
    }
    pub fn tx_lock (&mut self, height: u64, agent: &HumanAddr, amount: u128) -> TxResult {
        self.tx(height, agent, TX::Lock { amount: amount.into() })
    }
    pub fn tx_retrieve (&mut self, height: u64, agent: &HumanAddr, amount: u128) -> TxResult {
        self.tx(height, agent, TX::Retrieve { amount: amount.into() })
    }
    pub fn tx_claim (&mut self, height: u64, agent: &HumanAddr) -> TxResult {
        self.tx(height, agent, TX::Claim {})
    }

    pub fn q_pool_info (&self, now: u64) -> StdResult<Response> {
        self.q(QQ::PoolInfo { now })
    }
    pub fn q_user_info (&self, now: u64, address: &HumanAddr) -> StdResult<Response> {
        self.q(QQ::UserInfo { now, address: address.clone(), key: "".into() })
    }

    pub fn fund (self, amount: u128) -> Self {
        Self {
            _lp_token: self._lp_token,
            _deps: self._deps
                .change_querier(|q|RewardsMockQuerier {
                    balance: q.balance + amount.into()
                })
        }
    }
}

pub struct RewardsMockQuerier {
    pub balance: Uint128
}

#[derive(serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
enum Snip20Query {
    Balance {}
}

#[derive(serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
enum Snip20QueryAnswer {
    Balance { amount: Uint128 }
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

#[macro_export] macro_rules! test {

    ($T:ident = $now:expr ; pool_unprepared -> {
        balance: $balance:expr, lifetime: $lifetime:expr, updated: $updated:expr
    }) => {
        assert_eq!($T.q_pool_info($now as u64)?, Response::PoolInfo {
            lp_token: None,
            balance:  Amount::from($balance  as u128),
            lifetime: Volume::zero($lifetime as u128),
            updated:  $updated as u64,
            now:      $now     as u64,
        });
    };

    ($T:ident = $now:expr ; pool -> {
        balance: $balance:expr, lifetime: $lifetime:expr, updated: $updated:expr
    }) => {
        assert_eq!($T.q_pool_info($now as u64)?, Response::PoolInfo {
            lp_token: $T.lp_token(),
            balance:  Amount::from($balance  as u128),
            lifetime: Volume::from($lifetime as u128),
            updated:  $updated as u64,
            now:      $now     as u64,
        });
    };

    ($T:ident = $now:expr ; user($who:expr) -> {
        age: $age:expr, balance: $balance:expr, lifetime: $lifetime:expr,
        unlocked: $unlocked:expr, claimed: $claimed:expr, claimable: $claimable:expr
    }) => {
        assert_eq!($T.q_user_info($now as u64, &$who)?, Response::UserInfo {
            age:       $age as u64,
            balance:   Amount::from($balance   as u128),
            lifetime:  Volume::from($lifetime  as u128),
            unlocked:  Amount::from($unlocked  as u128),
            claimed:   Amount::from($claimed   as u128),
            claimable: Amount::from($claimable as u128)
        });
    };

    ($T:ident = $now:expr ; lock($who:expr, $amount:expr) -> [ $($msg:expr),* ]) => {
        assert_eq!($T.tx_lock($now, &$who, ($amount as u128).into())?, (vec![ $($msg,)* ], 0, 0))
    };

    ($T:ident = $now:expr ; retr($who:expr, $amount:expr) -> [ $($msg:expr),* ]) => {
        assert_eq!($T.tx_retrieve($now, &$who, ($amount as u128).into())?, (vec![ $($msg,)* ], 0, 0))
    };

    ($T:ident = $now:expr ; claim($who:expr) -> [ $($msg:expr),* ]) => {
        assert_eq!($T.tx_claim($now, &$who)?, (vec![ $($msg,)* ], 0, 0))
    };

}
