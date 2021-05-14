use std::{env::current_dir,fs::create_dir_all};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use sienna_amm_shared::msg::exchange as exchange;
use sienna_amm_shared::msg::factory as factory;
use sienna_amm_shared::msg::sienna_burner as burner;
use lp_token::msg as lp_token;
use snip20_reference_impl::msg as snip20;

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

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("factory");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(factory::InitMsg), &out_dir);
    export_schema(&schema_for!(factory::HandleMsg), &out_dir);
    export_schema(&schema_for!(factory::QueryMsg), &out_dir);
    export_schema(&schema_for!(factory::QueryResponse), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("burner");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(burner::InitMsg), &out_dir);
    export_schema(&schema_for!(burner::HandleMsg), &out_dir);
    export_schema(&schema_for!(burner::QueryMsg), &out_dir);
    export_schema(&schema_for!(burner::QueryAnswer), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("lp_token");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(sienna_amm_shared::msg::snip20::Snip20InitMsg), &out_dir);
    export_schema(&schema_for!(lp_token::HandleMsg), &out_dir);
    export_schema(&schema_for!(lp_token::HandleAnswer), &out_dir);
    export_schema(&schema_for!(lp_token::QueryMsg), &out_dir);
    export_schema(&schema_for!(lp_token::QueryAnswer), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("snip20");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(snip20::InitMsg), &out_dir);
    export_schema(&schema_for!(snip20::HandleMsg), &out_dir);
    export_schema(&schema_for!(snip20::HandleAnswer), &out_dir);
    export_schema(&schema_for!(snip20::QueryMsg), &out_dir);
    export_schema(&schema_for!(snip20::QueryAnswer), &out_dir);

}
