#![cfg(test)]
#![allow(unused_macros)]
#![allow(non_snake_case)]

#[macro_use] extern crate kukumba;
use fadroma::scrt::{
    cosmwasm_std::{
        HumanAddr, StdResult, StdError,
        InitResponse, HandleResponse, Binary,
        Env, BlockInfo, MessageInfo, ContractInfo,
        Extern, Storage, Api, Querier, MemoryStorage, 
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

macro_rules! assert_error {
    ($response:expr, $msg:expr) => {
        assert_eq!($response, Err(StdError::GenericErr {
            msg: $msg.into(),
            backtrace: None
        }))
    }
}

struct Harness {
    deps: Extern<MemoryStorage, MockApi, MockQuerier>
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

kukumba! {
    StdError,

    #[ok_init]
    given "no instance" {
        let mut harness = Harness::new();
        let admin  = HumanAddr::from("admin");
    }
    when  "someone inits with an asset token address" {
        let result = harness.init_configured(0, &admin)?;
    }
    then  "the instance is ready" {
        let result = harness.q_status(1u64)?;
    }

    #[ok_init_then_provide]
    given  "no instance" {
        let mut harness = Harness::new();
        let admin  = HumanAddr::from("admin");
        let badman = HumanAddr::from("badman");
    }
    when  "someone inits without providing an asset token address" {
        let result = harness.init(0, &admin, Init {
            provided_token: None,
            rewarded_token: ContractLink {
                address:    HumanAddr::from("rewarded_token_address"),
                code_hash:  "rewarded_token_hash".into(),
            },
            viewing_key: "".into()
        });
    }
    then  "the instance is not ready" {
        let result = harness.q_status(1u64)?;
    }
    when  "a stranger tries to provide an asset token address" {
        let result = harness.tx(2, &badman, TX::SetProvidedToken {
            address:   HumanAddr::from("malicious_address"),
            code_hash: "malicious_hash".into(),
        })?;
    }
    then  "an error is returned and nothing changes" {
        let result = harness.q_status(3u64)?;
    }
    when  "the admin provides an asset token address" {
        let result = harness.tx(4, &admin, TX::SetProvidedToken {
            address:   HumanAddr::from("provided_token_address"),
            code_hash: "provided_token_hash".into(),
        })?;
    }
    then  "the instance is ready" {
        let result = harness.q_status(5u64)?;
    }

    #[lock_and_retrieve]
    given "an instance" {
        let mut harness = Harness::new();
        let admin   = HumanAddr::from("admin");
        let alice   = HumanAddr::from("alice");
        let bob     = HumanAddr::from("bob");
        let mallory = HumanAddr::from("mallory");
        let result  = harness.init_configured(0, &admin);
    }
    when  "someone requests to lock tokens"
    then  "the instance transfers them to itself"
    and   "the liquidity provider starts accruing a reward" {
        let result = harness.tx_lock(1, &alice, 100u128)?;
        let result = harness.q_status(2u64)?;
    }
    when  "a provider requests to retrieve tokens"
    then  "the instance transfers them to the provider"
    and   "the reward now increases at a reduced rate" {
        let result = harness.tx_retrieve(3, &alice, 50u128)?;
        let result = harness.q_status(4u64.into())?;
    }
    when  "a provider requests to retrieve all their tokens"
    then  "the instance transfers them to the provider"
    and   "their reward stops increasing" {
        let result = harness.tx_retrieve(5, &alice, 50u128.into())?;
        let result = harness.q_status(5u64)?;
    }
    when  "someone else requests to lock tokens"
    then  "the previous provider's share of the rewards begins to diminish" {
        let result = harness.tx_lock(6, &bob, 100)?;
        let result = harness.q_status(7u64)?;
        let result = harness.q_status(8u64)?;
    }
    when  "a provider tries to retrieve too many tokens"
    then  "they get an error" {
        let result = harness.tx_retrieve(9, &bob, 1000u128);
        assert_error!(result, "not enough balance ({} < {})");
    }
    when  "a stranger tries to retrieve any tokens"
    then  "they get an error" {
        assert_error!(
            harness.tx_retrieve(10, &mallory, 100u128),
            "never provided liquidity"
        );
    }

    #[ok_claim]
    given "an instance" {
        let mut harness = Harness::new();
        let alice  = HumanAddr::from("alice");
    }
    when  "a stranger tries to claim rewards"
    then  "they get an error" {
        let result = harness.tx_claim(1, &alice)?;
    }
    when  "a provider claims their rewards" {
        let result = harness.tx_claim(1, &alice)?;
    }
    then  "the instance sends them amount of reward tokens" {}
    when  "a provider claims their rewards twice" {}
    then  "they are sent only once" {}
    when  "a provider claims their rewards later" {}
    then  "they receive an increment" {}

    #[rewards_parallel_or_sequential]
    given "todo" {}
    when  "do"   {}
    then  "done" {}

}
