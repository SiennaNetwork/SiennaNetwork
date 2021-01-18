#[macro_use] extern crate fadroma;

contract!(
    b"config"
    InitMsg (deps, env, msg: { value: i32 }) -> State {
        value: i32           = msg.value,
        owner: CanonicalAddr = deps.api.canonical_address(&env.message.sender)?
    }
    QueryMsg (deps, msg) {
        Equals () {
            let state = config_read(&deps.storage).load()?;
            to_binary(&crate::msg::EqualsResponse { value: state.value })
        }
    }
    HandleMsg (deps, env, msg) {
        Add {augend:     i32} (&mut state) {
            state.value += augend;
            Ok(state)
        }
        Sub {subtrahend: i32} (&mut state) {
            state.value -= subtrahend;
            Ok(state)
        }
        Mul {multiplier: i32} (&mut state) {
            state.value *= multiplier;
            Ok(state)
        }
        Div {divisor:    i32} (&mut state) {
            state.value /= divisor;
            Ok(state)
        }
    }
    Response {
        EqualsResponse { value: i32 }
    }
);

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::{mock_dependencies, mock_env};
    use cosmwasm_std::{coins, from_binary, StdError};
    use super::contract::{init, query, handle};
    use crate::msg::{InitMsg, HandleMsg, QueryMsg, EqualsResponse};
    #[test] fn proper_initialization() {
        let mut deps = mock_dependencies(20, &[]);
        let msg = InitMsg { value: 17 };
        let env = mock_env("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = init(&mut deps, env, msg).unwrap();
        assert_eq!(0, res.messages.len());
        // it worked, let's query the state
        let res = query(&deps, QueryMsg::Equals {}).unwrap();
        let value: EqualsResponse = from_binary(&res).unwrap();
        assert_eq!(17, value.value);
    }
    #[test] fn operations () {
        let mut deps = mock_dependencies(20, &coins(2, "token"));
        let msg = InitMsg { value: 17 };
        let env = mock_env("creator", &coins(2, "token"));
        let _res = init(&mut deps, env, msg).unwrap();
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Add { augend: 3 };
        let _res = handle(&mut deps, env, msg).unwrap();
        let res = query(&deps, QueryMsg::Equals {}).unwrap();
        let value: EqualsResponse = from_binary(&res).unwrap();
        assert_eq!(20, value.value);
        println!("{}", value.value);
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Sub { subtrahend: 10 };
        let _res = handle(&mut deps, env, msg).unwrap();
        let res = query(&deps, QueryMsg::Equals {}).unwrap();
        let value: EqualsResponse = from_binary(&res).unwrap();
        assert_eq!(10, value.value);
        println!("{}", value.value);
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Mul { multiplier: 10 };
        let _res = handle(&mut deps, env, msg).unwrap();
        let res = query(&deps, QueryMsg::Equals {}).unwrap();
        let value: EqualsResponse = from_binary(&res).unwrap();
        assert_eq!(100, value.value);
        println!("{}", value.value);
        let env = mock_env("anyone", &coins(2, "token"));
        let msg = HandleMsg::Div { divisor: 20 };
        let _res = handle(&mut deps, env, msg).unwrap();
        let res = query(&deps, QueryMsg::Equals {}).unwrap();
        let value: EqualsResponse = from_binary(&res).unwrap();
        assert_eq!(5, value.value);
        println!("{}", value.value);
    }
}
