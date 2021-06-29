use cosmwasm_std::{Binary, HumanAddr};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use fadroma_scrt_callback::{ContractInstantiationInfo, ContractInstance};
use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};
use sienna_rewards::msg::RewardPoolConfig;
use sienna_rewards::data::RewardPool;


#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub admin: Option<HumanAddr>,
    pub rewards_contract: ContractInstantiationInfo
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    CreatePool {
        info: PoolInitInfo
    },
    RegisterPool {
        signature: Binary
    },
    AddPools {
        instances: Vec<ContractInstance<HumanAddr>>
    },
    RemovePools {
        addresses: Vec<HumanAddr>
    },
    ChangeRewardsContract {
        contract: ContractInstantiationInfo
    },
    Admin(AdminHandleMsg)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Pools,
    Admin(AdminQueryMsg)
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsgResponse {
    Pools(Vec<PoolContractInfo>)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
pub struct PoolInitInfo {
    pub admin: Option<HumanAddr>,
    pub reward_token: ContractInstance<HumanAddr>,
    pub pool: RewardPoolConfig,
    pub claim_interval: u64,
    pub prng_seed: Binary,
    pub entropy: Binary
}

#[derive(Serialize, Deserialize, Debug, JsonSchema)]

pub struct PoolContractInfo {
    pub pool: RewardPool<HumanAddr>,
    pub address: HumanAddr
}
