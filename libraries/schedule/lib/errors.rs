//! Error definitions

/// Create an error result
#[macro_export] macro_rules! Error {
    ($msg:expr) => {
        Err(cosmwasm_std::StdError::GenericErr {
            msg: $msg.to_string(),
            backtrace: None //Some(snafu::Backtrace::generate())
        })
    };
}

/// `impl` error methods on one or more structs
#[macro_export] macro_rules! define_errors {
    ($(
        $Struct:ident $(<$G:tt$(:$GG:tt)?>)? { $(
            $name:ident
            ($(&$self:ident,)? $($arg:ident : $type:ty),*)
            { $format:literal $(, $var:expr)* }
        )* }
    )*) => {
        $( impl $(<$G$(:$GG)?>)? $Struct $(<$G>)? { $(
            #[doc=$format]
            pub fn $name<T> ($(&$self,)? $($arg : $type),*) -> cosmwasm_std::StdResult<T> {
                Error!(format!($format $(, $var)*))
            }
        )* } )*
    }
}

use crate::{Schedule, Pool, Account};
define_errors!(
    Schedule<A:Clone> {
        err_total (&self,) {
            "schedule: pools add up to {}, expected {}",
            &self.subtotal(), &self.total
        }
        err_pool_not_found (&self, name: &str) {
            "schedule: pool {} not found",
            &name
        }
    }
    Pool<A:Clone> {
        err_total (&self,) {
            "pool {}: accounts add up to {}, expected {}",
            &self.name, &self.subtotal(), &self.total
        }
        err_pool_full (&self,) {
            "pool {}: can't add any more accounts to this pool",
            &self.name
        }
        err_account_too_big (&self, account: &Account<A>) {
            "pool {}: account ({}) > unallocated funds in pool ({})",
            &self.name,
            account.amount.u128(),
            self.unallocated()
        }
    }
    Account<A:Clone> {
        err_empty (&self,) {
            "account {}: amount must be >0",
            &self.name
        }
        err_cliff_too_big (&self,) {
            "account {}: cliff ({}) > total ({})",
            &self.name,
            &self.cliff,
            &self.amount
        }
        err_does_not_add_up (&self,) {
            "account {}: cliff + portions + remainder don't add up to amount",
            &self.name
        }
    }
);
