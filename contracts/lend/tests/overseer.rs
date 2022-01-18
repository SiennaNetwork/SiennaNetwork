use lend_shared::{
    fadroma::{
        ensemble::MockEnv,
        Decimal256, StdError,
    },
    interfaces::{overseer::*}
};

use crate::setup::Lend;
use crate::ADMIN;

#[test]
fn whitelist() {
    let mut lend = Lend::new();

    lend.ensemble
        .execute(
            &HandleMsg::Whitelist {
                market: Market {
                    contract: lend.market.clone(),
                    symbol: "SIENNA".into(),
                    ltv_ratio: Decimal256::percent(90),
                },
            },
            MockEnv::new(ADMIN, lend.overseer.clone()),
        )
        .unwrap();

    let res = lend.ensemble.execute(
        &HandleMsg::Whitelist {
            market: Market {
                contract: lend.market,
                symbol: "SIENNA".into(),
                ltv_ratio: Decimal256::percent(90),
            },
        },
        MockEnv::new(ADMIN, lend.overseer),
    );

    assert_eq!(
        res.unwrap_err(),
        StdError::generic_err("Token is already registered as collateral.")
    );
}
