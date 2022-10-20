use cosmwasm_schema::cw_serde;


#[cw_serde]
pub struct WormholeMessage {
  // values optional if used for execution dependency settings 
	message: Option<Message>, 
	sender: String, // msg.sender of request
	emitter: Option<AccountInfo>, // emitter of VAA 
	sequence: Option<u64>, 
	is_response_expected: bool, // 1: receiver should send back response, otherwise abort 
	is_executable: bool, // 1: payload must be executed post-receival, otherwise abort 
	execution_dependency: Option<Box<WormholeMessage>> //DAG-like message execution dependencies
}

#[cw_serde]
pub struct Message {
	nonce: Option<u32>, 
	consistency_level: Option<u8>, 
	payload: Vec<u8> 
}

#[cw_serde]
pub struct AccountInfo {
  pub chain_id: u64, //wormhole chainid
  pub address: String,
  // pub is_emitter: u8, //1 emitter, 0 receiver

  //TODO: accountinfo"s" version w/ vec of <accountinfo>
}

#[cw_serde]
pub struct WormholeResponse {
	vaa_details: ParsedVAA, 
	wormhole_message: WormholeMessage 
}

#[cw_serde]
pub struct ParsedVAA { 
	pub version: u8,
  pub guardian_set_index: u32,
  pub timestamp: u32,
  pub nonce: u32,
  pub len_signers: u8,

  pub emitter_chain: u16,
  pub emitter_address: Vec<u8>,
  pub sequence: u64,
  pub consistency_level: u8,
  pub payload: Vec<u8>,

  pub hash: Vec<u8>,
}

#[cw_serde]
pub enum RequestStatus {
	Pending, // VAA transmission is currently in way 
	Complete, // Response is given or payload is sent without expecting response 
	Failed // Response given as failed 
}