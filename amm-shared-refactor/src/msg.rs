use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{HumanAddr, Binary, Uint128, Decimal};

use crate::{TokenPair, TokenType, TokenTypeAmount, TokenPairAmount};

use fadroma_scrt_callback::{ContractInstantiationInfo, ContractInstance, Callback};
use fadroma_scrt_migrate::types::ContractStatusLevel;

pub mod factory {
    use super::*;
    use super::ido::IdoInitConfig;
    use crate::{Pagination, Exchange, ExchangeSettings};
    use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub snip20_contract:   ContractInstantiationInfo,
        pub lp_token_contract: ContractInstantiationInfo,
        pub pair_contract:     ContractInstantiationInfo,
        pub ido_contract:      ContractInstantiationInfo,
        pub exchange_settings: ExchangeSettings<HumanAddr>
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        /// Set pause/migration status
        SetStatus {
            level: ContractStatusLevel,
            reason: String,
            new_address: Option<HumanAddr>
        },
        /// Set contract templates and exchange settings
        SetConfig {
            snip20_contract:   Option<ContractInstantiationInfo>,
            lp_token_contract: Option<ContractInstantiationInfo>,
            pair_contract:     Option<ContractInstantiationInfo>,
            ido_contract:      Option<ContractInstantiationInfo>,
            exchange_settings: Option<ExchangeSettings<HumanAddr>>
        },
        /// Instantiates an exchange pair contract
        CreateExchange {
            pair: TokenPair<HumanAddr>
        },
        /// Instantiates an IDO contract
        CreateIdo {
            info: IdoInitConfig
        },
        /// Used by a newly instantiated exchange contract to register
        /// itself with the factory
        RegisterExchange {
            pair: TokenPair<HumanAddr>,
            signature: Binary
        },
        /// Used by a newly instantiated IDO contract to register
        /// itself with the factory
        RegisterIdo {
            signature: Binary
        },

        Admin(AdminHandleMsg)
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        /// Get configuration (contract templates and exchange settings)
        GetConfig {},
        GetExchangeAddress { pair: TokenPair<HumanAddr> },
        ListIdos { pagination: Pagination },
        ListExchanges { pagination: Pagination },
        GetExchangeSettings,

        Admin(AdminQueryMsg)
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        GetExchangeAddress {
            address: HumanAddr
        },
        ListIdos {
            idos: Vec<HumanAddr>
        },
        ListExchanges {
            exchanges: Vec<Exchange<HumanAddr>>
        },
        GetExchangeSettings {
            settings: ExchangeSettings<HumanAddr>
        },
        Config {
            snip20_contract:   ContractInstantiationInfo,
            lp_token_contract: ContractInstantiationInfo,
            pair_contract:     ContractInstantiationInfo,
            ido_contract:      ContractInstantiationInfo,
            exchange_settings: ExchangeSettings<HumanAddr>
        }
    }
}

pub mod exchange {
    use super::*;

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        /// The tokens that will be managed by the exchange
        pub pair: TokenPair<HumanAddr>,
        /// LP token instantiation info
        pub lp_token_contract: ContractInstantiationInfo,
        /// Used by the exchange contract to
        /// send back its address to the factory on init
        pub factory_info: ContractInstance<HumanAddr>,
        pub callback: Callback<HumanAddr>
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        /// Set pause/migration status
        SetStatus {
            level: ContractStatusLevel,
            reason: String,
            new_address: Option<HumanAddr>
        },
        AddLiquidity {
            deposit: TokenPairAmount<HumanAddr>,
            /// The amount the price moves in a trading pair between when a transaction is submitted and when it is executed.
            /// Transactions that exceed this threshold will be rejected.
            slippage_tolerance: Option<Decimal>
        },
        RemoveLiquidity {
            /// The amount of LP tokens burned.
            amount: Uint128,
            /// The account to refund the tokens to.
            recipient: HumanAddr
        },
        Swap {
            /// The token type to swap from.
            offer: TokenTypeAmount<HumanAddr>,
            expected_return: Option<Uint128>,
        },
        /// Sent by the LP token contract so that we can record its address.
        OnLpTokenInit
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        PairInfo,
        SwapSimulation {
            /// The token type to swap from.
            offer: TokenTypeAmount<HumanAddr>
        }
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        PairInfo {
            liquidity_token: ContractInstance<HumanAddr>,
            factory:         ContractInstance<HumanAddr>,
            pair:            TokenPair<HumanAddr>,
            amount_0:        Uint128,
            amount_1:        Uint128,
        }
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct SwapSimulationResponse {
        pub return_amount: Uint128,
        pub spread_amount: Uint128,
        pub commission_amount: Uint128,
    }
}

pub mod ido {
    use super::*;
    use super::snip20::Snip20InitConfig;

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct IdoInitMsg {
        pub snip20_contract: ContractInstantiationInfo,
        pub info: IdoInitConfig,
        /// Used by the IDO to register itself with the factory.
        pub callback: Callback<HumanAddr>
    }
    
    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct IdoInitConfig {
        /// The token that will be used to buy the instantiated SNIP20
        pub input_token: TokenType<HumanAddr>,
        pub rate: Uint128,
        pub snip20_init_info: Snip20TokenInitInfo
    }
    
    #[derive(Serialize, Deserialize, JsonSchema)]
    /// Used to provide only the essential info
    /// to an IDO that instantiates a snip20 token
    pub struct Snip20TokenInitInfo {
        pub name: String,
        pub prng_seed: Binary,
        pub symbol: String,
        pub decimals: u8,
        pub config: Option<Snip20InitConfig>
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        /// Set pause/migration status
        SetStatus {
            level: ContractStatusLevel,
            reason: String,
            new_address: Option<HumanAddr>
        },
        OnSnip20Init,
        Swap {
            amount: Uint128
        }
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        GetRate
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        GetRate { 
            rate: Uint128 
        }
    }
}

pub mod sienna_burner {
    use super::*;
    use composable_admin::multi_admin::{
        MultiAdminHandleMsg, MultiAdminQueryMsg, MultiAdminQueryResponse
    };

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct InitMsg {
        /// SIENNA token
        pub sienna_token: ContractInstance<HumanAddr>,
        pub pairs: Option<Vec<HumanAddr>>,
        /// The account to burn SIENNA from
        pub burn_pool: HumanAddr,
        /// Needs to be added as an admin in order to allow
        /// it to add new pair addresses.
        pub factory_address: HumanAddr,
        pub admins: Option<Vec<HumanAddr>>
    }
    
    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        /// Set pause/migration status
        SetStatus {
            level:       ContractStatusLevel,
            reason:      String,
            new_address: Option<HumanAddr>
        },
        Burn           { amount: Uint128 },
        AddPairs       { pairs: Vec<HumanAddr>, },
        RemovePairs    { pairs: Vec<HumanAddr>, },
        SetBurnPool    { address: HumanAddr },
        SetSiennaToken { info: ContractInstance<HumanAddr> },
        Admin(MultiAdminHandleMsg),
    }
    
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        SiennaToken,
        BurnPool,
        Admin(MultiAdminQueryMsg),
    }
    
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryAnswer {
        SiennaToken { info: ContractInstance<HumanAddr> },
        BurnPool { address: HumanAddr },
        Admin(MultiAdminQueryResponse)
    }
    
    #[derive(Serialize, Deserialize, Clone, Debug, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ResponseStatus {
        Success,
        Failure,
    }
}

pub mod snip20 {
    use super::*;

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct Snip20InitMsg {
        pub name: String,
        pub admin: Option<HumanAddr>,
        pub symbol: String,
        pub decimals: u8,
        pub initial_balances: Option<Vec<Snip20InitialBalance>>,
        pub prng_seed: Binary,
        pub config: Option<Snip20InitConfig>,
        pub callback: Option<Callback<HumanAddr>>
    }
    
    #[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
    pub struct Snip20InitialBalance {
        pub address: HumanAddr,
        pub amount: Uint128,
    }
    
    /// This type represents optional configuration values which can be overridden.
    /// All values are optional and have defaults which are more private by default,
    /// but can be overridden if necessary
    #[derive(Serialize, Deserialize, JsonSchema, Clone, Default, Debug)]
    pub struct Snip20InitConfig {
        /// Indicates whether the total supply is public or should be kept secret.
        /// default: False
        pub public_total_supply: Option<bool>,
    }
    
    impl Snip20InitMsg {
        pub fn config(&self) -> Snip20InitConfig {
            self.config.clone().unwrap_or_default()
        }
    }
    
    impl Snip20InitConfig {
        pub fn public_total_supply(&self) -> bool {
            self.public_total_supply.unwrap_or(false)
        }
    }
}
