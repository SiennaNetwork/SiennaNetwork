use fadroma::scrt::{
    cosmwasm_std::{
        HumanAddr, StdResult,
        InitResponse, HandleResponse, Binary,
        Env, BlockInfo, MessageInfo, ContractInfo,
        Extern, MemoryStorage, Api, Querier,
        testing::{mock_dependencies, MockApi, MockQuerier},
    },
    callback::{ContractInstance as ContractLink}
};
use sienna_rewards_benchmark::{
    init, handle, query,
    msg::{Init, Handle as TX, Query as Q}
};

// See https://docs.rs/cosmwasm-std/0.10.1/cosmwasm_std/testing/fn.mock_dependencies_with_balances.html
const ADDR_LEN: usize = 45;

pub struct Harness {
    deps: Extern<MemoryStorage, MockApi, MockQuerier>
}

#[macro_export] macro_rules! assert_error {
    ($response:expr, $msg:expr) => {
        assert_eq!($response, Err(StdError::GenericErr {
            msg: $msg.into(),
            backtrace: None
        }))
    }
}

impl Harness {
    pub fn new () -> Self {
        Self { deps: mock_dependencies(ADDR_LEN, &[]) }
    }

    pub fn init (&mut self, height: u64, agent: &HumanAddr, msg: Init) -> StdResult<InitResponse> {
        init(&mut self.deps, Env {
            block:    BlockInfo    { height, time: height * 5, chain_id: "secret".into() },
            message:  MessageInfo  { sender: agent.into(), sent_funds: vec![] },
            contract: ContractInfo { address: "rewards_addr".into() },
            contract_key:       Some("rewards_key".into()),
            contract_code_hash: "rewards_hash".into()
        }, msg)
    }
    pub fn init_configured (&mut self, height: u64, agent: &HumanAddr) -> StdResult<InitResponse> {
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
    pub fn init_partial (&mut self, height: u64, agent: &HumanAddr) -> StdResult<InitResponse> {
        self.init(height, agent, Init {
            provided_token: None,
            rewarded_token: ContractLink {
                address:    HumanAddr::from("rewarded_token_address"),
                code_hash:  "rewarded_token_hash".into(),
            },
            viewing_key: "".into()
        })
    }

    pub fn q (&self, q: Q) -> StdResult<Binary> {
        query(&self.deps, q)
    }
    pub fn q_status (&self, now: u64) -> StdResult<Binary> {
        self.q(Q::Status { now })
    }

    pub fn tx (
        &mut self, height: u64, agent: &HumanAddr, tx: TX
    ) -> StdResult<HandleResponse> {
        handle(&mut self.deps, Env {
            block:    BlockInfo    { height, time: height * 5, chain_id: "secret".into() },
            message:  MessageInfo  { sender: agent.into(), sent_funds: vec![] },
            contract: ContractInfo { address: "rewards_addr".into() },
            contract_key:       Some("rewards_key".into()),
            contract_code_hash: "rewards_hash".into()
        }, tx)
    }

    pub fn tx_set_token (
        &mut self, height: u64, agent: &HumanAddr,
        address: &str, code_hash: &str
    ) -> StdResult<HandleResponse> {
        self.tx(height, agent, TX::SetProvidedToken {
            address:   HumanAddr::from(address),
            code_hash: code_hash.into(),
        })
    }
    pub fn tx_lock (
        &mut self, height: u64, agent: &HumanAddr, amount: u128
    ) -> StdResult<HandleResponse> {
        self.tx(height, agent, TX::Lock { amount: amount.into() })
    }
    pub fn tx_retrieve (
        &mut self, height: u64, agent: &HumanAddr, amount: u128
    ) -> StdResult<HandleResponse> {
        self.tx(height, agent, TX::Retrieve { amount: amount.into() })
    }
    pub fn tx_claim (
        &mut self, height: u64, agent: &HumanAddr
    ) -> StdResult<HandleResponse> {
        self.tx(height, agent, TX::Claim {})
    }
}
