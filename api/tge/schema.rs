use std::env::current_dir;
use std::fs::create_dir_all;
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use snip20_reference_impl::msg as token;
use sienna_mgmt::msg as mgmt;
use sienna_rpt::msg as rpt;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("token");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(token::InitMsg), &out_dir);
    export_schema(&schema_for!(token::HandleMsg), &out_dir);
    export_schema(&schema_for!(token::HandleAnswer), &out_dir);
    export_schema(&schema_for!(token::QueryMsg), &out_dir);
    export_schema(&schema_for!(token::QueryAnswer), &out_dir);

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
}
