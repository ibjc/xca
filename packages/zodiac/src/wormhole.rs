use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use cosmwasm_std::{Binary, Addr};

#[derive(Serialize, Deserialize, JsonSchema)]
pub struct InstantiateMsg {
    pub owner: String,
    pub wormhole_contract: String,
    pub accounts: Vec<AccountInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {

    //owner + emitter only
    UpdateConfig{
        owner: Option<String>,
        wormhole_contract: Option<String>,
    },
    UpsertAccount{
        account_info: AccountInfo,
    },
    DeleteAccount{
        chain_id: u64,
        address: String,
    },

    //emitter only
    WormholeDispatch{
        payload: Binary,
    },

    //all
    WormholeReceive{
        vaa: Binary,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config{},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ConfigResponse {
    pub owner: String,
    pub wormhole_contract: String,
    pub accounts: Vec<AccountInfo>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AccountInfo {
    pub chain_id: u64,
    pub address: String,
    pub is_emitter: u8, //1 emitter, 0 receiver
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WormholeQueryMsg {
    GuardianSetInfo {},
    VerifyVAA { vaa: Binary, block_time: u64 },
    GetState {},
    QueryAddressHex { address: Addr },
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WormholeExecuteMsg {
    SubmitVAA { vaa: Binary },
    PostMessage { message: Binary, nonce: u32 },
}