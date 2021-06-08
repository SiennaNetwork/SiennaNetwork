use std::{env::current_dir,fs::create_dir_all};
use cosmwasm_schema::{export_schema, remove_schemas, schema_for};

use amm_shared::msg::exchange as exchange;
use amm_shared::msg::factory as factory;
use amm_shared::msg::ido as ido;
use gov_token;
use lp_staking;
use lp_token::msg as lp_token;
use sienna_mgmt::msg as mgmt;
use sienna_rewards::msg as rewards;
use sienna_rpt::msg as rpt;
use snip20_reference_impl::msg as snip20;
use snip20_reference_impl::msg as token;
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

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("token");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(token::InitMsg), &out_dir);
    export_schema(&schema_for!(token::HandleMsg), &out_dir);
    export_schema(&schema_for!(token::HandleAnswer), &out_dir);
    export_schema(&schema_for!(token::QueryMsg), &out_dir);
    export_schema(&schema_for!(token::QueryAnswer), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("mgmt");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(mgmt::Init), &out_dir);
    export_schema(&schema_for!(mgmt::Handle), &out_dir);
    export_schema(&schema_for!(mgmt::Query), &out_dir);
    export_schema(&schema_for!(mgmt::Response), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("rpt");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(rpt::Init), &out_dir);
    export_schema(&schema_for!(rpt::Handle), &out_dir);
    export_schema(&schema_for!(rpt::Query), &out_dir);
    export_schema(&schema_for!(rpt::Response), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("rewards");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(rewards::InitMsg), &out_dir);
    export_schema(&schema_for!(rewards::HandleMsg), &out_dir);
    export_schema(&schema_for!(rewards::QueryMsg), &out_dir);
    export_schema(&schema_for!(rewards::QueryMsgResponse), &out_dir);

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
    out_dir.push("ido");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(ido::InitMsg), &out_dir);
    export_schema(&schema_for!(ido::HandleMsg), &out_dir);
    export_schema(&schema_for!(ido::QueryMsg), &out_dir);
    export_schema(&schema_for!(ido::QueryResponse), &out_dir);

    let mut out_dir = current_dir().unwrap();
    out_dir.push("api");
    out_dir.push("lp_token");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();
    export_schema(&schema_for!(amm_shared::msg::snip20::Snip20InitMsg), &out_dir);
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
