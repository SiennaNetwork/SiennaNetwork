use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use sienna_rpt::msg;
use sienna_rpt::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(State),         &out_dir);
    export_schema(&schema_for!(msg::Init),     &out_dir);
    export_schema(&schema_for!(msg::Handle),   &out_dir);
    export_schema(&schema_for!(msg::Query),    &out_dir);
    export_schema(&schema_for!(msg::Response), &out_dir);
}
