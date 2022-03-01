use amm_shared::fadroma::{export_schema, remove_schemas, schema_for};
use std::fs::create_dir_all;

use amm_shared::msg::ido;

fn main() {
    let mut out_dir = std::path::PathBuf::from(file!());
    out_dir.pop();
    out_dir.pop();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(ido::InitMsg), &out_dir);
    export_schema(&schema_for!(ido::HandleMsg), &out_dir);
    export_schema(&schema_for!(ido::QueryMsg), &out_dir);
    export_schema(&schema_for!(ido::QueryResponse), &out_dir);
    export_schema(&schema_for!(ido::ReceiverCallbackMsg), &out_dir);
}
