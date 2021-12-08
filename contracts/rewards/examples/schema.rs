use fadroma::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use sienna_rewards as rewards;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("rewards");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(rewards::Init), &out_dir);
    export_schema(&schema_for!(rewards::Handle), &out_dir);
    export_schema(&schema_for!(rewards::Query), &out_dir);
    export_schema(&schema_for!(rewards::Response), &out_dir);
}
