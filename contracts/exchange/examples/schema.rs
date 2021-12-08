use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::{env::current_dir, fs::create_dir_all};

use amm_shared::msg::exchange;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("amm");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(exchange::InitMsg), &out_dir);
    export_schema(&schema_for!(exchange::HandleMsg), &out_dir);
    export_schema(&schema_for!(exchange::QueryMsg), &out_dir);
    export_schema(&schema_for!(exchange::QueryMsgResponse), &out_dir);
    export_schema(&schema_for!(exchange::ReceiverCallbackMsg), &out_dir);
}
