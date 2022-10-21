use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
    pub chain_id_here: u64,
    pub x_account_code_id: u64,
}

#[cw_serde]
pub enum ExecuteMsg {
    UpdateConfig {
        x_account_code_id: Option<u64>,
    },
    UpsertChainInfo {
        chain_info: ChainInfo,
    },
}

#[cw_serde]
pub enum QueryMsg {
    Config {},
}

#[cw_serde]
pub struct ConfigResponse {
    pub chain_id_here: u64,
    pub x_account_code_id: u64,
    pub chain_info: Vec<ChainInfo>,
}

#[cw_serde]
pub struct ChainInfo {
    pub wormhole_id: u64,
    pub wormhole_core: String,
    pub x_account_factory: String,
    pub x_account_deployer: String,
}
