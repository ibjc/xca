use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Binary;
use xca::messages::AccountInfo;
use xca::messages::*;
use xca::request::Request;

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
        msg_type: Option<String>, // e.g. ExecuteMsg, QueryMsg, InstatiateMsg, MigrateMsg. null => ExecuteMsg
        msg: Binary,              // base64-encoded stringified JSON
        destination: AccountInfo, // address registry required
        receive_caller: Option<String>, // optional general relayer usage
        is_response_expected: Option<bool>, // give back Ok(x => VAA)
        execution_dependency: Option<WormholeMessage>, // wormhole_message.sequence here
    },
    BroadcastCall {
        request: Request,
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
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ParsedVAA)]
    VerifyVAA { vaa: Binary },
}

#[cw_serde]
pub struct VerifyResponse {}
