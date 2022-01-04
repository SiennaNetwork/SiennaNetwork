use amm_shared::fadroma::{export_schema, remove_schemas, schema_for};
use std::fs::create_dir_all;

use amm_shared::msg::router as msg;

fn main() {
    let mut out_dir = std::path::PathBuf::from(file!());
    out_dir.pop();
    out_dir.pop();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(msg::InitMsg), &out_dir);
    export_schema(&schema_for!(msg::HandleMsg), &out_dir);
    export_schema(&schema_for!(msg::QueryMsg), &out_dir);
}
