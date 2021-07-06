pub use crate::snip20_impl::msg as snip20;

use cosmwasm_std::{Binary, Decimal, HumanAddr, Uint128};
use fadroma_scrt_callback::{Callback, ContractInstance, ContractInstantiationInfo};
use fadroma_scrt_migrate::types::ContractStatusLevel;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{TokenPair, TokenPairAmount, TokenType, TokenTypeAmount};

pub mod factory {
    use super::ido::TokenSaleConfig;
    use super::*;
    use crate::{Exchange, ExchangeSettings, Pagination};
    use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub snip20_contract: ContractInstantiationInfo,
        pub lp_token_contract: ContractInstantiationInfo,
        pub pair_contract: ContractInstantiationInfo,
        pub ido_contract: ContractInstantiationInfo,
        pub exchange_settings: ExchangeSettings<HumanAddr>,
        pub admin: Option<HumanAddr>,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        /// Set pause/migration status
        SetStatus {
            level: ContractStatusLevel,
            reason: String,
            new_address: Option<HumanAddr>,
        },
        /// Set contract templates and exchange settings. Admin only command.
        SetConfig {
            snip20_contract: Option<ContractInstantiationInfo>,
            lp_token_contract: Option<ContractInstantiationInfo>,
            pair_contract: Option<ContractInstantiationInfo>,
            ido_contract: Option<ContractInstantiationInfo>,
            exchange_settings: Option<ExchangeSettings<HumanAddr>>,
        },
        /// Instantiates an exchange pair contract
        CreateExchange {
            pair: TokenPair<HumanAddr>,
        },
        /// Instantiates an IDO contract
        CreateIdo {
            info: TokenSaleConfig,
        },
        /// Used by a newly instantiated exchange contract to register
        /// itself with the factory
        RegisterExchange {
            pair: TokenPair<HumanAddr>,
            signature: Binary,
        },
        /// Used by a newly instantiated IDO contract to register
        /// itself with the factory
        RegisterIdo {
            signature: Binary,
        },
        /// Adds already created exchanges to the registry. Admin only command.
        AddExchanges {
            exchanges: Vec<Exchange<HumanAddr>>,
        },
        /// Adds already created IDO addresses to the registry. Admin only command.
        AddIdos {
            idos: Vec<HumanAddr>,
        },
        Admin(AdminHandleMsg),
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        /// Get configuration (contract templates and exchange settings)
        GetConfig {},
        GetExchangeAddress {
            pair: TokenPair<HumanAddr>,
        },
        ListIdos {
            pagination: Pagination,
        },
        ListExchanges {
            pagination: Pagination,
        },
        GetExchangeSettings,

        Admin(AdminQueryMsg),
    }

    #[derive(Serialize, Deserialize, Debug, JsonSchema, PartialEq)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        GetExchangeAddress {
            address: HumanAddr,
        },
        ListIdos {
            idos: Vec<HumanAddr>,
        },
        ListExchanges {
            exchanges: Vec<Exchange<HumanAddr>>,
        },
        GetExchangeSettings {
            settings: ExchangeSettings<HumanAddr>,
        },
        Config {
            snip20_contract: ContractInstantiationInfo,
            lp_token_contract: ContractInstantiationInfo,
            pair_contract: ContractInstantiationInfo,
            ido_contract: ContractInstantiationInfo,
            exchange_settings: ExchangeSettings<HumanAddr>,
        },
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
        pub callback: Callback<HumanAddr>,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        /// Set pause/migration status
        SetStatus {
            level: ContractStatusLevel,
            reason: String,
            new_address: Option<HumanAddr>,
        },
        AddLiquidity {
            deposit: TokenPairAmount<HumanAddr>,
            /// The amount the price moves in a trading pair between when a transaction is submitted and when it is executed.
            /// Transactions that exceed this threshold will be rejected.
            slippage_tolerance: Option<Decimal>,
        },
        RemoveLiquidity {
            /// The amount of LP tokens burned.
            amount: Uint128,
            /// The account to refund the tokens to.
            recipient: HumanAddr,
        },
        Swap {
            /// The token type to swap from.
            offer: TokenTypeAmount<HumanAddr>,
            expected_return: Option<Uint128>,
        },
        /// Sent by the LP token contract so that we can record its address.
        OnLpTokenInit,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        PairInfo,
        Version,
        SwapSimulation {
            /// The token type to swap from.
            offer: TokenTypeAmount<HumanAddr>,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        PairInfo {
            liquidity_token: ContractInstance<HumanAddr>,
            factory: ContractInstance<HumanAddr>,
            pair: TokenPair<HumanAddr>,
            amount_0: Uint128,
            amount_1: Uint128,
            total_liquidity: Uint128,
        },
        Version {
            version: u32,
        },
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
    use composable_admin::admin::{AdminHandleMsg, AdminQueryMsg};
    use fadroma::scrt::callback::ContractInstance;

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct InitMsg {
        pub info: TokenSaleConfig,
        /// Should be the address of the original sender, since this is initiated by the factory.
        pub admin: HumanAddr,
        /// Used by the IDO to register itself with the factory.
        pub callback: Callback<HumanAddr>,
    }
    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct TokenSaleConfig {
        /// The token that will be used to buy the SNIP20.
        pub input_token: TokenType<HumanAddr>,
        /// The price for a single token.
        pub rate: Uint128,
        // The address of the SNIP20 token beind sold.
        pub sold_token: ContractInstance<HumanAddr>,
        /// The addresses that are eligible to participate in the sale.
        pub whitelist: Vec<HumanAddr>,
        /// The maximum number of participants allowed.
        pub max_seats: u32,
        /// The total amount that each participant is allowed to buy.
        pub max_allocation: Uint128,
        /// The minimum amount that each participant is allowed to buy.
        pub min_allocation: Uint128,
        /// Time when the sale will start (if None, it will start immediately)
        pub start_time: Option<u64>,
        /// Time when the sale will end
        pub end_time: Option<u64>,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        Swap { amount: Uint128 },
        Admin(AdminHandleMsg),
        Refund,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        GetRate,
        Admin(AdminQueryMsg),
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        GetRate { rate: Uint128 },
    }
}
