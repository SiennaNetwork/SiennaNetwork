use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use snip20_sienna::msg as snip20;

fn main() {
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
}

