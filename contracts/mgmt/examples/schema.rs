use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use sienna_mgmt::msg as mgmt;

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
}
