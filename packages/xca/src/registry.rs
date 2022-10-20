use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub wormhole_core_contract: String,
    pub x_account_factory: String,
    pub wormhole_chain_ids: Vec<Chain>,
    pub x_account_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        wormhole_core_contract: Option<String>,
        x_account_factory: Option<String>,
        x_account_code_id: Option<u64>,
    },
    UpsertWormholeChainId {
        chain: Chain,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub wormhole_core_contract: String,
    pub x_account_factory: String,
    pub wormhole_chain_ids: Vec<Chain>,
    pub x_account_code_id: u64,
}

#[cw_serde]
pub struct Chain {
    pub name: String,
    pub wormhole_id: u8,
}
