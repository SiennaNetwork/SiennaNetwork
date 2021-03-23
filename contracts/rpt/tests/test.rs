#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;

use cosmwasm_std::{
    Extern, Storage, Api, testing::{MockStorage, MockApi},
    SystemResult, StdResult, StdError,
    Env, BlockInfo, MessageInfo, ContractInfo,
    Querier, QueryRequest, Empty, WasmQuery, QuerierResult,
    CosmosMsg, WasmMsg,
    HandleResponse,
    Binary, from_binary, from_slice, to_binary,
    HumanAddr, Uint128,
};
use sienna_rpt::{
    init, query, handle,
    msg::{Init as RPTInit, Query as RPTQuery, Handle as RPTHandle, Response as RPTResponse}
};
use sienna_mgmt::msg::{Query as MGMTQuery, Response as MGMTResponse, Handle as MGMTHandle};
use snip20_reference_impl::msg::{HandleMsg as TokenHandle};
use linear_map::LinearMap;

kukumba!(

    #[rpt]
    given "the contract is not yet deployed" {
        let ALICE   = HumanAddr::from("secret1ALICE");
        let BOB     = HumanAddr::from("secret1BOB");
        let CAROL   = HumanAddr::from("secret1CAROL");
        let MALLORY = HumanAddr::from("secret1MALLORY");
        let mut deps = Extern {
            storage: MockStorage::default(),
            api:     MockApi::new(45),
            querier: MockQuerier { portion: 2500 }
        }
        let config = LinearMap(vec![
            (BOB.clone(),   Uint128::from(1000u128)),
            (CAROL.clone(), Uint128::from(1500u128))
        ]);
    }

    when "someone deploys the contract" {
        assert_eq!(
            init(&mut deps, mock_env(0, 0, &ALICE), RPTInit {
                pool:       "Pool0".to_string(),
                account:    "Account0".to_string(),
                config:     LinearMap(vec![]),
                token_addr: HumanAddr::from("token"),
                token_hash: String::new(),
                mgmt_addr:  HumanAddr::from("mgmt"),
                mgmt_hash:  String::new(),
            }).unwrap().messages.len(),
            0,
            "deploy failed"
        );
    }

    then "they become admin"
    and "they can set the configuration"
    and "noone else can"
    and "it has to be a valid configuration" {
        assert_eq!(
            status(&deps),
            RPTResponse::Status { config: LinearMap(vec![]) },
            "querying status failed"
        );
        assert_eq!(
            (
                handle(&mut deps, mock_env(1, 1, &MALLORY), RPTHandle::Configure {
                    config: config.clone()
                }),
                status(&deps),
            ),
            (
                Err(cosmwasm_std::StdError::Unauthorized { backtrace: None }),
                RPTResponse::Status { config: LinearMap(vec![]) },
            ),
            "wrong user was able to set config"
        );
        assert_eq!(
            {
                handle(&mut deps, mock_env(2, 2, &ALICE), RPTHandle::Configure {
                    config: LinearMap(vec![(BOB.clone(),   Uint128::from(1001u128)),
                                           (CAROL.clone(), Uint128::from(1500u128))])
                });
                status(&deps)
            },
            RPTResponse::Status { config: LinearMap(vec![]) },
            "admin was able to set invalid config"
        );
        assert_eq!(
            {
                handle(&mut deps, mock_env(2, 2, &ALICE), RPTHandle::Configure {
                    config: config.clone()
                });
                status(&deps)
            },
            RPTResponse::Status { config: config.clone() },
            "admin was unable to set valid config"
        );
    }

    when "anyone calls the vest method"
    then "the contract claims funds from mgmt"
    and "it distributes them to the configured recipients" {
        let messages = handle(
            &mut deps, mock_env(2, 2, &MALLORY), RPTHandle::Vest {}
        ).unwrap().messages;
        assert_eq!(messages.len(), 3, "unexpected message count");
        // check claim from token
        if let CosmosMsg::Wasm(WasmMsg::Execute {
            msg, contract_addr, callback_code_hash, ..
        }) = messages.get(0).unwrap() {
            if let MGMTHandle::Claim {..} = from_binary::<MGMTHandle>(&msg).unwrap() {} else {
                panic!("unexpected 1st message");
            }
        } else {
            panic!("unexpected 1st message");
        }
        // check vestings to recipients
        for i in 1..3 {
            if let CosmosMsg::Wasm(WasmMsg::Execute {
                msg, contract_addr, callback_code_hash, ..
            }) = messages.get(i).unwrap() {
                if let TokenHandle::Transfer {recipient,amount,..} = from_binary::<TokenHandle>(&msg).unwrap() {
                    let (expected_recipient, expected_amount) = config.0.get(i-1).unwrap();
                    assert_eq!(*expected_recipient, recipient);
                    assert_eq!(*expected_amount, amount);
                } else {
                    panic!("unexpected message #{}", i+1);
                }
            } else {
                panic!("unexpected message #{}", i+1);
            }
        }
    }

);

fn mock_env (height: u64, time: u64, sender: &HumanAddr) -> Env {
    Env {
        block: BlockInfo { height, time, chain_id: "secret".into() },
        message: MessageInfo { sender: sender.into(), sent_funds: vec![] },
        contract: ContractInfo { address: "rpt".into() },
        contract_key: Some("".into()),
        contract_code_hash: "0".into()
    }
}

fn status<S:Storage,A:Api,Q:Querier> (deps: &Extern<S,A,Q>) -> RPTResponse {
    from_binary::<RPTResponse>(
        &query(&deps, RPTQuery::Status {}).unwrap()
    ).unwrap()
}

//fn print_type_of<T>(_: &T) {
    //println!("{}", std::any::type_name::<T>())
//}
struct MockQuerier {
    portion: u128
}
impl Querier for MockQuerier {
    fn raw_query (&self, bin_request: &[u8]) -> QuerierResult {
        let request: QueryRequest<Empty> = from_slice(bin_request).unwrap();
        match &request {
            QueryRequest::Wasm(msg) => {
                match msg {
                    WasmQuery::Smart { contract_addr, msg, .. } => {
                        let mgmt = HumanAddr::from("mgmt");
                        match &contract_addr {
                            mgmt => {
                                //let msg: MGMTQuery = from_binary(&msg).unwrap();
                                let response = MGMTResponse::Portion {
                                    portion: Uint128::from(self.portion)
                                };
                                QuerierResult::Ok(to_binary(&response))
                            },
                            _ => unimplemented!()
                        }
                    },
                    _ => unimplemented!(),
                }
            },
            _ => unimplemented!(),
        }
    }
}
