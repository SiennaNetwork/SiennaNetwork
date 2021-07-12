use std::str::from_utf8;
use fadroma::scrt::{
    cosmwasm_std::{
        Uint128, HumanAddr, StdResult,
        InitResponse, HandleResponse,
        Env, BlockInfo, MessageInfo, ContractInfo,
        Extern, MemoryStorage, Querier, QuerierResult, testing::MockApi,
        from_binary, CosmosMsg, WasmMsg,
        SystemError, from_slice, Empty, QueryRequest, WasmQuery, to_binary,
    },
    callback::{ContractInstance as ContractLink},
    //snip20 // todo work around circular dep ( via more reexports :( )
};
use sienna_rewards_benchmark::{
    init, handle, query,
    msg::{Init, Handle as TX, Query as QQ, Response}
};

#[macro_export] macro_rules! assert_error {
    ($response:expr, $msg:expr) => { assert_eq!($response, Err(StdError::generic_err($msg))) }
}

/// Successful transaction return a vector of relevant messages and a count of any others
type TxResult = StdResult<(Vec<String>, usize, usize)>;

// CosmosMsg::Wasm(WasmMsg::Execute { msg: the_actual_message_as_binary })
// loses type information. Thinking of genericizing it (hard, would require platform changes,
// would also enable custom non-JSON serializers) and/or adding a response builder macro
// (add_response_message!, add_response_log!, set_response_data!).

/// Reusable test harness with overridable post processing
/// for init and tx response messages.
pub trait Harness <Q: Querier> {

    fn deps     (&self)     -> &Extern<MemoryStorage, MockApi, Q>;
    fn deps_mut (&mut self) -> &mut Extern<MemoryStorage, MockApi, Q>;

    fn init (&mut self, height: u64, agent: &HumanAddr, msg: Init) -> TxResult {
        init(self.deps_mut(), Env {
            block:    BlockInfo    { height, time: height * 5, chain_id: "secret".into() },
            message:  MessageInfo  { sender: agent.into(), sent_funds: vec![] },
            contract: ContractInfo { address: "contract_addr".into() },
            contract_key:       Some("contract_key".into()),
            contract_code_hash: "contract_hash".into()
        }, msg).map(|result|Self::postprocess_init(result))?
    }

    fn postprocess_init (result: InitResponse) -> TxResult {
        let mut relevant = vec![];
        let mut other    = 0;
        let mut invalid  = 0;
        for cosmos_msg in result.messages.iter() {
            match cosmos_msg {
                CosmosMsg::Wasm(wasm_msg) => match wasm_msg {
                    WasmMsg::Execute { msg, .. } => match from_utf8(msg.as_slice()) {
                        Ok(msg) => relevant.push(msg.trim().into()),
                        Err(_) => invalid += 1,
                    },
                    _ => other += 1
                },
                _ => other += 1
            }
        }
        Ok((relevant, other, invalid))
    }

    fn tx (&mut self, height: u64, agent: &HumanAddr, tx: TX) -> TxResult {
        handle(self.deps_mut(), Env {
            block:    BlockInfo    { height, time: height * 5, chain_id: "secret".into() },
            message:  MessageInfo  { sender: agent.into(), sent_funds: vec![] },
            contract: ContractInfo { address: "contract_addr".into() },
            contract_key:       Some("contract_key".into()),
            contract_code_hash: "contract_hash".into()
        }, tx).map(|result|Self::postprocess_tx(result))?
    }

    fn postprocess_tx (result: HandleResponse) -> TxResult {
        let mut relevant = vec![];
        let mut other    = 0;
        let mut invalid  = 0;
        for cosmos_msg in result.messages.iter() {
            match cosmos_msg {
                CosmosMsg::Wasm(wasm_msg) => match wasm_msg {
                    WasmMsg::Execute { msg, .. } => match from_utf8(msg.as_slice()) {
                        Ok(msg) => relevant.push(msg.trim().into()),
                        Err(_)  => invalid += 1,
                    },
                    _ => other += 1
                },
                _ => other += 1
            }
        }
        Ok((relevant, other, invalid))
    }

    fn q (&self, q: QQ) -> StdResult<Response> {
        match query(self.deps(), q) {
            Ok(response) => from_binary(&response),
            Err(e) => Err(e)
        }
    }
}

pub struct RewardsHarness <Q: Querier> {
    _deps: Extern<MemoryStorage, MockApi, Q>,
    _lp_token: ContractLink<HumanAddr>
}

// trait fields WHEN?
impl <Q: Querier> Harness <Q> for RewardsHarness<Q> {
    fn deps (&self) -> &Extern<MemoryStorage, MockApi, Q> {
        &self._deps
    }
    fn deps_mut (&mut self) -> &mut Extern<MemoryStorage, MockApi, Q> {
        &mut self._deps
    }
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
    pub fn q_user_info (&self, now: u64, address: HumanAddr) -> StdResult<Response> {
        self.q(QQ::UserInfo { now, address, key: "".into() })
    }

    pub fn fund (mut self, amount: Uint128) -> Self {
        Self {
            _lp_token: self._lp_token,
            _deps: self._deps.change_querier(|q|RewardsMockQuerier { balance: q.balance + amount })
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
