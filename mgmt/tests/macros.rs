macro_rules! query {
    (
        $Query:ident ( $deps:ident ) -> $Response:ident ( $($arg:ident),* )
        $Assertions:block
    ) => {
        let response = from_binary(
            &mgmt::query(&$deps, mgmt::msg::Query::$Query {}).unwrap()
        ).unwrap();
        match response {
            mgmt::msg::Response::$Response {$($arg),*} => {
                $Assertions
            },
            _ => panic!("{} returned something other than {}",
                stringify!($Query), stringify!($Response))
        }
    }
}

macro_rules! tx {
    (
        $deps:ident $env:ident
        $Msg:ident $({ $($arg:ident : $val:expr),* })?
    ) => {
        let msg = mgmt::msg::Handle::$Msg { $($($arg:$val)*)? };
        let _ = mgmt::handle(&mut $deps, $env, msg);
    }
}

macro_rules! canon {
    ($deps:ident, $($x:tt)*) => {
        $deps.api.canonical_address($($x)*).unwrap();
    }
}
