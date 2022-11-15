use crate::messages::{AccountInfo, Request, Envelope};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;

#[cw_serde]
pub struct Config {
    pub x_chain_registry: String,   // Updatable by admins
    pub admin: AccountInfo,         // Can update Config. (chain, addr)
    pub master: AccountInfo,        // Can accept VAA executions from these. (chain, addr)
    pub slave: Option<AccountInfo>, //
}

#[cw_serde]
pub struct InstantiateMsg {
    pub x_chain_registry_address: String,
    pub admin: AccountInfo,
    pub master: AccountInfo,
    pub slave: Option<AccountInfo>,
}

#[cw_serde]
pub enum ExecuteMsg {
    Call {
        outgoing_envelope: Envelope, 
        msg_type: Option<String>, // e.g. ExecuteMsg, QueryMsg, InstatiateMsg, MigrateMsg, xData. null => ExecuteMsg
        msg: Binary, // base64-encoded stringified JSON
        x_data: Option<Binary>, // optional data, not used here
    },
    FinishCall {
        vaas: Vec<Binary>,
    },
    UpdateConfig {
        x_chain_registry: String,
        admin: AccountInfo,
        master: AccountInfo,
        slave: Option<AccountInfo>,
    },
}

#[cw_serde]
pub enum QueryMsg{
    Config{},
}