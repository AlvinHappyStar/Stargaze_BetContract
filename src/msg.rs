use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Uint128, Addr};
use cw20::Denom;

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    UpdateOwner {
        owner: Addr,
    },
    UpdateEnabled {
        enabled: bool,
    },
    Flip {
        level: u64
    },
    Rps {
        level: u64
    },
    Withdraw {
        amount: Uint128
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    RistoryMsg {
        count: u32
    },
    FistoryMsg {
        count: u32
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: Addr,
    pub enabled: bool,
    pub denom: Denom,
    pub treasury_amount: Uint128,
    pub flip_count: u64,
    pub rps_count: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RHistory {
    pub id: u64,
    pub address: Addr,
    pub level: u64,
    pub win: Option<u8>,
    pub bet_amount: Uint128,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RHistoryResponse {
    pub list: Vec<RHistory>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FHistory {
    pub id: u64,
    pub address: Addr,
    pub level: u64,
    pub win: Option<u8>,
    pub bet_amount: Uint128,
    pub timestamp: u64
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FHistoryResponse {
    pub list: Vec<FHistory>
}


#[derive(Hash)]
pub struct HashObj {
    pub time: u64,
    pub address: Addr,
    pub level: u64,
    pub count: u64
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}
