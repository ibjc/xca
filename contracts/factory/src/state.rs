use cosmwasm_std::Addr;

use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const VAA_ARCHIVE: Map<&[u8], bool> = Map::new("vaa_archive");

#[cw_serde]
pub struct Config {
    pub x_account_registry: Addr,
}
