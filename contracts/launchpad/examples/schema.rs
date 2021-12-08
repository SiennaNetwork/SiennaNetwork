use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use amm_shared::msg::launchpad;

fn main() {
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
}
