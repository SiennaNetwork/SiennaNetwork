use crate::types::*;
use cosmwasm_std::{HumanAddr, Uint128};

/// Addresses
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
    (Remaining) => { HumanAddr::from("secret1Remaining") };
    () => {};
}

lazy_static! {
    pub static ref SCHEDULE: Schedule = Schedule {

        total: SIENNA!(10000000),

        configurable: SIENNA!(300000),
        configurable_daily: SIENNA!(2500),

        predefined: vec! [
            monthly!   (DevFund   1500000 20 4  5%),

            daily!     (Investors 2000000 16 6  0%),
            daily!     (Founder1   897000 16 6 10%),
            daily!     (Founder2   897000 16 6 10%),
            daily!     (Founder3   437000 16 6 10%),
            daily!     (Founder4    69000 16 6 10%),

            daily!     (Advisor1    50000 16 6  0%),
            daily!     (Advisor2    50000 16 6  0%),
            daily!     (Advisor3    10000  6 6  0%),
            daily!     (Advisor4     5000  6 6  0%),

            immediate! (AdvisorR    85000         ),
            immediate! (Remaining 3700000         ),
        ]

    };
}
