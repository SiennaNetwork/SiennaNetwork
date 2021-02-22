//! Below is such an example, demonstrating most features of this crate;
//! including the usage of `.all()` to get the resulting list of transcations.
//!
//! ```rust
//! # #[allow(non_snake_case)]
//! # #[macro_use] extern crate sienna_schedule; use sienna_schedule::Vesting;
//! # fn main() {
//! let Alice = cosmwasm_std::HumanAddr::from("Alice");
//! let Bob   = cosmwasm_std::HumanAddr::from("Bob");
//! let Candy = cosmwasm_std::HumanAddr::from("Candy");
//! let S = schedule!(300 => (
//!     P0(100) = (
//!         // these two channels vest everything immediately upon launch
//!         C00(50) = (
//!             Alice => 50
//!         )
//!         C01(50) = (
//!             Bob   => 25
//!             Candy => 25
//!         )
//!     )
//!     P1(200) = (
//!         // this channel vests portions periodically after a cliff
//!         C10(100) = (
//!             cliff(20 at 5) = (
//!                 Alice => 12
//!                 Bob   =>  8
//!             )
//!             regular(30 every 30) = (
//!                 Alice => 15
//!                 Bob   => 15
//!             ),
//!             remainder(20) = (
//!                 Candy => 20
//!             )
//!         )
//!         // TODO: this channel has its allocations updated halfway
//!     )
//! ));
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
/// create `Schedule` w/ (`Pool`s w/ (`Channel`s w/ `Periodic`s & (`AllocationSet`s w/ `Allocation`s)))
#[macro_export] macro_rules! schedule {
    ($total:expr => ($(
        $pool:ident ( $pool_total:expr ) = ($(
            $channel:ident ( $channel_total:expr ) = $allocations:tt
        )+)
    )+)) => {
        sienna_schedule::Schedule {
            total: cosmwasm_std::Uint128::from($total as u128),
            pools: vec![$(
                sienna_schedule::Pool {
                    name: String::from(stringify!($pool)),
                    total: cosmwasm_std::Uint128::from($pool_total as u128),
                    partial: true,
                    channels: vec![$(
                        sienna_schedule::Channel {
                            name: String::from(stringify!($channel)),
                            amount: cosmwasm_std::Uint128::from($channel_total as u128),
                            periodic: None,
                            allocations: vec![]
                        }
                    ),+]
                }
            ),+]
        }
    };
    //(@allocations (
        //cliff ($cliff:literal at $start_at:literal) = $cliff_alloc:tt
        //regular ($portion:literal every $interval:literal) = $regular_alloc:tt
        //remainder ($remainder:literal) = $remainder_alloc:tt
    //)) => {};
    //(@allocations (
        //$who:expr => $how_much:expr
    //)) => {};
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
