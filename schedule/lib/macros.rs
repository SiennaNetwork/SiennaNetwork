//! Macros for internal use
//!
//! Example of using `schedule!` and `portions!` to materialize a schedule.
//!
//! ```rust
//! # #[macro_use] extern crate sienna_schedule; use sienna_schedule::Vesting;
//! # #[allow(non_snake_case)]
//! # fn main() {
//! let Alice = cosmwasm_std::HumanAddr::from("Alice");
//! let Bob   = cosmwasm_std::HumanAddr::from("Bob");
//! let Candy = cosmwasm_std::HumanAddr::from("Candy");
//! let S = schedule!(0 => []);
//! let S = schedule!(0 => [ P0(0) => [] ]);
//! let S = schedule!(0 => [ P0(0) => [ C0(0) => [] ] ]);
//! let S = schedule!(300 => [
//!     P0(100) => [
//!         C00(50) => [ Alice => 50 ]
//!         C01(50) => [ Bob => 25; Candy => 25 ]
//!     ]
//!     // one channel vests portions periodically after a cliff
//!     // TODO: this channel has its allocations altered halfway
//!     P1(200) => [
//!         C10(100) => [
//!             0: [
//!                 cliff(20 at 5)       => [ Alice => 12; Bob =>  8 ]
//!                 regular(30 every 30) => [ Alice => 15; Bob => 15 ]
//!                 remainder(20)        => [ Candy => 20 ]
//!             ]
//!         ]
//!     ]
//! ]);
//! assert_eq!(S.all(), Ok(portions!(
//!     [  0  Alice  50  "C00: immediate"]
//!     [  0  Bob    25  "C01: immediate"]
//!     [  0  Candy  25  "C01: immediate"]
//!
//!     [  5  Alice  12  "C10: cliff"    ]
//!     [  5  Bob     8  "C10: cliff"    ]
//!     [ 35  Alice  15  "C10: vesting"  ]
//!     [ 35  Bob    15  "C10: vesting"  ]
//!     [ 65  Alice  15  "C10: vesting"  ]
//!     [ 65  Bob    15  "C10: vesting"  ]
//!     [ 65  Candy  20  "C10: remainder"]
//! )))
//! # }
//! ```

#[macro_export] macro_rules! U128 {
    ($x:expr) => { cosmwasm_std::Uint128::from($x as u128) }
}

#[macro_export] macro_rules! string {
    ($x:expr) => { String::from(stringify!($x)) }
}

#[macro_export] macro_rules! portions {
    ($([$t:literal $addr:ident $amount:literal $reason:literal])+) => {
        vec![$(sienna_schedule::Portion {
            vested:  $t,
            address: $addr.clone(),
            amount:  cosmwasm_std::Uint128::from($amount as u128),
            reason:  $reason.to_string()
        }),+]
    };
}

/// instantiate the described schedule
#[macro_export] macro_rules! schedule {
    ( $total:expr => [ $( $pool:ident ( $pool_total:expr ) => [ $(
        $channel:ident ( $channel_total:expr ) => [ $($allocations:tt)* ]
    )* ] )* ] ) => {
        sienna_schedule::Schedule {
            total:        U128!($total),
            pools:        vec![$( sienna_schedule::Pool {
                name:     string!($pool),
                total:    U128!($pool_total),
                partial:  true,
                channels: vec![$(
                    channel!($channel ($channel_total) => [$($allocations)*])
                ),*]
            } ),*]
        }
    };
}

#[macro_export] macro_rules! channel {
    ($name:ident ($total:expr) => []) => {
        sienna_schedule::Channel {
            name: string!($name),
            amount: U128!($total),
            periodic: None,
            allocations: vec![]
        }
    };
    ($name:ident ($total:expr) => [ $($t:literal : [
        cliff ($cliff:literal at $start_at:literal)
            => [ $($cliff_allocations:tt)* ]
        regular ($portion:literal every $interval:literal)
            => [ $($regular_allocations:tt)* ]
        remainder ($remainder:literal)
            => [ $($remainder_allocations:tt)* ]
    ] ),* ] ) => {
        sienna_schedule::Channel {
            name:        string!($name),
            amount:      U128!($total),
            periodic:    sienna_schedule::Periodic {
                start_at: $start_at
                cliff:    U128!($cliff),
                interval: $interval,
                duration: 0
            },
            allocations: vec![$(sienna_schedule::AllocationSet {
                t: 0,
                cliff: allocations!($($cliff_allocations)*),
                regular: allocations!($($regular_allocations)*),
                remainder: allocations!($($remainder_allocations)*)
            }),*]
        }
    };
    ($name:ident ($total:expr) => [ $($who:expr => $how_much:expr);* ] ) => {
        sienna_schedule::Channel {
            name:        string!($name),
            amount:      U128!($total),
            periodic:    None,
            allocations: vec![sienna_schedule::AllocationSet {
                t: 0,
                cliff: vec![],
                regular: allocations!($($who => $how_much)*),
                remainder: vec![]
            }]
        }
    };
}           

#[macro_export] macro_rules! allocations {
    ($($who:expr => $how_much:expr)+) => {
        vec![$(sienna_schedule::Allocation { amount: $who.clone(), address: U128!($how_much) }),+]
    }
}

/// error result constructor
#[macro_export] macro_rules! Error {
    ($msg:expr) => {
        Err(cosmwasm_std::StdError::GenericErr { msg: $msg.to_string(), backtrace: None })
    };
}

/// define error conditions with corresponding parameterized messages
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
