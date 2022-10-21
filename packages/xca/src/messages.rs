use crate::{
    byte_utils::{extend_address_to_32, ByteUtils},
    error::ContractError,
};
use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Binary, StdResult};
use sha3::{Digest, Keccak256};

#[cw_serde]
pub struct Envelope {

    // --- Filled in by xAccount ---
    pub id: Option<RequestInfo>,
    pub sender: Option<AccountInfo>,
    pub emitter: Option<AccountInfo>,
    pub nonce: Option<u32>,

    // --- Filled in by sender ---
	/*
	Destination chain and address of request
	destination_chain == "0" for all receivers to process request


	*/
    pub destination_chain: u64,
    pub destination_address: String,

    pub is_response_expected: bool,
    pub is_executable: bool,
    pub execution_dependency: Option<RequestId>,

    /* 
	Optional caller designation - possible use for smart contracts to execute logic atomically right after VAA receival 
	null to allow relayers to finish receival on behalf of receiver
	*/ 
    pub caller: Option<String>,

    /*
	filled if request is a response of a previous request. 
	Sequence of VAA that triggered this response 
	*/
    pub response_of: Option<RequestId>,

}

#[cw_serde]
pub struct Request{
    pub envelope: Envelope,
    pub payload: Binary,
    pub status: RequestStatus,
    pub response: Option<RequestId>,
}

#[cw_serde]
pub struct RequestId{
    pub chain_id: u64,
    pub sequence: u64,
}

#[cw_serde]
pub struct RequestInfo {
	pub status: u8, 
	pub x_account: AccountInfo // used if a xAccount pair is being deployed. Stores the newly deployed xAccount of chain_id_here 
}

#[cw_serde]
pub struct AccountInfo{
    pub chain_id: u64,
    pub address: String,
}

#[cw_serde]
pub struct RequestStatus;

impl Request {
    pub const PENDING: u8 = 1;
    pub const COMPLETE: u8 = 2;
    pub const FAILED: u8 = 3;
}

// Validator Action Approval(VAA) data
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

impl ParsedVAA {
    /* VAA format:
    header (length 6):
    0   uint8   version (0x01)
    1   uint32  guardian set index
    5   uint8   len signatures
    per signature (length 66):
    0   uint8       index of the signer (in guardian keys)
    1   [65]uint8   signature
    body:
    0   uint32      timestamp (unix in seconds)
    4   uint32      nonce
    8   uint16      emitter_chain
    10  [32]uint8   emitter_address
    42  uint64      sequence
    50  uint8       consistency_level
    51  []uint8     payload
    */

    pub const HEADER_LEN: usize = 6;
    pub const SIGNATURE_LEN: usize = 66;

    pub const GUARDIAN_SET_INDEX_POS: usize = 1;
    pub const LEN_SIGNER_POS: usize = 5;

    pub const VAA_NONCE_POS: usize = 4;
    pub const VAA_EMITTER_CHAIN_POS: usize = 8;
    pub const VAA_EMITTER_ADDRESS_POS: usize = 10;
    pub const VAA_SEQUENCE_POS: usize = 42;
    pub const VAA_CONSISTENCY_LEVEL_POS: usize = 50;
    pub const VAA_PAYLOAD_POS: usize = 51;

    // Signature data offsets in the signature block
    pub const SIG_DATA_POS: usize = 1;
    // Signature length minus recovery id at the end
    pub const SIG_DATA_LEN: usize = 64;
    // Recovery byte is last after the main signature
    pub const SIG_RECOVERY_POS: usize = Self::SIG_DATA_POS + Self::SIG_DATA_LEN;

    pub fn deserialize(data: &[u8]) -> StdResult<Self> {
        let version = data.get_u8(0);

        // Load 4 bytes starting from index 1
        let guardian_set_index: u32 = data.get_u32(Self::GUARDIAN_SET_INDEX_POS);
        let len_signers = data.get_u8(Self::LEN_SIGNER_POS) as usize;
        let body_offset: usize = Self::HEADER_LEN + Self::SIGNATURE_LEN * len_signers as usize;

        // Hash the body
        if body_offset >= data.len() {
            return ContractError::InvalidVAA.std_err();
        }
        let body = &data[body_offset..];
        let mut hasher = Keccak256::new();
        hasher.update(body);
        let hash = hasher.finalize().to_vec();

        // Rehash the hash
        let mut hasher = Keccak256::new();
        hasher.update(hash);
        let hash = hasher.finalize().to_vec();

        // Signatures valid, apply VAA
        if body_offset + Self::VAA_PAYLOAD_POS > data.len() {
            return ContractError::InvalidVAA.std_err();
        }

        let timestamp = data.get_u32(body_offset);
        let nonce = data.get_u32(body_offset + Self::VAA_NONCE_POS);
        let emitter_chain = data.get_u16(body_offset + Self::VAA_EMITTER_CHAIN_POS);
        let emitter_address = data
            .get_bytes32(body_offset + Self::VAA_EMITTER_ADDRESS_POS)
            .to_vec();
        let sequence = data.get_u64(body_offset + Self::VAA_SEQUENCE_POS);
        let consistency_level = data.get_u8(body_offset + Self::VAA_CONSISTENCY_LEVEL_POS);
        let payload = data[body_offset + Self::VAA_PAYLOAD_POS..].to_vec();

        Ok(ParsedVAA {
            version,
            guardian_set_index,
            timestamp,
            nonce,
            len_signers: len_signers as u8,
            emitter_chain,
            emitter_address,
            sequence,
            consistency_level,
            payload,
            hash,
        })
    }
}
