//! This module defines the vesting schedule and recipients.
//!
//! The `SIENNA!` macro (defined in `helpers.rs`; also used by
//! `monthly!`/`daily!`/`immediate!`) automatically multiplies
//! the amounts by 10^18 (`ONE_SIENNA`, defined in `types.rs`)

use crate::constants::{ONE_SIENNA, MONTH};
use crate::types::{Schedule, Stream, Vesting, Interval};
use cosmwasm_std::{HumanAddr, Uint128};

/// Addresses.
/// Internally, the vesting macros call `recipient!(Something)`
/// to establish the recipient addresses at compile time.
macro_rules! recipient {
    (DevFund)   => { HumanAddr::from("secret1DevFund")   };
    (Investors) => { HumanAddr::from("secret1Investors") };
    (Founder1)  => { HumanAddr::from("secret1Founder1")  };
    (Founder2)  => { HumanAddr::from("secret1Founder2")  };
    (Founder3)  => { HumanAddr::from("secret1Founder3")  };
    (Founder4)  => { HumanAddr::from("secret1Founder4")  };
    (Advisor1)  => { HumanAddr::from("secret1Advisor1")  };
    (Advisor2)  => { HumanAddr::from("secret1Advisor2")  };
    (Advisor3)  => { HumanAddr::from("secret1Advisor3")  };
    (Advisor4)  => { HumanAddr::from("secret1Advisor4")  };
    (AdvisorR)  => { HumanAddr::from("secret1AdvisorR")  };
    (Liquidity) => { HumanAddr::from("secret1Liquidity") };
    (Remaining) => { HumanAddr::from("secret1Remaining") };
    () => {};
}

lazy_static! {
    /// The hardcoded token vesting schedule.
    pub static ref SCHEDULE: Schedule = Schedule {

        total: SIENNA!(10000000),

        predefined: vec! [
            monthly!   (DevFund   1500000 5 18  0%),

            daily!     (Investors 2000000 6 16  0%),
            daily!     (Founder1   897000 6 16 10%),
            daily!     (Founder2   897000 6 16 10%),
            daily!     (Founder3   437000 6 16 10%),
            daily!     (Founder4    69000 6 16 10%),

            daily!     (Advisor1    50000 6 16  0%),
            daily!     (Advisor2    50000 6 16  0%),
            daily!     (Advisor3    10000 6  6  0%),
            daily!     (Advisor4     5000 6  6  0%),
            immediate! (AdvisorR    85000         ),

            immediate! (Liquidity  300000         ),

            //immediate! (Remaining 3700000         ),
            // configurable, see below:
        ],

        configurable:     SIENNA!(3700000),
        configurable_daily:  SIENNA!(2500),

    };
}
