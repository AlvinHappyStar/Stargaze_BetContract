use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr};
use crate::msg::History;

use cw_storage_plus::{Item, Map};
use cw20::Denom;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Config {
    /// Owner If None set, contract is frozen.
    pub owner: Addr,
    pub denom: Denom,
    pub enabled: bool,
    pub flip_count: u64
}

pub const CONFIG_KEY: &str = "config";
pub const CONFIG: Item<Config> = Item::new(CONFIG_KEY);

pub const HISTORY_KEY: &str = "history";
pub const HISTORY: Map<u64, History> = Map::new(HISTORY_KEY);