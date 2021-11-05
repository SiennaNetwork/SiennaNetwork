pub use crate::snip20_impl::msg as snip20;

use fadroma::{
    scrt_callback::Callback,
    scrt_link::{ContractLink, ContractInstantiationInfo},
    scrt::{Binary, Decimal, HumanAddr, Uint128},
    scrt_migrate::ContractStatusLevel,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{TokenPair, TokenPairAmount, TokenType, TokenTypeAmount};

pub mod factory {
    use super::ido::TokenSaleConfig;
    use super::*;
    use crate::{
        exchange::{Exchange, ExchangeSettings},
        Pagination,
    };
    use fadroma::admin::{
        HandleMsg as AdminHandleMsg,
        QueryMsg as AdminQueryMsg
    };

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct InitMsg {
        pub snip20_contract: ContractInstantiationInfo,
        pub lp_token_contract: ContractInstantiationInfo,
        pub pair_contract: ContractInstantiationInfo,
        pub launchpad_contract: ContractInstantiationInfo,
        pub ido_contract: ContractInstantiationInfo,
        pub exchange_settings: ExchangeSettings<HumanAddr>,
        pub admin: Option<HumanAddr>,
        pub prng_seed: Binary,
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
            launchpad_contract: Option<ContractInstantiationInfo>,
            ido_contract: Option<ContractInstantiationInfo>,
            exchange_settings: Option<ExchangeSettings<HumanAddr>>,
        },
        /// Instantiates an exchange pair contract
        CreateExchange {
            pair: TokenPair<HumanAddr>,
            entropy: Binary,
        },
        /// Create the launchpad contract
        CreateLaunchpad {
            tokens: Vec<super::launchpad::TokenSettings>,
            entropy: Binary,
        },
        /// Instantiates an IDO contract
        CreateIdo {
            info: TokenSaleConfig,
            // If whitelist addresses will be filled from the launchpad, you'll have to provide
            // list of tokens that will be used for gathering whitelist addresses.
            // For native token None is used.
            tokens: Option<Vec<Option<HumanAddr>>>,
            entropy: Binary,
        },
        /// Add addresses that are allowed to create IDOs
        IdoWhitelist {
            addresses: Vec<HumanAddr>,
        },
        /// Used by a newly instantiated exchange contract to register
        /// itself with the factory
        RegisterExchange {
            pair: TokenPair<HumanAddr>,
            signature: Binary,
        },
        /// Used by the launchpad to register itself
        RegisterLaunchpad {
            signature: Binary,
        },
        /// Used by a newly instantiated IDO contract to register
        /// itself with the factory
        RegisterIdo {
            signature: Binary,
        },
        /// Transfers exchanges to a new instance. Admin only command.
        TransferExchanges {
            /// The password set on the receiving instance.
            password: String,
            /// New factory instance.
            new_instance: ContractLink<HumanAddr>,
            /// Optionally, skip transferring the given exchanges.
            skip: Option<Vec<HumanAddr>>
        },
        ReceiveExchanges {
            /// The password that was set on this instance.
            password: String,
            /// Indicates whether all exchanges have been transferred.
            finalize: bool,
            exchanges: Vec<Exchange<HumanAddr>>
        },
        SetMigrationPassword {
            password: String
        },
        /// Adds already created IDO addresses to the registry. Admin only command.
        AddIdos {
            idos: Vec<HumanAddr>,
        },
        /// Adds already existing launchpad contract, admin only command.
        AddLaunchpad {
            launchpad: ContractLink<HumanAddr>,
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
        GetLaunchpadAddress,
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
        GetLaunchpadAddress {
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
            launchpad_contract: ContractInstantiationInfo,
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
        pub factory_info: ContractLink<HumanAddr>,
        pub callback: Callback<HumanAddr>,
        pub prng_seed: Binary,
        pub entropy: Binary,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum HandleMsg {
        AddLiquidity {
            deposit: TokenPairAmount<HumanAddr>,
            /// The amount the price moves in a trading pair between when a transaction is submitted and when it is executed.
            /// Transactions that exceed this threshold will be rejected.
            slippage_tolerance: Option<Decimal>,
        },
        Swap {
            /// The token type to swap from.
            offer: TokenTypeAmount<HumanAddr>,
            expected_return: Option<Uint128>,
            to: Option<HumanAddr>,
        },
        // SNIP20 receiver interface
        Receive {
            from: HumanAddr,
            msg: Option<Binary>,
            amount: Uint128,
        },
        /// Sent by the LP token contract so that we can record its address.
        OnLpTokenInit,
        /// Can only be called by the current factory.
        ChangeFactory {
            contract: ContractLink<HumanAddr>
        }
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ReceiverCallbackMsg {
        Swap {
            expected_return: Option<Uint128>,
            to: Option<HumanAddr>,
        },
        RemoveLiquidity {
            recipient: HumanAddr,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        PairInfo,
        SwapSimulation {
            /// The token type to swap from.
            offer: TokenTypeAmount<HumanAddr>,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsgResponse {
        PairInfo {
            liquidity_token: ContractLink<HumanAddr>,
            factory: ContractLink<HumanAddr>,
            pair: TokenPair<HumanAddr>,
            amount_0: Uint128,
            amount_1: Uint128,
            total_liquidity: Uint128,
            contract_version: u32,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct SwapSimulationResponse {
        pub return_amount: Uint128,
        pub spread_amount: Uint128,
        pub commission_amount: Uint128,
    }
}

pub mod launchpad {

    use super::*;
    use fadroma::admin::{
        HandleMsg as AdminHandleMsg,
        QueryMsg as AdminQueryMsg
    };

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct InitMsg {
        pub tokens: Vec<TokenSettings>,
        /// Should be the address of the original sender, since this is initiated by the factory.
        pub admin: HumanAddr,
        /// Seed for creating viewkey
        pub prng_seed: Binary,
        pub entropy: Binary,
        /// Used by the Launchpad to register itself with the factory.
        pub callback: Callback<HumanAddr>,
    }

    /// Configuration for single token that can be locked into the launchpad
    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct TokenSettings {
        pub token_type: TokenType<HumanAddr>,
        pub segment: Uint128,
        pub bounding_period: u64,
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
        /// SNIP20 receiver interface
        Receive {
            from: HumanAddr,
            msg: Option<Binary>,
            amount: Uint128,
        },
        /// Lock for native token, amount set for locking
        /// will be floored to the closest segment, the rest
        /// will be sent back.
        Lock { amount: Uint128 },
        /// Perform unlocking of native token
        Unlock { entries: u32 },
        /// Add additional token for locking into launchpad
        AdminAddToken { config: TokenSettings },
        /// Remove token from the launchpad, this will send all the previously
        /// locked funds back to their owners
        AdminRemoveToken { index: u32 },
        /// Change admin handle
        Admin(AdminHandleMsg),
        CreateViewingKey {
            entropy: String,
            padding: Option<String>,
        },
        SetViewingKey {
            key: String,
            padding: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        Admin(AdminQueryMsg),
        LaunchpadInfo,
        /// Get information about the users account
        UserInfo {
            address: HumanAddr,
            key: String,
        },
        /// Get a list of addresses that are drawm
        /// as potential participants in an IDO
        Draw {
            tokens: Vec<Option<HumanAddr>>,
            number: u32,
            timestamp: u64,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ReceiverCallbackMsg {
        /// Perform locking of the funds into the launchpad contract
        /// Amount sent through the snip20 will be floored to closest
        /// segment and the rest will be sent back to sender.
        Lock {},

        /// Perform unlocking of the funds, for any token that is not
        /// native user will have to send 0 amount to launchpad with unlock
        /// message and send how many entries he wants to unlock
        Unlock { entries: u32 },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        LaunchpadInfo(Vec<QueryTokenConfig>),
        UserInfo(Vec<QueryAccountToken>),
        DrawnAddresses(Vec<HumanAddr>),
    }

    /// Token configuration that holds the configuration for each token
    #[derive(Serialize, Deserialize, JsonSchema, Clone)]
    pub struct QueryTokenConfig {
        pub token_type: TokenType<HumanAddr>,
        pub segment: Uint128,
        pub bounding_period: u64,
        pub token_decimals: u8,
        pub locked_balance: Uint128,
    }

    /// Account token representation that holds all the entries for this token
    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct QueryAccountToken {
        pub token_type: TokenType<HumanAddr>,
        pub balance: Uint128,
        pub entries: Vec<u64>,
    }
}

pub mod ido {
    use super::*;
    
    use fadroma::admin::{
        HandleMsg as AdminHandleMsg,
        QueryMsg as AdminQueryMsg
    };
    use fadroma::scrt_link::ContractLink;

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct InitMsg {
        pub info: TokenSaleConfig,
        /// Should be the address of the original sender, since this is initiated by the factory.
        pub admin: HumanAddr,
        /// Used by the IDO to register itself with the factory.
        pub callback: Callback<HumanAddr>,
        /// Used by the IDO to fill the whitelist spots with random pics
        pub launchpad: Option<WhitelistRequest>,
        /// Seed for creating viewkey
        pub prng_seed: Binary,
        pub entropy: Binary,
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    pub struct WhitelistRequest {
        /// Launchpad contract instance information
        pub launchpad: ContractLink<HumanAddr>,
        /// Vector of tokens address needs to have locked in order to be considered
        /// for a draw. Tokens need to be configured in the Launchpad as eligible.
        /// Option<> is because if None that will represent a native token.
        pub tokens: Vec<Option<HumanAddr>>,
    }

    #[derive(Serialize, Deserialize, JsonSchema, Clone)]
    pub struct TokenSaleConfig {
        /// The token that will be used to buy the SNIP20.
        pub input_token: TokenType<HumanAddr>,
        /// The price for a single token.
        pub rate: Uint128,
        // The address of the SNIP20 token beind sold.
        pub sold_token: ContractLink<HumanAddr>,
        /// The addresses that are eligible to participate in the sale.
        pub whitelist: Vec<HumanAddr>,
        /// The maximum number of participants allowed.
        pub max_seats: u32,
        /// The total amount that each participant is allowed to buy.
        pub max_allocation: Uint128,
        /// The minimum amount that each participant is allowed to buy.
        pub min_allocation: Uint128,
        /// Sale type settings
        pub sale_type: Option<SaleType>,
    }

    #[derive(Clone, Serialize, Deserialize, Eq, PartialEq, JsonSchema, Debug)]
    pub enum SaleType {
        PreLockAndSwap,
        PreLockOnly,
        SwapOnly,
    }

    impl Default for SaleType {
        fn default() -> SaleType {
            SaleType::PreLockAndSwap
        }
    }

    impl From<&str> for SaleType {
        fn from(source: &str) -> SaleType {
            match source {
                "pre_lock_only" => SaleType::PreLockOnly,
                "swap_only" => SaleType::SwapOnly,
                _ => SaleType::PreLockAndSwap,
            }
        }
    }

    impl From<String> for SaleType {
        fn from(source: String) -> SaleType {
            SaleType::from(source.as_str())
        }
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
        // SNIP20 receiver interface
        Receive {
            from: HumanAddr,
            msg: Option<Binary>,
            amount: Uint128,
        },
        /// Pre lock funds before the sale has started, and then claim them after the sale starts.
        PreLock { amount: Uint128 },
        /// Swap custom or native coin for selling coin
        Swap {
            amount: Uint128,
            /// If the recipient of the funds
            /// is going to be someone different
            /// then the sender
            recipient: Option<HumanAddr>,
        },
        /// Change admin handle
        Admin(AdminHandleMsg),
        /// Ask for a refund after the sale is finished
        AdminRefund { address: Option<HumanAddr> },
        /// Admin can claim profits from sale after the sale finishes
        AdminClaim { address: Option<HumanAddr> },
        /// Add new address to whitelist
        AdminAddAddresses { addresses: Vec<HumanAddr> },
        CreateViewingKey {
            entropy: String,
            padding: Option<String>,
        },
        SetViewingKey {
            key: String,
            padding: Option<String>,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        /// Get pause/migration status
        Status,
        EligibilityInfo {
            address: HumanAddr,
        },
        SaleInfo,
        SaleStatus,
        Admin(AdminQueryMsg),
        // Do not change the signatures below. They need to work with Keplr.
        Balance {
            address: HumanAddr,
            key: String,
        },
        TokenInfo {},
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryResponse {
        Eligibility {
            can_participate: bool,
        },
        SaleInfo {
            /// The token that is used to buy the sold SNIP20.
            input_token: TokenType<HumanAddr>,
            /// The token that is being sold.
            sold_token: ContractLink<HumanAddr>,
            /// The conversion rate at which the token is sold.
            rate: Uint128,
            /// Number of participants currently.
            taken_seats: u32,
            /// The maximum number of participants allowed.
            max_seats: u32,
            /// The total amount that each participant is allowed to buy.
            max_allocation: Uint128,
            /// The minimum amount that each participant is allowed to buy.
            min_allocation: Uint128,
            /// Sale start time.
            start: Option<u64>,
            /// Sale end time.
            end: Option<u64>,
        },
        Status {
            total_allocation: Uint128,
            available_for_sale: Uint128,
            sold_in_pre_lock: Uint128,
            is_active: bool,
        },
        // Do not change the signatures below. They need to work with Keplr.
        Balance {
            pre_lock_amount: Uint128,
            total_bought: Uint128,
        },
        TokenInfo {
            name: String,
            symbol: String,
            decimals: u8,
            total_supply: Option<Uint128>,
        },
    }

    #[derive(Serialize, Deserialize, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum ReceiverCallbackMsg {
        Activate {
            /// Time when the sale will start (if None, it will start immediately)
            start_time: Option<u64>,
            /// Time when the sale will end
            end_time: u64,
        },
        /// Pre lock sent funds before the sale has started, and then claim them after the sale starts.
        PreLock {},
        Swap {
            /// If the recipient of the funds
            /// is going to be someone different
            /// then the sender
            recipient: Option<HumanAddr>,
        },
    }
}
