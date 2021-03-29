#![allow(unused_macros)]

use cosmwasm_std::{
    HumanAddr, Coin,
    Extern, MemoryStorage,
    Env, BlockInfo, MessageInfo, ContractInfo,
};

use cosmwasm_std::testing::{
    mock_dependencies_with_balances, /*mock_env,*/
    MockApi, MockQuerier
};

type ExternMock = Extern<MemoryStorage, MockApi, MockQuerier>;
type HandleResult = cosmwasm_std::StdResult<cosmwasm_std::HandleResponse>;

macro_rules! harness {
    ($deps:ident ; $($AGENT:ident),+) => {
        $(let $AGENT: cosmwasm_std::HumanAddr =
            cosmwasm_std::HumanAddr::from(stringify!($AGENT));)+
        let mut $deps = harness(&[
            $((&$AGENT, &[])),+
        ]);
    }
}

pub fn harness (balances: &[(&HumanAddr, &[Coin])])-> ExternMock {
    let mut deps = mock_dependencies_with_balances(45, &balances);
    let res = sienna_mgmt::init(
        &mut deps,
        mock_env(0, 0, balances[0].0), // first address in `balances` is admin
        sienna_mgmt::msg::Init {
            schedule: sienna_schedule::Schedule::new(&[]),
            token:    (cosmwasm_std::HumanAddr::from("token"), String::new()),
        }
    ).unwrap();
    assert_eq!(0, res.messages.len());
    deps
}

pub fn mock_env (
    height: u64, time: u64, sender: &HumanAddr
) -> Env { Env {
    block: BlockInfo { height, time, chain_id: "secret".into() },
    message: MessageInfo { sender: sender.into(), sent_funds: vec![] },
    contract: ContractInfo { address: "mgmt".into() },
    contract_key: Some("".into()),
    contract_code_hash: "0".into()
} }

macro_rules! test_q {
    ( $deps:expr;
        $QueryVariant:ident $( { $($query_field:ident $(: $query_value:expr)?),* } )? ==
        $ResponseVariant:ident { $($response_field:ident : $expected_value:expr),* }
    ) => {
        let msg = sienna_mgmt::msg::Query::$QueryVariant {
            $( $($query_field $(:$query_value)?),* )?
        };
        let response = sienna_mgmt::query(&$deps, msg).unwrap();
        match cosmwasm_std::from_binary(&response).unwrap() {
            sienna_mgmt::msg::Response::$ResponseVariant {$($response_field),*} => {
                $(assert_eq!($response_field, $expected_value));*
            },
            _ => {
                panic!("{} didn't return {}", stringify!($QueryVariant), stringify!($ResponseVariant))
            },
        }
    }
}

macro_rules! test_tx {
    ( $deps: ident; $SENDER:expr, $block:expr, $time:expr
    ; $TX:ident { $($arg:ident : $val:expr),* }
    == $ExpectedResult:expr
    ) => {

        macro_rules! ok {
            () => {
                Ok(cosmwasm_std::HandleResponse { data: None, log: vec![], messages: vec![] })
            };
            (messages: $msgs:tt) => {
                Ok(cosmwasm_std::HandleResponse { data: None, log: vec![], messages: vec! $msgs })
            };
            (launched: $amount:expr) => {
                ok!(messages: [
                    secret_toolkit::snip20::handle::mint_msg(
                        cosmwasm_std::HumanAddr::from("mgmt"),
                        cosmwasm_std::Uint128::from($amount),
                        None, 256, String::new(), cosmwasm_std::HumanAddr::from("token")
                    ).unwrap(),
                    secret_toolkit::snip20::handle::set_minters_msg(
                        vec![],
                        None, 256, String::new(), cosmwasm_std::HumanAddr::from("token")
                    ).unwrap()
                ])
            };
            (claimed: $addr:expr, $amount:expr) => {
                ok!(messages: [
                    secret_toolkit::snip20::handle::transfer_msg(
                        $addr.clone(), $amount,
                        None, 256, String::new(), HumanAddr::from("token")
                    ).unwrap()
                ])
            };
        }
        macro_rules! err {
            (auth) => { Err(cosmwasm_std::StdError::Unauthorized { backtrace: None }) };
            ($msg:tt) => { Err(cosmwasm_std::StdError::GenericErr {
                backtrace: None, msg: $msg.to_string()
            }) }
        }

        let expected_response = $ExpectedResult;
        let request = sienna_mgmt::msg::Handle::$TX { $($arg : $val),* };
        let response = sienna_mgmt::handle(&mut $deps, mock_env($block, $time, &$SENDER), request);
        if response != expected_response {
            println!("!!! Test transaction failed (block {}, time {})", $block, $time);
            if let Ok(cosmwasm_std::HandleResponse { messages, log, data }) = expected_response {
                println!("Expected data =\n {:#?}", &data);
                println!("Expected logs =\n {:#?}", &log);
                println!("Expected messages:");
                for message in messages.iter() {
                    match message {
                        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute{contract_addr,callback_code_hash,msg,send}) => {
                            println!("  WASM execute");
                            println!("    contract_addr:     = {:?}", contract_addr);
                            println!("    callback_code_hash = {:?}", callback_code_hash);
                            println!("    send               = {:?}", send);
                            println!("    msg:\n{}", std::str::from_utf8(msg.as_slice()).unwrap())
                        },
                        _ =>
                            println!("  {:#?}", &message)
                    }
                }
            } else {
                println!("Expected response:\n  {:#?}", &expected_response);
            }
            if let Ok(cosmwasm_std::HandleResponse { messages, log, data }) = response {
                println!("Actual data =\n{:#?}", &data);
                println!("Actual logs =\n{:#?}", &log);
                println!("Actual messages:");
                for message in messages.iter() {
                    match message {
                        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute{contract_addr,callback_code_hash,msg,send}) => {
                            println!("  WASM execute");
                            println!("    contract_addr:     = {:?}", contract_addr);
                            println!("    callback_code_hash = {:?}", callback_code_hash);
                            println!("    send:              = {:?}", send);
                            println!("    msg:\n{}", std::str::from_utf8(msg.as_slice()).unwrap())
                        },
                        _ =>
                            println!("  {:#?}", &message)
                    }
                }
            } else {
                println!("Actual response:\n  {:#?}", &response);
            }
            panic!("transaction test failed")
        }
    }
}

/// Add 18 zeroes and make serializable
macro_rules! SIENNA {
    ($x:expr) => { Uint128::from($x as u128 * sienna_schedule::ONE_SIENNA) }
}
