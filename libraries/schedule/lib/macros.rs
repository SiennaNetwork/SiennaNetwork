//! Macros, for internal use>
//!
//! Example of using `schedule!` and `portions!` to materialize a schedule.
//!
//! ```rust
//! # #[allow(non_snake_case)]
//! # #[macro_use] extern crate sienna_schedule;
//! # extern crate snafu; use snafu::GenerateBacktrace;
//! # /// custom output on equality assertion fail
//! # fn type_of<T> (_: &T) -> String {
//! #   String::from(format!("{}", std::any::type_name::<T>()))
//! # }
//! # macro_rules! assert_eq { ($actual:expr, $expected:expr) => {
//! #   match (&$actual, &$expected) {
//! #       (actual_val, expected_val) => {
//! #           if !(*actual_val == *expected_val) {
//! #               println!("expected {}:", type_of(&expected_val));
//! #               match expected_val {
//! #                 Ok(val) => for x in val.iter() { println!("{}", x); },
//! #                 Err(e)  => println!("{:#?}", &e)
//! #               }
//! #               println!("\nactual {}:", type_of(&actual_val));
//! #               match actual_val {
//! #                 Ok(val) => for x in val.iter() { println!("{}", x); },
//! #                 Err(e)  => println!("{:#?}", &e)
//! #               }
//! #               panic!("schedule didn't generate expected portions");
//! #           }
//! #       }
//! #   }
//! # }; }
//! # fn main () {
//! // some imaginary people:
//! use cosmwasm_std::HumanAddr;
//! let Alice = HumanAddr::from("Alice");
//! let Bob   = HumanAddr::from("Bob");
//! let Candy = HumanAddr::from("Candy");
//! // some empty, but valid schedules
//! use sienna_schedule::Vesting; // needed for .all() trait fn
//! assert_eq!(
//!     Schedule!(0).all(),
//!     Portions!());
//! // sibling pools
//! assert_eq!(
//!     Schedule!(0 (P0 0) (P1 0)).all(),
//!     Portions!());
//! // sibling accounts inside a pool
//! assert_eq!(
//!     Schedule!(0 (P0 0 (C0 0 (Alice 0)) (C1 0 (Bob 0)))).all(),
//!     Portions!());
//! // now let's try a populated schedule
//! assert_eq!(
//!     Schedule!(300 (P0 100 (C00 50 (Alice 50))
//!                           (C01 50 (Bob 25) (Candy 25)))
//!                   (P1 200 (C10 100 head 20     at 5 (Alice 12) (Bob 8)
//!                                    body 30 every 30 (Alice 15) (Bob 15)
//!                                    tail 20          (Candy 20))
//!                           (C11 100 head 20     at 5 (Alice 12) (Bob 8)
//!                                    body 30 every 30 (Alice 15) (Bob 15)
//!                                    tail 20          (Candy 20)))).all(),
//!     Portions!(
//!          [  0  Alice  50  "C00: head" ]
//!          [  0  Bob    25  "C01: head" ]
//!          [  0  Candy  25  "C01: head" ]
//!
//!          [  5  Alice  12  "C10: head" ]
//!          [  5  Bob     8  "C10: head" ]
//!          [ 35  Alice  15  "C10: body" ]
//!          [ 35  Bob    15  "C10: body" ]
//!          [ 65  Alice  15  "C10: body" ]
//!          [ 65  Bob    15  "C10: body" ]
//!          [ 65  Candy  20  "C10: tail" ]
//!
//!          [  5  Alice  12  "C11: head" ]
//!          [  5  Bob     8  "C11: head" ]
//!          [ 35  Alice  15  "C11: body" ]
//!          [ 35  Bob    15  "C11: body" ]
//!          [ 65  Alice  15  "C11: body" ]
//!          [ 65  Bob    15  "C11: body" ]
//!          [ 65  Candy  20  "C11: tail" ] ));
//!
//! # }
//! ```

/// `100` -> `Uint128::from(100u128)`
#[macro_export] macro_rules! U128 {
    ($x:literal) => { cosmwasm_std::Uint128::from($x as u128) }
}

/// `Foo` -> `String::from("Foo")`
#[macro_export] macro_rules! Str {
    ($x:ident) => { String::from(stringify!($x)) }
}

/// Create a `Account` from a short description
#[macro_export] macro_rules! Account {
    ( (
        ($name:ident $address:ident)
        (amount $amount:expr)
        (start $start_at:expr)
        (cliff $cliff:expr)
        (interval $interval:expr)
        (duration $duration:expr)
    ) ) => { $crate::Account {
        name:     Str!($name),
        address:  $address.clone(),
        amount:   U128!($amount),
        cliff:    U128!($cliff)
        start_at: $start_at,
        interval: $interval,
        duration: $duration,
    } };
}

/// Create a `Schedule`->`Pool`s->`Account`s tree
/// from a short description
#[macro_export] macro_rules! Schedule {
    ( $total:literal $((
        $pool:ident $pool_total:literal
        $($account:tt)*
    ) )* ) => { $crate::Schedule {
        total:        U128!($total),
        pools:        vec![$( $crate::Pool {
            name:     Str!($pool),
            total:    U128!($pool_total),
            partial:  true,
            accounts: vec![$(Account!($account),)*]
        } ),*]
    } }
}

/// Create a `Result<Vec<Portion>>` from a short description
#[macro_export] macro_rules! Portions {
    ($([$t:literal $addr:ident $amount:literal $reason:literal])*) => {
        Ok(vec![$($crate::Portion {
            vested:  $t,
            address: $addr.clone(),
            amount:  cosmwasm_std::Uint128::from($amount as u128),
            reason:  $reason.to_string()
        }),*])
    };
}

/// Create an error result
#[macro_export] macro_rules! Error {
    ($msg:expr) => {
        Err(cosmwasm_std::StdError::GenericErr {
            msg: $msg.to_string(),
            backtrace: Some(snafu::Backtrace::generate())
        })
    };
}

/// Define error conditions (inside and `impl` block)
#[macro_export] macro_rules! define_errors {
    ($(
        $name:ident ($(&$self:ident,)? $($arg:ident : $type:ty),*) ->
        ($format:literal $(, $var:expr)*)
    )+) => {
        $(
            #[doc=$format]
            pub fn $name<T> ($(&$self,)? $($arg : $type),*) -> StdResult<T> {
                Error!(format!($format $(, $var)*))
            }
        )+
    }
}
