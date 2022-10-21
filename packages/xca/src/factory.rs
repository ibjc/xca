use crate::messages::AccountInfo;
use cosmwasm_std::{Binary};
use cosmwasm_schema::cw_serde;

#[cw_serde]
pub struct InstantiateMsg {
  pub x_account_registry: String,
}

#[cw_serde]
pub enum ExecuteMsg{
  OpenMail{
    //incoming_envelope: Envelope,
    msg: Binary,
  },
  CreateXAccounts{
    chain_ids: Vec<u64>, //only deploy to factory's chain if empty
    initial_master: Option<AccountInfo>
  }
}

#[cw_serde]
pub enum QueryMsg{
  Config{},
  CallLogs{}
}

#[cw_serde]
pub struct ConfigResponse {
  pub x_account_registry: String,
}



