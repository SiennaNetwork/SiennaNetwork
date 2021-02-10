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

    // As the admin
    // When I init the contract
    // Then I want to be able to query its state
    let res = sienna_mgmt::init(
        &mut deps,
        mock_env(0, 0, balances[0].0), // first address in `balances` is admin
        sienna_mgmt::msg::Init {
            token_addr: cosmwasm_std::HumanAddr::from("token"),
            token_hash: String::new(),
            schedule: None
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

pub fn tx (
    deps: &mut ExternMock,
    env:  Env,
    tx:   sienna_mgmt::msg::Handle
) -> HandleResult {
    sienna_mgmt::handle(deps, env, tx)
}

macro_rules! test_tx {
    ( $deps: ident, $SENDER:expr, $block:expr, $time:expr
    ; $TX:expr => $ExpectedResult:expr
    ) => {
        let expected_response = $ExpectedResult;
        let response = tx(&mut $deps, mock_env($block, $time, &$SENDER), $TX);
        if response != expected_response {
            println!("TRANSACTION TEST FAILED");
            if let Ok(cosmwasm_std::HandleResponse { messages, log, data }) = expected_response {
                println!("Expected messages:");
                for message in messages.iter() {
                    match message {
                        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute{contract_addr,callback_code_hash,msg,send}) => {
                            println!("  WASM execute");
                            println!("    contract_addr:      {:?}", contract_addr);
                            println!("    callback_code_hash: {:?}", callback_code_hash);
                            println!("    send:               {:?}", send);
                            println!("    msg:\n      {}", std::str::from_utf8(msg.as_slice()).unwrap())
                        },
                        _ =>
                            println!("  {:#?}", &message)
                    }
                }
                println!("Expected log:\n  {:#?}", &log);
                println!("Expected data:\n  {:#?}", &data);
            } else {
                println!("Expected response:\n  {:#?}", &expected_response);
            }
            if let Ok(cosmwasm_std::HandleResponse { messages, log, data }) = response {
                println!("Actual messages:");
                for message in messages.iter() {
                    match message {
                        cosmwasm_std::CosmosMsg::Wasm(cosmwasm_std::WasmMsg::Execute{contract_addr,callback_code_hash,msg,send}) => {
                            println!("  WASM execute");
                            println!("    contract_addr:      {:?}", contract_addr);
                            println!("    callback_code_hash: {:?}", callback_code_hash);
                            println!("    send:               {:?}", send);
                            println!("    msg:\n      {}", std::str::from_utf8(msg.as_slice()).unwrap())
                        },
                        _ =>
                            println!("  {:#?}", &message)
                    }
                }
                println!("Actual log:\n  {:#?}", &log);
                println!("Actual data:\n  {:#?}", &data);
            } else {
                println!("Actual response:\n  {:#?}", &response);
            }
            panic!("transaction test failed")
        }
    }
}

macro_rules! tx_ok {
    () => {
        Ok(cosmwasm_std::HandleResponse { data: None, log: vec![], messages: vec![] })
    };
    ($($msg: expr),+) => {
        Ok(cosmwasm_std::HandleResponse { data: None, log: vec![], messages: vec![$($msg),+] })
    }
}

macro_rules! tx_ok_claim {
    ($addr:expr, $amount:expr) => {
        tx_ok!(transfer_msg(
            $addr.clone(), $amount,
            None, 256, String::new(), HumanAddr::from("token")
        ).unwrap())
    }
}

macro_rules! tx_err {
    ($msg:expr) => {
        Err(cosmwasm_std::StdError::GenericErr { backtrace: None, msg: $msg.to_string() })
    }
}

macro_rules! tx_err_auth {
    () => {
        Err(cosmwasm_std::StdError::Unauthorized { backtrace: None })
    }
}

macro_rules! test_q {
    ( $deps:expr
    , $Query:ident
    ; $Response:ident {
        $($arg:ident : $val:expr),*
    } ) => {
        match cosmwasm_std::from_binary(
            &sienna_mgmt::query(&$deps, sienna_mgmt::msg::Query::$Query {}).unwrap()
        ).unwrap() {
            sienna_mgmt::msg::Response::$Response {$($arg),*} => {
                $(assert_eq!($arg, $val));*
            },
            _ => panic!("{} didn't return {}",
                stringify!($Query), stringify!($Response)),
        }
    }
}

/// Add 18 zeroes and make serializable
macro_rules! SIENNA {
    ($x:expr) => { Uint128::from($x as u128 * sienna_schedule::ONE_SIENNA) }
}
