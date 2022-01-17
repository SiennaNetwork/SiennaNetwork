use fadroma::{export_schema, remove_schemas, schema_for};
use std::fs::create_dir_all;

use sienna_rewards as rewards;

fn main() {
    let mut out_dir = std::path::PathBuf::from(file!());
    out_dir.pop();
    out_dir.pop();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(rewards::Init), &out_dir);
    export_schema(&schema_for!(rewards::Handle), &out_dir);
    export_schema(&schema_for!(rewards::Query), &out_dir);
    export_schema(&schema_for!(rewards::Response), &out_dir);
}
