use cosmwasm_std::{Addr, StdResult};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const VAA_ARCHIVE: Map<&[u8], bool> = Map::new("vaa_archive");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub wormhole_contract: Addr,
}

pub struct ShouterMessage {
    pub payload: Vec<u8>,
}

impl ShouterMessage {
    pub fn deserialize(data: &Vec<u8>) -> StdResult<Self> {
        let data = data.as_slice();
        let payload = data;

        Ok(ShouterMessage{
            payload: payload.to_vec(),
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.payload.clone()
    }
}
