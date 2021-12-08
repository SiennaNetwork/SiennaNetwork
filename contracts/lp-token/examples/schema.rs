use amm_shared::fadroma::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use amm_shared::msg::snip20;

fn main() {
    let mut out_dir = std::path::PathBuf::from(file!());
    out_dir.pop();
    out_dir.pop();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(snip20::InitMsg), &out_dir);
    export_schema(&schema_for!(snip20::HandleMsg), &out_dir);
    export_schema(&schema_for!(snip20::HandleAnswer), &out_dir);
    export_schema(&schema_for!(snip20::QueryMsg), &out_dir);
    export_schema(&schema_for!(snip20::QueryAnswer), &out_dir);
}
