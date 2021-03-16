/// Create an error result
#[macro_export] macro_rules! Error {
    ($msg:expr) => {
        Err(cosmwasm_std::StdError::GenericErr {
            msg: $msg.to_string(),
            backtrace: None //Some(snafu::Backtrace::generate())
        })
    };
}

/// Define error conditions (inside and `impl` block)
#[macro_export] macro_rules! define_errors {
    ($( $Struct:ident {
        $(
            $name:ident
            ($(&$self:ident,)? $($arg:ident : $type:ty),*)
            -> ($format:literal $(, $var:expr)*)
        )*
    } )*) => {
        $(
            impl $Struct {
                $(
                    #[doc=$format]
                    pub fn $name<T> ($(&$self,)? $($arg : $type),*) -> StdResult<T> {
                        Error!(format!($format $(, $var)*))
                    }
                )*
            }
        )*
    }
}

use crate::{Schedule, Pool, Account};
use cosmwasm_std::{StdResult, Uint128};
define_errors!(
    Schedule {
        err_total (actual: u128, expected: Uint128) ->
            ("schedule: pools add up to {}, expected {}",
                actual, expected)
    }
    Pool {
        err_total (name: &str, actual: u128, expected: &Uint128) ->
            ("pool {}: accounts add up to {}, expected {}",
                name, actual, expected)
        err_add_account_complete (&self,) ->
            ("pool {}: can't add any more accounts to this pool",
                &self.name)
        err_add_account_too_big (&self, actual: Uint128, expected: u128) ->
            ("pool {}: account ({}) > unallocated funds in pool ({})",
                &self.name, actual, expected)
    }
    Account {
        err_cliff_too_big (&self,) ->
            ("account {}: cliff ({}) > total ({})",
                &self.name, &self.cliff, &self.amount)
    }
);
