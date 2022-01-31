use lend_shared::fadroma::{export_schema, remove_schemas, schema_for};
use std::fs::create_dir_all;

use lend_shared::interfaces::market::{InitMsg, QueryMsg, HandleMsg};

fn main() {
    let mut out_dir = std::path::PathBuf::from(file!());
    out_dir.pop();
    out_dir.pop();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InitMsg), &out_dir);
    export_schema(&schema_for!(HandleMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
}
