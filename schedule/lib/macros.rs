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
//! let S = Schedule!(0);
//! let S = Schedule!(0
//!     (P0 0)
//!     (P1 0));
//! let S = Schedule!(0
//!     (P0 0 (C0 0)));
//! let S = Schedule!(0
//!     (P0 0 (C0 0 (Alice 0) (Bob 0))
//!           (C1 0)));
//! let S = Schedule!(0
//!     (P0 0 (C0 0 (T=0 cliff     0    at 0 (Alice 0) (Bob 0)
//!                      regular   0 every 0 (Alice 0) (Bob 0)
//!                      remainder 0         (Alice 0) (Bob 0)))
//!           (C1 0 (T=0 cliff     0    at 0 (Alice 0) (Bob 0)
//!                      regular   0 every 0 (Alice 0) (Bob 0)
//!                      remainder 0         (Alice 0) (Bob 0))
//!                 (T=1 cliff     0    at 0 (Alice 0) (Bob 0)
//!                      regular   0 every 0 (Alice 0) (Bob 0)
//!                      remainder 0         (Alice 0) (Bob 0)))));
//! // now let's try a populated schedule
//! let S = Schedule!(300
//!     (P0 100 (C00 50 (Alice 50))
//!             (C01 50 (Bob 25) (Candy 25)))
//!     (P1 200 (C10 100  (T=0 cliff     20     at 5 (Alice 12) (Bob 8)
//!                            regular   30 every 30 (Alice 15) (Bob 15)
//!                            remainder 20          (Candy 20)))
//!             (C11 100  (T=0 cliff     20     at 5 (Alice 12) (Bob 8)
//!                            regular   30 every 30 (Alice 15) (Bob 15)
//!                            remainder 20          (Candy 20)))));
//! assert_eq!(S.all(), Ok(Portions!(
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
//!
//!     [  5  Alice  12  "C11: cliff"    ]
//!     [  5  Bob     8  "C11: cliff"    ]
//!     [ 35  Alice  15  "C11: vesting"  ]
//!     [ 35  Bob    15  "C11: vesting"  ]
//!     [ 65  Alice  15  "C11: vesting"  ]
//!     [ 65  Bob    15  "C11: vesting"  ]
//!     [ 65  Candy  20  "C11: remainder"]
//! )))
//! # }
//! ```

#[macro_export] macro_rules! U128 {
    ($x:expr) => { cosmwasm_std::Uint128::from($x as u128) }
}

#[macro_export] macro_rules! string {
    ($x:expr) => { String::from(stringify!($x)) }
}

#[macro_export] macro_rules! Portions {
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
#[macro_export] macro_rules! Schedule {
    ( $total:literal
        $( ( $pool:ident $pool_total:literal
            $( ( $channel:ident $channel_total:literal
                $( $allocations:tt )* ) )* ) )* ) => {
        sienna_schedule::Schedule {
            total:        U128!($total),
            pools:        vec![$( sienna_schedule::Pool {
                name:     string!($pool),
                total:    U128!($pool_total),
                partial:  true,
                channels: vec![$(
                    Channel!($channel $channel_total $($allocations)*)
                ),*]
            } ),*]
        }
    };
}

#[macro_export] macro_rules! Channel {
    ($name:ident $total:literal) => {
        sienna_schedule::Channel {
            name: string!($name),
            amount: U128!($total),
            periodic: None,
            allocations: vec![]
        }
    };
    ($name:ident $total:literal $(($who:ident $how_much:literal))*) => {
        sienna_schedule::Channel {
            name:        string!($name),
            amount:      U128!($total),
            periodic:    None,
            allocations: vec![sienna_schedule::AllocationSet {
                t: 0,
                cliff: vec![],
                regular: Allocations!($($who => $how_much)*),
                remainder: vec![]
            }]
        }
    };
    ($name:ident $total:literal $( ( T= $t:literal
        cliff $cliff:literal at $start_at:literal
        $(($who1:ident $how_much1:literal))*
        regular $portion:literal every $interval:literal
        $(($who2:ident $how_much2:literal))*
        remainder $remainder:literal
        $(($who3:ident $how_much3:literal))*
    ) )*) => {
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
                cliff:     Allocations!($($who1 => $how_much1)*),
                regular:   Allocations!($($who2 => $how_much2)*),
                remainder: Allocations!($($who3 => $how_much3)*)
            }),*]
        }
    };
}           

#[macro_export] macro_rules! Allocations {
    ($($who:expr => $how_much:expr)+) => {
        vec![$(sienna_schedule::Allocation { address: $who.clone(), amount: U128!($how_much) }),+]
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
