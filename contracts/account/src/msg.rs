use cosmwasm_schema::{cw_serde, QueryResponses};
use xca::{account::ExecuteMsg as XcaExecuteMsg, messages::AccountInfo};

#[cw_serde]
pub struct InstantiateMsg {
    pub x_chain_registry_address: String,
    pub admin: AccountInfo,
    pub master: AccountInfo,
    pub slave: Option<AccountInfo>,
}

#[cw_serde]
pub enum ExecuteMsg {}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {}
