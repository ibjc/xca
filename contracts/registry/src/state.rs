use cosmwasm_std::Addr;

use cosmwasm_schema::cw_serde;
use cw_storage_plus::{Item, Map};
use xca::registry::Chain;

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const VAA_ARCHIVE: Map<&[u8], bool> = Map::new("vaa_archive");

#[cw_serde]
pub struct Config {
    pub wormhole_core_contract: Addr,
    pub x_account_factory: Addr,
    pub wormhole_chain_ids: Vec<Chain>,
    pub x_account_code_id: u64,
}
