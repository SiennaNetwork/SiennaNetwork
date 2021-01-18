use cosmwasm_std::testing::{mock_dependencies_with_balances, mock_env};
use cosmwasm_std::{coins, from_binary, StdError, HumanAddr};

use sienna_mgmt::contract::{init, query, handle};
use sienna_mgmt::msg::{InitMsg, HandleMsg, QueryMsg, StatusResponse};

#[test] fn proper_initialization() {

    // context
    let mut deps = mock_dependencies_with_balances(20, &[
        (&HumanAddr::from("Alice"),   &coins(1000, "SIENNA")),
        (&HumanAddr::from("Bob"),     &coins(1000, "SIENNA")),
        (&HumanAddr::from("Mallory"), &coins(   0, "SIENNA"))
    ]);

    // we can just call .unwrap() to assert this was a success
    let env = mock_env("Alice", &coins(1000, "SIENNA"));
    let res = init(&mut deps, env, InitMsg { token_contract: None }).unwrap();
    assert_eq!(0, res.messages.len());

    // it worked, let's query the state
    let res = query(&deps, QueryMsg::StatusQuery {}).unwrap();
    let value: StatusResponse = from_binary(&res).unwrap();
    assert_eq!(value.launched, None);

    // try triggering a mutation
    let env = mock_env("Alice", &coins(1000, "SIENNA"));
    let time = env.block.time;
    let res = handle(&mut deps, env, HandleMsg::Launch {});

    // check that the state has changed
    let res = query(&deps, QueryMsg::StatusQuery {}).unwrap();
    let value: StatusResponse = from_binary(&res).unwrap();
    assert_eq!(value.launched, Some(time));
}

//#[test] fn operations () {
    //let mut deps = mock_dependencies(20, &coins(2, "token"));
    //let msg = InitMsg { value: 17 };
    //let env = mock_env("creator", &coins(2, "token"));
    //let _res = init(&mut deps, env, msg).unwrap();
    //let env = mock_env("anyone", &coins(2, "token"));
    //let msg = HandleMsg::Add { augend: 3 };
    //let _res = handle(&mut deps, env, msg).unwrap();
    //let res = query(&deps, QueryMsg::Equals {}).unwrap();
    //let value: EqualsResponse = from_binary(&res).unwrap();
    //assert_eq!(20, value.value);
    //println!("{}", value.value);
    //let env = mock_env("anyone", &coins(2, "token"));
    //let msg = HandleMsg::Sub { subtrahend: 10 };
    //let _res = handle(&mut deps, env, msg).unwrap();
    //let res = query(&deps, QueryMsg::Equals {}).unwrap();
    //let value: EqualsResponse = from_binary(&res).unwrap();
    //assert_eq!(10, value.value);
    //println!("{}", value.value);
    //let env = mock_env("anyone", &coins(2, "token"));
    //let msg = HandleMsg::Mul { multiplier: 10 };
    //let _res = handle(&mut deps, env, msg).unwrap();
    //let res = query(&deps, QueryMsg::Equals {}).unwrap();
    //let value: EqualsResponse = from_binary(&res).unwrap();
    //assert_eq!(100, value.value);
    //println!("{}", value.value);
    //let env = mock_env("anyone", &coins(2, "token"));
    //let msg = HandleMsg::Div { divisor: 20 };
    //let _res = handle(&mut deps, env, msg).unwrap();
    //let res = query(&deps, QueryMsg::Equals {}).unwrap();
    //let value: EqualsResponse = from_binary(&res).unwrap();
    //assert_eq!(5, value.value);
    //println!("{}", value.value);
//}

