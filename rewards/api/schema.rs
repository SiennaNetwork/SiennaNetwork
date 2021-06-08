use std::env::current_dir;
use std::fs::create_dir_all;
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use gov_token;
use lp_staking;
use weight_master;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("gov-token");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(gov_token::msg::InitMsg), &out_dir);
    export_schema(&schema_for!(gov_token::msg::HandleMsg), &out_dir);
    export_schema(&schema_for!(gov_token::msg::HandleAnswer), &out_dir);
    export_schema(&schema_for!(gov_token::msg::QueryMsg), &out_dir);
    export_schema(&schema_for!(gov_token::msg::QueryAnswer), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("lp-staking");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(lp_staking::msg::LPStakingInitMsg), &out_dir);
    export_schema(&schema_for!(lp_staking::msg::LPStakingHandleMsg), &out_dir);
    export_schema(&schema_for!(lp_staking::msg::LPStakingHandleAnswer), &out_dir);
    export_schema(&schema_for!(lp_staking::msg::LPStakingQueryMsg), &out_dir);
    export_schema(&schema_for!(lp_staking::msg::LPStakingQueryAnswer), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("weight-master");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(weight_master::MasterInitMsg), &out_dir);
    export_schema(&schema_for!(weight_master::MasterHandleMsg), &out_dir);
    export_schema(&schema_for!(weight_master::MasterHandleAnswer), &out_dir);
    export_schema(&schema_for!(weight_master::MasterQueryMsg), &out_dir);
    export_schema(&schema_for!(weight_master::MasterQueryAnswer), &out_dir);
}

//fn main() {
    ////let mut out_dir = current_dir().unwrap();
    ////out_dir.push("schema");
    ////create_dir_all(&out_dir).unwrap();
    ////remove_schemas(&out_dir).unwrap();

    ////export_schema(&schema_for!(InitMsg), &out_dir);
    ////export_schema(&schema_for!(HandleMsg), &out_dir);
    ////export_schema(&schema_for!(QueryMsg), &out_dir);
    ////export_schema(&schema_for!(State), &out_dir);
    ////export_schema(&schema_for!(CountResponse), &out_dir);
//}

//fn main() {
    //let mut out_dir = current_dir().unwrap();
    //out_dir.push("schema");
    //create_dir_all(&out_dir).unwrap();
    //remove_schemas(&out_dir).unwrap();

    //export_schema(&schema_for!(InitMsg), &out_dir);
    //export_schema(&schema_for!(HandleMsg), &out_dir);
    //export_schema(&schema_for!(HandleAnswer), &out_dir);
    //export_schema(&schema_for!(QueryMsg), &out_dir);
    //export_schema(&schema_for!(QueryAnswer), &out_dir);
//}

//fn main() {
    //// let mut out_dir = current_dir().unwrap();
    //// out_dir.push("schema");
    //// create_dir_all(&out_dir).unwrap();
    //// remove_schemas(&out_dir).unwrap();
    ////
    //// export_schema(&schema_for!(InitMsg), &out_dir);
    //// export_schema(&schema_for!(HandleMsg), &out_dir);
    //// export_schema(&schema_for!(QueryMsg), &out_dir);
    //// export_schema(&schema_for!(State), &out_dir);
    //// export_schema(&schema_for!(CountResponse), &out_dir);
//}

//fn main() {
    //let mut out_dir = current_dir().unwrap();
    //out_dir.push("schema");
    //create_dir_all(&out_dir).unwrap();
    //remove_schemas(&out_dir).unwrap();

    //export_schema(&schema_for!(InitMsg), &out_dir);
    //export_schema(&schema_for!(HandleMsg), &out_dir);
    //export_schema(&schema_for!(QueryMsg), &out_dir);
    //export_schema(&schema_for!(State), &out_dir);
    //export_schema(&schema_for!(CountResponse), &out_dir);
//}
