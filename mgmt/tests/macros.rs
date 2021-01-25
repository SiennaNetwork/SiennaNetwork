macro_rules! query {
    (
        $deps:ident $Query:ident
        ($res:ident: $Response:ident) $Assertions:block
    ) => {
        let $res: mgmt::msg::$Response = from_binary(
            &mgmt::query(&$deps, mgmt::msg::QueryMsg::$Query {}).unwrap()
        ).unwrap();
        $Assertions
    }
}

macro_rules! tx {
    (
        $deps:ident $env:ident
        $Msg:ident $({ $($arg:ident : $val:expr),* })?
    ) => {
        let msg = mgmt::msg::HandleMsg::$Msg { $($($arg:$val)*)? };
        let _ = mgmt::handle(&mut $deps, $env, msg);
    }
}
