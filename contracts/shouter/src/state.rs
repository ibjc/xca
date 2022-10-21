use cosmwasm_std::{Addr, StdResult, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};
use xca::byte_utils::ByteUtils;

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


pub struct ShoutBackMessage {
    pub best_bid: Uint128,
    pub best_ask: Uint128,
}

impl ShoutBackMessage {
    pub fn deserialize(data: &Vec<u8>) -> StdResult<Self> {
        let data = data.as_slice();
        let (best_bid, best_ask) = data.get_u256(0);

        Ok(ShoutBackMessage{
            best_bid: Uint128::from(best_bid),
            best_ask: Uint128::from(best_ask),
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        [self.best_bid.to_be_bytes().to_vec(), self.best_ask.to_be_bytes().to_vec()].concat()
    }
}
