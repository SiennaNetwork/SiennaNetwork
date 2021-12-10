use amm_shared::fadroma::{export_schema, remove_schemas, schema_for};
use std::fs::create_dir_all;

use amm_shared::msg::exchange;

fn main() {
    let mut out_dir = std::path::PathBuf::from(file!());
    out_dir.pop();
    out_dir.pop();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(exchange::InitMsg),             &out_dir);
    export_schema(&schema_for!(exchange::HandleMsg),           &out_dir);
    export_schema(&schema_for!(exchange::QueryMsg),            &out_dir);
    export_schema(&schema_for!(exchange::QueryMsgResponse),    &out_dir);
    export_schema(&schema_for!(exchange::ReceiverCallbackMsg), &out_dir);
}
