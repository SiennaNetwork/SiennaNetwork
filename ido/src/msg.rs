use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    OnSnip20Init,
}

#[derive(Serialize, Deserialize, JsonSchema)]
pub enum QueryMsg {

}