use fadroma::scrt::{
    Uint128, HumanAddr, StdResult,
    Extern, testing::MockApi, MemoryStorage,
    Querier, QueryRequest, Empty, WasmQuery, QuerierResult,
    to_binary, from_slice, SystemError,
    BlockInfo, MessageInfo, ContractInfo, Env,
    CosmosMsg, WasmMsg
};

use fadroma::scrt_link::ContractLink;

use sienna_rewards::msg::Response as RewardsResponse;

const ADDR_LEN: usize = 45;

#[derive(Debug, serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum Snip20QueryAnswer {
    Balance { amount: Uint128 }
}

#[derive(Debug, serde::Serialize,serde::Deserialize)]
#[serde(rename_all="snake_case")]
pub enum RewardsQueryAnswer {
    UserInfo { claimable: Uint128 }
}

struct MockQuerier {
    pub balance: Uint128,
}

impl Querier for MockQuerier {

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
            QueryRequest::Wasm(WasmQuery::Smart {
                callback_code_hash,
                contract_addr,
                msg
            }) => {
                let decoded = std::str::from_utf8(&msg.as_slice()).unwrap();
                println!("raw_query.wasm.msg: {:#?}", &decoded.trim());
                if decoded.contains("user_info") {
                    Ok(to_binary(&RewardsResponse::UserInfo {
                        it_is_now:        0u64,
                        pool_closed:      None,
                        pool_last_update: 0u64,
                        pool_lifetime:    0u128.into(),
                        pool_locked:      0u128.into(),
                        user_last_update: Some(0u64),
                        user_lifetime:    0u128.into(),
                        user_locked:      0u128.into(),
                        user_share:       0u128.into(),
                        user_earned:      0u128.into(),
                        user_claimed:     0u128.into(),
                        user_claimable:   self.balance,
                        user_age:         0u64,
                        user_cooldown:    0u64
                    }))
                } else if decoded.contains("balance") {
                    Ok(to_binary(&Snip20QueryAnswer::Balance {
                        amount: self.balance
                    }))
                } else {
                    unimplemented!()
                }
            },
            _ => panic!("MockSnip20Querier: Expected WasmQuery::Smart.")
        }

    }
}

#[test] fn test_rewards_emergency_proxy () -> StdResult<()> {

    let mut deps = Extern {
        storage: MemoryStorage::default(),
        api:     MockApi::new(ADDR_LEN),
        querier: MockQuerier {
            balance: 397500u128.into(),
        },
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

    assert_eq!(handle_result_1.messages.len(), 2);
    for (index, expected) in vec![
        (0, "{\"claim\":{}}"),
        (1, "{\"transfer_from\":{\"owner\":\"AGENT\",\"recipient\":\"COLLECTOR\",\"amount\":\"395000\",\"padding\":null}}")
    ] {
        match handle_result_1.messages.get(index) {
            Some(CosmosMsg::Wasm(WasmMsg::Execute { msg, .. })) => {
                let decoded = std::str::from_utf8(&msg.as_slice()).unwrap();
                println!("handle_result_1.msg[{}]: {:#?}", index, &decoded.trim());
                assert_eq!(decoded.trim(), expected);
            },
            _ => unimplemented!()
        }
    }

    deps.querier.balance = 2500u128.into();

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

    assert_eq!(handle_result_2.messages.len(), 1);
    for (index, expected) in vec![
        (0, "{\"claim\":{}}"),
    ] {
        match handle_result_2.messages.get(index) {
            Some(CosmosMsg::Wasm(WasmMsg::Execute { msg, .. })) => {
                let decoded = std::str::from_utf8(&msg.as_slice()).unwrap();
                println!("handle_result_1.msg[{}]: {:#?}", index, &decoded.trim());
                assert_eq!(decoded.trim(), expected);
            },
            _ => unimplemented!()
        }
    }

    Ok(())

}
