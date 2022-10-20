use crate::messages::{AccountInfo, WormholeMessage};
use crate::request::Request;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::Binary;


#[cw_serde]
pub struct Config {
  x_chain_registry: String, // Updatable by admins

	admin: AccountInfo, // Can update Config. (chain, addr) 
	master: AccountInfo, // Can accept VAA executions from these. (chain, addr) 
	slave: Option<AccountInfo> // 
}

#[cw_serde]
pub struct InstantiateMsg {
  x_chain_registry_address: String,
  admin: AccountInfo,
  master: AccountInfo,
  slave: Option<AccountInfo>
}

#[cw_serde]
pub enum ExecuteMsg{
  Call {
    msg_type: Option<String>, // e.g. ExecuteMsg, QueryMsg, InstatiateMsg, MigrateMsg. null => ExecuteMsg
    msg: Binary, // base64-encoded stringified JSON
    destination: AccountInfo, // address registry required 
    receive_caller: Option<String>, // optional general relayer usage 
    is_response_expected: Option<bool>, // give back Ok(x => VAA)
    execution_dependency: Option<WormholeMessage> // wormhole_message.sequence here 
  },
  BroadcastCall {
    request: Request 
  },
  FinishCall {
    vaas: Vec<Binary> 
  },
  UpdateConfig{
    x_chain_registry: String,
    admin: AccountInfo,
    master: AccountInfo,
    slave: Option<AccountInfo>,
  }
}

#[cw_serde]
pub enum MsgType{
  Execute,
  Query,
  Instantiate,
}