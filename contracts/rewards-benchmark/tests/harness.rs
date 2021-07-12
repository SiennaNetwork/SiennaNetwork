use std::str::{from_utf8, Utf8Error};
use fadroma::scrt::{
    cosmwasm_std::{
        HumanAddr, StdResult, StdError,
        InitResponse, HandleResponse, Binary,
        Env, BlockInfo, MessageInfo, ContractInfo,
        Extern, MemoryStorage, Api, Querier,
        testing::{mock_dependencies, MockApi, MockQuerier},
        from_binary, CosmosMsg, WasmMsg,
    },
    callback::{ContractInstance as ContractLink}
};
use sienna_rewards_benchmark::{
    init, handle, query,
    msg::{Init, Handle as TX, Query as Q, Response}
};

#[macro_export] macro_rules! assert_error {
    ($response:expr, $msg:expr) => { assert_eq!($response, Err(StdError::generic_err($msg))) }
}

/// See https://docs.rs/cosmwasm-std/0.10.1/cosmwasm_std/testing/fn.mock_dependencies_with_balances.html
const ADDR_LEN: usize = 45;

/// Successful transaction return a vector of relevant messages and a count of any others
type TxResult = StdResult<(Vec<String>, usize, usize)>;

// CosmosMsg::Wasm(WasmMsg::Execute { msg: the_actual_message_as_binary })
// loses type information. Thinking of genericizing it (hard, would require platform changes,
// would also enable custom non-JSON serializers) and/or adding a response builder macro
// (add_response_message!, add_response_log!, set_response_data!).

/// Reusable test harness with overridable post processing
/// for init and tx response messages.
pub trait Harness {

    fn deps     (&self)     -> &Extern<MemoryStorage, MockApi, MockQuerier>;
    fn deps_mut (&mut self) -> &mut Extern<MemoryStorage, MockApi, MockQuerier>;

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
                        Err(_) => invalid += 1,
                    },
                    _ => other += 1
                },
                _ => other += 1
            }
        }
        Ok((relevant, other, invalid))
    }

    fn q (&self, q: Q) -> StdResult<Response> {
        match query(self.deps(), q) {
            Ok(response) => from_binary(&response),
            Err(e) => Err(e)
        }
    }
}

pub struct RewardsHarness {
    _deps: Extern<MemoryStorage, MockApi, MockQuerier>
}

// trait fields WHEN?
impl Harness for RewardsHarness {
    fn deps (&self) -> &Extern<MemoryStorage, MockApi, MockQuerier> {
        &self._deps
    }
    fn deps_mut (&mut self) -> &mut Extern<MemoryStorage, MockApi, MockQuerier> {
        &mut self._deps
    }
}

impl RewardsHarness {
    pub fn new () -> Self {
        Self { _deps: mock_dependencies(ADDR_LEN, &[]) }
    }

    pub fn init_configured (&mut self, height: u64, agent: &HumanAddr) -> TxResult {
        self.init(height, agent, Init {
            provided_token: Some(ContractLink {
                address:   HumanAddr::from("provided_token_address"),
                code_hash: "provided_token_hash".into(),
            }),
            rewarded_token: ContractLink {
                address:   HumanAddr::from("rewarded_token_address"),
                code_hash: "rewarded_token_hash".into(),
            },
            viewing_key: "".into()
        })
    }
    pub fn init_partial (&mut self, height: u64, agent: &HumanAddr) -> TxResult {
        self.init(height, agent, Init {
            provided_token: None,
            rewarded_token: ContractLink {
                address:    HumanAddr::from("rewarded_token_address"),
                code_hash:  "rewarded_token_hash".into(),
            },
            viewing_key: "".into()
        })
    }
    pub fn q_status (&self, now: u64) -> StdResult<Response> {
        self.q(Q::Status { now })
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
}
