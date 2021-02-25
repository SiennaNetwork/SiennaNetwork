//! Macros for internal use
//!
//! Example of using `schedule!` and `portions!` to materialize a schedule.
//!
//! ```rust
//! # #[macro_use] extern crate sienna_schedule; use sienna_schedule::Vesting;
//! # #[allow(non_snake_case)]
//! # fn main() {
//! // some imaginary people:
//! let Alice = cosmwasm_std::HumanAddr::from("Alice");
//! let Bob   = cosmwasm_std::HumanAddr::from("Bob");
//! let Candy = cosmwasm_std::HumanAddr::from("Candy");
//! // some empty, but valid schedules
//! assert_eq!(
//!     Schedule!(0).all(),
//!     Portions!());
//! // sibling pools
//! assert_eq!(
//!     Schedule!(0 (P0 0) (P1 0)).all(),
//!     Portions!());
//! // sibling channels inside a pool
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
//!          [  0  Alice  50  "C00: head"    ]
//!          [  0  Bob    25  "C01: head"    ]
//!          [  0  Candy  25  "C01: head"    ]
//!
//!          [  5  Alice  12  "C10: head"    ]
//!          [  5  Bob     8  "C10: head"    ]
//!          [ 35  Alice  15  "C10: vesting" ]
//!          [ 35  Bob    15  "C10: vesting" ]
//!          [ 65  Alice  15  "C10: vesting" ]
//!          [ 65  Bob    15  "C10: vesting" ]
//!          [ 65  Candy  20  "C10: tail"    ]
//!
//!          [  5  Alice  12  "C11: head"    ]
//!          [  5  Bob     8  "C11: head"    ]
//!          [ 35  Alice  15  "C11: vesting" ]
//!          [ 35  Bob    15  "C11: vesting" ]
//!          [ 65  Alice  15  "C11: vesting" ]
//!          [ 65  Bob    15  "C11: vesting" ]
//!          [ 65  Candy  20  "C11: tail"    ] ));
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

/// Create a `Channel` from a short description
#[macro_export] macro_rules! Channel {
    ( (
        $name:ident $total:literal
        $(($who:ident $how_much:literal))*
    ) ) => { sienna_schedule::Channel {
        name:             Str!($name),
        total:            U128!($total),
        start_at:         0,
        interval:         0,
        duration:         0,
        head:             U128!(0),
        head_allocations: vec![$(sienna_schedule::Allocation{address:$who.clone(),amount:U128!($how_much)}),*],
        body_allocations: vec![],
        tail_allocations: vec![]
    } };
    ( (
        $name:ident $total:literal
        head    $head:literal    at $start_at:literal $(($who1:ident $how_much1:literal))*
        body $portion:literal every $interval:literal $(($who2:ident $how_much2:literal))*
        tail    $tail:literal                         $(($who3:ident $how_much3:literal))*
    ) ) => { sienna_schedule::Channel {
        name:             Str!($name),
        total:            U128!($total),
        start_at:         $start_at,
        interval:         $interval,
        duration:         0, // TODO
        head:             U128!($head),
        head_allocations: vec![$(sienna_schedule::Allocation{address:$who1.clone(),amount:U128!($how_much1)}),*],
        body_allocations: vec![$(sienna_schedule::Allocation{address:$who2.clone(),amount:U128!($how_much2)}),*],
        tail_allocations: vec![$(sienna_schedule::Allocation{address:$who3.clone(),amount:U128!($how_much3)}),*],
    } };
}

/// Create a `Schedule`->`Pool`s->`Channel`s tree
/// from a short description
#[macro_export] macro_rules! Schedule {
    ( $total:literal $((
        $pool:ident $pool_total:literal
        $($channel:tt)*
    ) )* ) => { sienna_schedule::Schedule {
        total:        U128!($total),
        pools:        vec![$( sienna_schedule::Pool {
            name:     Str!($pool),
            total:    U128!($pool_total),
            partial:  true,
            channels: vec![$(Channel!($channel),)*]
        } ),*]
    } }
}

/// Create a `Result<Vec<Portion>>` from a short description
#[macro_export] macro_rules! Portions {
    ($([$t:literal $addr:ident $amount:literal $reason:literal])*) => {
        Ok(vec![$(sienna_schedule::Portion {
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
        Err(cosmwasm_std::StdError::GenericErr { msg: $msg.to_string(), backtrace: None })
    };
}

/// Inside `impl`, define error cases and messages
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
