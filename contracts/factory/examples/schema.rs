use amm_shared::fadroma::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use amm_shared::msg::factory;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("factory");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(factory::InitMsg), &out_dir);
    export_schema(&schema_for!(factory::HandleMsg), &out_dir);
    export_schema(&schema_for!(factory::QueryMsg), &out_dir);
    export_schema(&schema_for!(factory::QueryResponse), &out_dir);
}
