use crate::messages::{WormholeMessage, WormholeResponse, RequestStatus};
use cosmwasm_schema::cw_serde;


#[cw_serde]
pub struct Request {
	wormhole_message: WormholeMessage, 

	/*
	Destination chain and address of request
	null for a general broadcast to all chains
	*/
	destination_chain: Option<Vec<u8>>, 
	destination_address: Option<Vec<String>>, 

	/* 
	Optional caller designation - possible use for smart contracts to execute logic atomically right after VAA receival 
	null to allow relayers to finish receival on behalf of receiver
	*/ 
	caller: Option<String>, 

	/* 
	Status of request. Can be one of pending, complete, or failed 
	*/
	status: RequestStatus, 

	/*
  Request response. Filled in once response is received
	*/
	response: Option<WormholeResponse> 
}