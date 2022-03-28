use amm_shared::{
    fadroma::{
        ContractLink,
        cosmwasm_std::{
            HumanAddr, Uint128, StdError, 
        },
        ensemble::MockEnv,
        Decimal256
    },
    TokenPairAmount, TokenTypeAmount,
    msg
};

use crate::setup::{LaunchpadIdo};

#[test]
fn launchpad_ido_init() {
   let lpd = LaunchpadIdo::new();
}