use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use crate::msg::{
    RHistory, FHistory
};
use cw_storage_plus::{Item, Map};
use cw20::Denom;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Addr,
    pub denom: Denom,
    pub enabled: bool,
    pub flip_count: u64,
    pub rps_count: u64,
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const RHISTORY_KEY: &str = "rhistory";
pub const RHISTORY: Map<u64, RHistory> = Map::new(RHISTORY_KEY);

pub const FHISTORY_KEY: &str = "fhistory";
pub const FHISTORY: Map<u64, FHistory> = Map::new(FHISTORY_KEY);