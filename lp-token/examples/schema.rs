use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use lp_token::msg::{HandleAnswer, HandleMsg, QueryAnswer, QueryMsg};
use shared::LpTokenInitMsg;

fn main() {
    let ref mut out_dir = current_dir().unwrap();

    out_dir.push("amm-lp-token");
    out_dir.push("schema");

    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(LpTokenInitMsg), out_dir);
    export_schema(&schema_for!(HandleMsg), out_dir);
    export_schema(&schema_for!(HandleAnswer), out_dir);
    export_schema(&schema_for!(QueryMsg), out_dir);
    export_schema(&schema_for!(QueryAnswer), out_dir);
}
