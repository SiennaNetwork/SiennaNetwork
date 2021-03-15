#![allow(non_snake_case)]
#[macro_use] extern crate kukumba;

use cosmwasm_std::{
    Env, BlockInfo, MessageInfo, ContractInfo,
    from_binary, CosmosMsg, WasmMsg, HumanAddr, Uint128,
    testing::mock_dependencies_with_balances
};
use sienna_rpt::{init, query, msg};

kukumba!(

    #[rpt]
    given "the contract is not yet deployed" {
        let ALICE   = HumanAddr::from("secret1ALICE");
        let BOB     = HumanAddr::from("secret1BOB");
        let CAROL   = HumanAddr::from("secret1CAROL");
        let MALLORY = HumanAddr::from("secret1MALLORY");
        let config = vec![ (BOB.clone(),   Uint128::from(1000u128))
                         , (CAROL.clone(), Uint128::from(1500u128)) ];
        let mut deps = mock_dependencies_with_balances(45, &[(&ALICE, &[])]);
    }
    when "someone deploys the contract" {
        assert_eq!(
            init(&mut deps, mock_env(0, 0, &ALICE), msg::Init {
                config: config.clone(),
                token_addr:  HumanAddr::from("token"),
                token_hash:  String::new(),
                mgmt_addr:   HumanAddr::from("mgmt"),
                mgmt_hash:   String::new(),
            }).unwrap().messages.len(),
            0,
            "deploy failed"
        );
    }
    then "they become admin" {
        assert_eq!(
            from_binary::<msg::Response>(&query(&deps, msg::Query::Status {}).unwrap()).unwrap(),
            msg::Response::Status { errors: 0, config: config },
            "querying status failed"
        );
    }

);

fn mock_env (height: u64, time: u64, sender: &HumanAddr) -> Env {
    Env {
        block: BlockInfo { height, time, chain_id: "secret".into() },
        message: MessageInfo { sender: sender.into(), sent_funds: vec![] },
        contract: ContractInfo { address: "mgmt".into() },
        contract_key: Some("".into()),
        contract_code_hash: "0".into()
    }
}
