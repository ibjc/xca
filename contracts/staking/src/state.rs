use cosmwasm_std::{Uint128, Addr};

use cw_storage_plus::{Item, Map};
use cosmwasm_schema::cw_serde;

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const STATE: Item<State> = Item::new("\u{0}\u{7}state");
pub const STAKERS: Map<&Addr, Uint128> = Map::new("stakers");
pub const VAA_ARCHIVE: Map<&[u8], bool> = Map::new("vaa_archive");

#[cw_serde]
pub struct Config {
    pub denom_name: String,
}

#[cw_serde]
pub struct State{
  pub total_staked: Uint128,
}