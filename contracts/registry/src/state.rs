use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};
use xca::registry::ChainInfo;

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const VAA_ARCHIVE: Map<&[u8], bool> = Map::new("vaa_archive");

#[cw_serde]
pub struct Config {
    pub chain_id_here: u64,
    pub x_account_code_id: u64,
    pub chain_info: Vec<ChainInfo>,
}
