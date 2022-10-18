use cosmwasm_std::{Addr, StdResult, Binary};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_storage_plus::{Item, Map};
use cw_asset::AssetInfo;
use xca::wormhole::AccountInfo;

pub const CONFIG: Item<Config> = Item::new("\u{0}\u{6}config");
pub const STATE: Item<State> = Item::new("\u{0}\u{7}state");
pub const VAA_ARCHIVE: Map<&[u8], bool> = Map::new("vaa_archive");

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub owner: Addr,
    pub wormhole_contract: Addr,
    pub accounts: Vec<AccountInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct State{
    pub nonce: u32,
}




pub struct TokenBridgeMessage {
    pub payload: Vec<u8>,
}

impl TokenBridgeMessage {
    pub fn deserialize(data: &Vec<u8>) -> StdResult<Self> {
        let data = data.as_slice();
        let payload = data;

        Ok(TokenBridgeMessage{
            payload: payload.to_vec(),
        })
    }

    pub fn serialize(&self) -> Vec<u8> {
        self.payload.clone()
    }
}
