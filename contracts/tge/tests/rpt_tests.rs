use sienna_mgmt::ConfigResponse;

use crate::setup::TGE;

#[test]
fn test_rpt_init() {
    let tge = TGE::default();

    let _: ConfigResponse = tge
        .ensemble
        .query(tge.mgmt.address, sienna_mgmt::QueryMsg::Config {})
        .unwrap();
}
