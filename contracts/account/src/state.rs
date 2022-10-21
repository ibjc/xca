use cw_storage_plus::Item;
use xca::account::Config;
use cosmwasm_schema::cw_serde;
use cosmwasm_std::StdResult;
use xca::byte_utils::ByteUtils;

pub const CONFIG: Item<Config> = Item::new("config");

#[cw_serde]
pub struct XRequest{

  // filled by the xAccount deployer
  pub status: u8,
  pub request_chain_id: u64,
  pub request_address: Vec<u8>,

  pub sender_chain_id: u64,
  pub sender_address: Vec<u8>,
  
  pub emitter_chain_id: u64,
  pub emitter_address: Vec<u8>,
  
  pub nonce: u32,

  // filled by sender
  pub destination_chain: u64,
  pub destination_address: Vec<u8>,

  pub is_response_expected: u8,
  pub is_executable: u8,
  pub execution_dependency_chain_id: u64,
  pub execution_dependency_sequence: u64,

  pub caller: Vec<u8>,

  pub response_of_chain_id: u64,
  pub response_of_sequence: u64,

  pub request_status: u8,

  pub response_chain_id: u64,
  pub response_sequence: u64,

  pub payload: Vec<u8>,

}

impl XRequest{
  pub fn deserialize(data: &Vec<u8>) -> StdResult<Self>{
    let data = data.as_slice();
    let status = data.get_u8(0);
    let request_chain_id = data.get_u64(1);
    let request_address = data.get_const_bytes::<32>(9);
    let sender_chain_id = data.get_u64(41);
    let sender_address = data.get_const_bytes::<32>(49);
    let emitter_chain_id = data.get_u64(81);
    let emitter_address = data.get_const_bytes::<32>(89);
    let destination_chain = data.get_u64(121);
    let destination_address = data.get_const_bytes::<32>(129);
    let is_response_expected = data.get_u8(161);
    let is_executable = data.get_u8(162);
    let execution_dependency_chain_id = data.get_u64(163);
    let execution_dependency_sequence = data.get_u64(171);
    let caller = data.get_const_bytes::<32>(179);
    let response_of_chain_id = data.get_u64(211);
    let response_of_sequence = data.get_u64(219);
    let request_status = data.get_u8(227);
    let response_chain_id = data.get_u64(228);
    let response_sequence = data.get_u64(236);
    let nonce = data.get_u32(244);
    let payload = data[248..].to_vec();

    Ok(XRequest{
      status,
      request_chain_id,
      request_address: request_address.into(),
      sender_chain_id,
      sender_address: sender_address.into(),
      emitter_chain_id,
      emitter_address: emitter_address.into(),
      nonce,
      destination_chain,
      destination_address: destination_address.into(),
      is_response_expected,
      is_executable,
      execution_dependency_chain_id,
      execution_dependency_sequence,
      caller: caller.into(),
      response_of_chain_id,
      response_of_sequence,
      request_status,
      response_chain_id,
      response_sequence,
      payload,
    })
  }

  pub fn serialize(&self) -> Vec<u8>{
    [
      self.status.to_be_bytes().to_vec(),
      self.request_chain_id.to_be_bytes().to_vec(),
      self.request_address[0..32].to_vec(),
      self.sender_chain_id.to_be_bytes().to_vec(),
      self.sender_address.to_vec(),
      self.emitter_chain_id.to_be_bytes().to_vec(),
      self.emitter_address.to_vec(),
      self.destination_chain.to_be_bytes().to_vec(),
      self.destination_address.to_vec(),
      self.is_response_expected.to_be_bytes().to_vec(),
      self.is_executable.to_be_bytes().to_vec(),
      self.execution_dependency_chain_id.to_be_bytes().to_vec(),
      self.execution_dependency_sequence.to_be_bytes().to_vec(),
      self.caller.to_vec(),
      self.response_of_chain_id.to_be_bytes().to_vec(),
      self.response_of_sequence.to_be_bytes().to_vec(),
      self.request_status.to_be_bytes().to_vec(),
      self.response_chain_id.to_be_bytes().to_vec(),
      self.response_sequence.to_be_bytes().to_vec(),
      self.payload.clone(),
    ].concat()
  }

  
}