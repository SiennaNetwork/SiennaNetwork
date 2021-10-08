use fadroma::scrt::cosmwasm_std::{
    Uint128, HumanAddr, StdResult, StdError, Binary,
    Extern, testing::MockApi, MemoryStorage,
    Querier, QueryRequest, Empty, WasmQuery, QuerierResult,
    from_binary, to_binary, from_slice, SystemError,
    BlockInfo, MessageInfo, ContractInfo, Env
};

use fadroma::scrt::callback::{ContractInstance as ContractLink};

use fadroma::scrt::snip20_api::mock::*;

const ADDR_LEN: usize = 45;

struct MockQuerier {
    balance: Uint128
}

impl Querier for MockQuerier {
    fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
        let s = match std::str::from_utf8(&bin_request) {
            Ok(v) => v,
            Err(e) => panic!("Invalid UTF-8 sequence: {}", e),
        };
        println!("raw_query: {}", &s);
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
                println!("raw_query.wasm.msg: {:#?}", &std::str::from_utf8(&msg.as_slice())?);
                Ok(to_binary(&self.mock_query_dispatch(&ContractLink {
                    code_hash: callback_code_hash,
                    address: contract_addr
                }, &msg)))
            },
            _ => panic!("MockSnip20Querier: Expected WasmQuery::Smart.")
        }
    }
}

impl MockQuerier {
    fn mock_query_dispatch (
        &self,
        link: &ContractLink<HumanAddr>,
        msg:  &Binary
    ) -> StdResult<Binary> {
        if link.address == HumanAddr::from("POOL") {
            let query: Snip20Query = from_binary(&msg)?;
            println!("POOL->{:#?}", &query);
        } else {
            println!("{:#?}->{:#?}", &link, &msg);
        }
        unimplemented!();
        //match msg {
            //Snip20Query::Balance { .. } => {
                ////if contract != self.reward_token {
                    ////panic!("MockSnip20Querier: Expected balance query for {:?}", self.reward_token)
                ////}
                //Snip20QueryAnswer::Balance { amount: self.balance }
            //},

            //_ => unimplemented!()
        //}
    }
    pub fn increment_balance (&mut self, amount: u128) {
        self.balance = self.balance + amount.into();
    }
    pub fn decrement_balance (&mut self, amount: u128) -> StdResult<()> {
        self.balance = (self.balance - amount.into())?;
        Ok(())
    }
}

#[test] fn test_rewards_emergency_proxy () -> StdResult<()> {

    let mut deps = Extern {
        storage:   MemoryStorage::default(),
        api:       MockApi::new(ADDR_LEN),
        querier:   MockQuerier { balance: 0u128.into() },
    };

    let agent = HumanAddr::from("AGENT");

    let collector = HumanAddr::from("COLLECTOR");

    let pool = ContractLink {
        address:   "POOL".into(),
        code_hash: "POOL_HASH".into()
    };

    let reward_token = ContractLink {
        address:   "SIENNA".into(),
        code_hash: "SIENNA_HASH".into()
    };

    let init_result = crate::init(&mut deps, Env {
        block:    BlockInfo    { height: 0u64, time: 0u64, chain_id: "secret".into() },
        message:  MessageInfo  { sender: agent.clone(), sent_funds: vec![] },
        contract: ContractInfo { address: "PROXY".into() },
        contract_key:       Some("PROXY_KEY".into()),
        contract_code_hash: "PROXY_HASH".into()
    }, crate::msg::Init {
        collector,
        reward_token
    })?;

    println!("{:#?}", &init_result);

    let handle_result_1 = crate::handle(&mut deps, Env {
        block:    BlockInfo    { height: 0u64, time: 0u64, chain_id: "secret".into() },
        message:  MessageInfo  { sender: agent.clone(), sent_funds: vec![] },
        contract: ContractInfo { address: "PROXY".into() },
        contract_key:       Some("PROXY_KEY".into()),
        contract_code_hash: "PROXY_HASH".into()
    }, crate::msg::Handle::Claim {
        pool: pool.clone(),
        key:  "".into()
    })?;

    println!("{:#?}", &handle_result_1);

    let handle_result_2 = crate::handle(&mut deps, Env {
        block:    BlockInfo    { height: 0u64, time: 0u64, chain_id: "secret".into() },
        message:  MessageInfo  { sender: agent.clone(), sent_funds: vec![] },
        contract: ContractInfo { address: "PROXY".into() },
        contract_key:       Some("PROXY_KEY".into()),
        contract_code_hash: "PROXY_HASH".into()
    }, crate::msg::Handle::Claim {
        pool: pool.clone(),
        key:  "".into()
    })?;

    println!("{:#?}", &handle_result_2);

    Ok(())

}
