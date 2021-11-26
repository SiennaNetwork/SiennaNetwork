use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use amm_shared::msg::exchange;
use amm_shared::msg::factory;
use amm_shared::msg::ido;
use amm_shared::msg::launchpad;
use amm_shared::msg::router;
use amm_shared::msg::snip20;

use sienna_mgmt::msg as mgmt;
use sienna_rewards::msg as rewards;
use sienna_rewards_emergency_proxy::msg as rewards_emergency_proxy;
use sienna_rpt::msg as rpt;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("mgmt");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(mgmt::Init), &out_dir);
    export_schema(&schema_for!(mgmt::Handle), &out_dir);
    export_schema(&schema_for!(mgmt::Query), &out_dir);
    export_schema(&schema_for!(mgmt::Response), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("rpt");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(rpt::Init), &out_dir);
    export_schema(&schema_for!(rpt::Handle), &out_dir);
    export_schema(&schema_for!(rpt::Query), &out_dir);
    export_schema(&schema_for!(rpt::Response), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("amm");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(exchange::InitMsg), &out_dir);
    export_schema(&schema_for!(exchange::HandleMsg), &out_dir);
    export_schema(&schema_for!(exchange::QueryMsg), &out_dir);
    export_schema(&schema_for!(exchange::QueryMsgResponse), &out_dir);
    export_schema(&schema_for!(exchange::ReceiverCallbackMsg), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("factory");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(factory::InitMsg), &out_dir);
    export_schema(&schema_for!(factory::HandleMsg), &out_dir);
    export_schema(&schema_for!(factory::QueryMsg), &out_dir);
    export_schema(&schema_for!(factory::QueryResponse), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("launchpad");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(launchpad::InitMsg), &out_dir);
    export_schema(&schema_for!(launchpad::HandleMsg), &out_dir);
    export_schema(&schema_for!(launchpad::QueryMsg), &out_dir);
    export_schema(&schema_for!(launchpad::QueryResponse), &out_dir);
    export_schema(&schema_for!(launchpad::ReceiverCallbackMsg), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("ido");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(ido::InitMsg), &out_dir);
    export_schema(&schema_for!(ido::HandleMsg), &out_dir);
    export_schema(&schema_for!(ido::QueryMsg), &out_dir);
    export_schema(&schema_for!(ido::QueryResponse), &out_dir);
    export_schema(&schema_for!(ido::ReceiverCallbackMsg), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("snip20");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(snip20::InitMsg), &out_dir);
    export_schema(&schema_for!(snip20::HandleMsg), &out_dir);
    export_schema(&schema_for!(snip20::HandleAnswer), &out_dir);
    export_schema(&schema_for!(snip20::QueryMsg), &out_dir);
    export_schema(&schema_for!(snip20::QueryAnswer), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("exchange");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(exchange::InitMsg), &out_dir);
    export_schema(&schema_for!(exchange::HandleMsg), &out_dir);
    export_schema(&schema_for!(exchange::QueryMsg), &out_dir);
    export_schema(&schema_for!(exchange::QueryMsgResponse), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("rewards");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(rewards::Init), &out_dir);
    export_schema(&schema_for!(rewards::Handle), &out_dir);
    export_schema(&schema_for!(rewards::Query), &out_dir);
    export_schema(&schema_for!(rewards::Response), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("router");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(router::InitMsg), &out_dir);
    export_schema(&schema_for!(router::HandleMsg), &out_dir);
    export_schema(&schema_for!(router::QueryMsg), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("rewards_emergency_proxy");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(rewards_emergency_proxy::Init), &out_dir);
    export_schema(&schema_for!(rewards_emergency_proxy::Handle), &out_dir);
    export_schema(&schema_for!(rewards_emergency_proxy::Query), &out_dir);
    export_schema(&schema_for!(rewards_emergency_proxy::Response), &out_dir);
}
