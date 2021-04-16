use std::fmt;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, HumanAddr, Binary};
use cosmwasm_utils::viewing_key::ViewingKey;

use crate::data::ContractStatusLevel;

/// Generates an enum with equivalent named variants, but without the payload.
/// The generated enum is used in InitMsg to specify which messages to be
/// disabled in the contract. This way, the two are also kept in sync.
macro_rules! generate_disable_msg {
    ($pub:vis
    enum $EnumName:ident {
        $(
            $VariantName:ident $(
                { $($Field:ident: $Type:ty),+ $(,)? }
            )?
        ),+ $(,)?
    }
    ) => {
        #[derive(Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "snake_case")]      
        $pub enum $EnumName {
            $(
                $VariantName $(
                    { $($Field: $Type ,)+ }
                )? ,
            )+
        }

        #[derive(Serialize, Deserialize, JsonSchema, PartialEq)]
        #[serde(rename_all = "snake_case")]
        $pub enum DisabledMsg {
            $($VariantName,)+
        }

        impl $EnumName {
            pub fn to_disabled(&self) -> DisabledMsg {
                match *self {
                    $($EnumName::$VariantName { .. } => DisabledMsg::$VariantName,)*
                }
            }
        }

        impl fmt::Display for DisabledMsg {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                let name = match *self {
                    $(DisabledMsg::$VariantName { .. } => stringify!($VariantName),)*
                };

                write!(
                    f,
                    "{}",
                    name
                )
            }
        }
    }
}

generate_disable_msg!{
    pub enum HandleMsg {
        // Native coin interactions
        Redeem {
            amount: Uint128,
            denom: Option<String>,
            padding: Option<String>,
        },
        Deposit {
            padding: Option<String>,
        },

        // Base ERC-20 stuff
        Transfer {
            recipient: HumanAddr,
            amount: Uint128,
            padding: Option<String>,
        },
        Send {
            recipient: HumanAddr,
            amount: Uint128,
            msg: Option<Binary>,
            padding: Option<String>,
        },
        Burn {
            amount: Uint128,
            padding: Option<String>,
        },
        RegisterReceive {
            code_hash: String,
            padding: Option<String>,
        },
        CreateViewingKey {
            entropy: String,
            padding: Option<String>,
        },
        SetViewingKey {
            key: String,
            padding: Option<String>,
        },

        // Allowance
        IncreaseAllowance {
            spender: HumanAddr,
            amount: Uint128,
            expiration: Option<u64>,
            padding: Option<String>,
        },
        DecreaseAllowance {
            spender: HumanAddr,
            amount: Uint128,
            expiration: Option<u64>,
            padding: Option<String>,
        },
        TransferFrom {
            owner: HumanAddr,
            recipient: HumanAddr,
            amount: Uint128,
            padding: Option<String>,
        },
        SendFrom {
            owner: HumanAddr,
            recipient: HumanAddr,
            amount: Uint128,
            msg: Option<Binary>,
            padding: Option<String>,
        },
        BurnFrom {
            owner: HumanAddr,
            amount: Uint128,
            padding: Option<String>,
        },

        // Mint
        Mint {
            recipient: HumanAddr,
            amount: Uint128,
            padding: Option<String>,
        },
        AddMinters {
            minters: Vec<HumanAddr>,
            padding: Option<String>,
        },
        RemoveMinters {
            minters: Vec<HumanAddr>,
            padding: Option<String>,
        },
        SetMinters {
            minters: Vec<HumanAddr>,
            padding: Option<String>,
        },

        // Admin
        ChangeAdmin {
            address: HumanAddr,
            padding: Option<String>,
        },
        SetContractStatus {
            level: ContractStatusLevel,
            padding: Option<String>,
        }
    }
}

#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum HandleAnswer {
    // Native
    Deposit {
        status: ResponseStatus,
    },
    Redeem {
        status: ResponseStatus,
    },

    // Base
    Transfer {
        status: ResponseStatus,
    },
    Send {
        status: ResponseStatus,
    },
    Burn {
        status: ResponseStatus,
    },
    RegisterReceive {
        status: ResponseStatus,
    },
    CreateViewingKey {
        key: ViewingKey,
    },
    SetViewingKey {
        status: ResponseStatus,
    },

    // Allowance
    IncreaseAllowance {
        spender: HumanAddr,
        owner: HumanAddr,
        allowance: Uint128,
    },
    DecreaseAllowance {
        spender: HumanAddr,
        owner: HumanAddr,
        allowance: Uint128,
    },
    TransferFrom {
        status: ResponseStatus,
    },
    SendFrom {
        status: ResponseStatus,
    },
    BurnFrom {
        status: ResponseStatus,
    },

    // Mint
    Mint {
        status: ResponseStatus,
    },
    AddMinters {
        status: ResponseStatus,
    },
    RemoveMinters {
        status: ResponseStatus,
    },
    SetMinters {
        status: ResponseStatus,
    },

    // Other
    ChangeAdmin {
        status: ResponseStatus,
    },
    SetContractStatus {
        status: ResponseStatus,
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema)]
pub struct CreateViewingKeyResponse {
    pub key: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Success,
    Failure,
}