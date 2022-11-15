#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, SubMsg, WasmMsg, WasmQuery, StdError, Addr, from_binary, Reply
};
use cw2::set_contract_version;
use xca::account::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};
use xca::messages::{AccountInfo, ParsedVAA, Envelope, Request, RequestInfo, RequestStatus};
use xca::registry::{ChainInfo, ConfigResponse as RegistryConfigResponse, QueryMsg as RegistryQueryMsg};
use xca::wormhole::{WormholeQueryMsg, GetAddressHexResponse, WormholeExecuteMsg};
use xca::byte_utils::ByteUtils;
use xca::staking::{InstantiateMsg as StakingInitMsg};

use crate::state::{CONFIG, XRequest};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:account";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

const POST_REPLY_ID: u64 = 1;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // TODO gate access for who can instantiate
    // TODO validate addrs
    deps.api.addr_validate(&msg.x_chain_registry_address)?;
    CONFIG.save(
        deps.storage,
        &Config {
            x_chain_registry: msg.x_chain_registry_address,
            admin: msg.admin,
            master: msg.master,
            slave: msg.slave,
        },
    )?;

    Ok(Response::new()
        .add_attribute("action", "instantiate")
        .add_attribute("contract-name", CONTRACT_NAME)
        .add_attribute("contract-version", CONTRACT_VERSION))
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> StdResult<Response> {
    match reply.id {
        POST_REPLY_ID => {



            Ok(Response::new())
        },
        _ => Err(StdError::generic_err("invalid reply")),
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::Call {
            outgoing_envelope, 
            msg_type, // e.g. ExecuteMsg, QueryMsg, InstatiateMsg, MigrateMsg, xData. null => ExecuteMsg
            msg, // base64-encoded stringified JSON
            x_data, // optional data, not used here
        } => execute_call(
            deps,
            env,
            info,
            outgoing_envelope,
            msg_type,
            msg,
            x_data,
        ),
        ExecuteMsg::FinishCall { vaas } => execute_finish_call(deps, env, info, vaas),
        ExecuteMsg::UpdateConfig {
            x_chain_registry,
            admin,
            master,
            slave,
        } => execute_update_config(deps, info, x_chain_registry, admin, master, slave),
    }
}

pub fn execute_call(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    outgoing_envelope: Envelope, 
    msg_type: Option<String>, // e.g. ExecuteMsg, QueryMsg, InstatiateMsg, MigrateMsg, xData. null => ExecuteMsg
    msg: Binary, // base64-encoded stringified JSON
    x_data: Option<Binary>, // optional data, not used here
) -> StdResult<Response>{
    let config: Config = CONFIG.load(deps.storage)?;

    // query registry
    let registry_addr: Addr = deps.api.addr_validate(&config.x_chain_registry)?;
    let res: RegistryConfigResponse = deps
        .querier
        .query_wasm_smart(registry_addr, &RegistryQueryMsg::Config {})?;

    let this_chain_info: ChainInfo = res.chain_info.into_iter().find(|x| x.wormhole_id==res.chain_id_here).ok_or_else(|| StdError::generic_err("missing local chain info in registry")).unwrap();

    let mut outgoing_envelope = outgoing_envelope;

    //xaccount-deployer envelop details
    outgoing_envelope.nonce = Some(6969u32);
    outgoing_envelope.id = Some(RequestInfo{
        status: 1u8,
        x_account: AccountInfo{
            chain_id: this_chain_info.wormhole_id,
            address: info.sender.clone().into(),
        },
    });
    outgoing_envelope.sender = Some(AccountInfo{
        chain_id: this_chain_info.wormhole_id,
        address: info.sender.clone().into(),
    });
    outgoing_envelope.emitter = Some(AccountInfo{
        chain_id: this_chain_info.wormhole_id,
        address: env.contract.address.clone().into(),
    });

    //fetch hex'd verison of every address
    let request_address_hex_response: GetAddressHexResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart{
        contract_addr: this_chain_info.wormhole_core.clone().into(),
        msg: to_binary(&WormholeQueryMsg::QueryAddressHex{
            address: info.sender.clone(),
        })?
    }))?;

    let sender_address_hex_response: GetAddressHexResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart{
        contract_addr: this_chain_info.wormhole_core.clone().into(),
        msg: to_binary(&WormholeQueryMsg::QueryAddressHex{
            address: info.sender.clone(),
        })?
    }))?;

    let emitter_address_hex_response: GetAddressHexResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart{
        contract_addr: this_chain_info.wormhole_core.clone().into(),
        msg: to_binary(&WormholeQueryMsg::QueryAddressHex{
            address: env.contract.address.clone(),
        })?
    }))?;

    let caller_hex: Option<String> = if let Some(caller) = outgoing_envelope.caller{
        let caller_address_hex_response: GetAddressHexResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart{
            contract_addr: this_chain_info.wormhole_core.clone().into(),
            msg: to_binary(&WormholeQueryMsg::QueryAddressHex{
                address: deps.api.addr_validate(&caller)?,
            })?
        }))?;

        Some(caller_address_hex_response.hex)
    } else {
        None
    };

    //figure out payload based on type + binary
    let is_executable: u8 = if let Some(msg_type) = msg_type{
        if msg_type == String::from("QueryMsg"){
            0u8
        } else {
            1u8
        }
    } else {
        1u8
    };

    //create xrequest
    let request: XRequest = XRequest{
        status: outgoing_envelope.id.clone().unwrap().status,
        request_chain_id: outgoing_envelope.id.unwrap().x_account.chain_id,

        /*
        //request_address: request_address_hex_response.hex.as_bytes().to_vec(),
        sender_chain_id: outgoing_envelope.sender.unwrap().chain_id,
        //sender_address: sender_address_hex_response.hex.as_bytes().to_vec(),
        emitter_chain_id: outgoing_envelope.emitter.unwrap().chain_id,
        //emitter_address: emitter_address_hex_response.hex.as_bytes().to_vec(),
        nonce: outgoing_envelope.nonce.unwrap(),
        destination_chain: outgoing_envelope.destination_chain,
        //destination_address: outgoing_envelope.destination_address.as_bytes().to_vec(),
        is_response_expected: outgoing_envelope.is_response_expected,
        is_executable: is_executable,
        execution_dependency_chain_id: 0u64,
        execution_dependency_sequence: 0u64,
        
        response_of_chain_id: outgoing_envelope.response_of.clone().unwrap().chain_id,
        response_of_sequence: outgoing_envelope.response_of.clone().unwrap().sequence,
        request_status: RequestStatus::PENDING,

        response_chain_id: this_chain_info.wormhole_id,
        response_sequence: outgoing_envelope.response_of.clone().unwrap().sequence,
        */

        payload: msg.into(),
    };

    let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Execute{
        contract_addr: this_chain_info.wormhole_core.into(),
        funds: vec![],
        msg: to_binary(&WormholeExecuteMsg::PostMessage{
            message: request.serialize().into(),
            nonce: 6969u32,
        })?,
    });

    Ok(Response::new().add_message(msg))
}

pub fn execute_finish_call(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    vaas: Vec<Binary>,
) ->StdResult<Response> {

    //fetch config
    let config: Config = CONFIG.load(deps.storage)?;

    // query registry
    let registry_addr: Addr = deps.api.addr_validate(&config.x_chain_registry)?;
    let res: RegistryConfigResponse = deps
        .querier
        .query_wasm_smart(registry_addr, &RegistryQueryMsg::Config {})?;

    let this_chain_info: ChainInfo = res.chain_info.into_iter().find(|x| x.wormhole_id==res.chain_id_here).ok_or_else(|| StdError::generic_err("missing local chain info in registry")).unwrap();

    let mut messages: Vec<CosmosMsg> = vec![];

    for vaa in vaas{
        let parsed_vaa: ParsedVAA = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart{
            contract_addr: this_chain_info.wormhole_core.clone().into(),
            msg: to_binary(&WormholeQueryMsg::VerifyVAA{
                vaa: vaa.clone(),
                block_time: env.block.time.seconds(),
            })?,
        }))?;
        
        let request: XRequest = XRequest::deserialize(&parsed_vaa.payload)?;

        /*
        //confirm vaa's destination_chain matches
        if request.destination_chain > 0u64 && request.destination_chain != this_chain_info.wormhole_id{
            continue;
        }

        //confirm caller (ie relayer or not)
        let caller = deps.api.addr_humanize(&(request.caller.as_slice()).get_address(0))?;
        if request.caller != &[0u8;32] && caller != info.sender{
            continue;
        } 

        //confirm emitter
        let emitter_address = deps.api.addr_humanize(&(request.emitter_address.as_slice()).get_address(0))?;
        if emitter_address != config.master.address || request.emitter_chain_id != config.master.chain_id{
            continue;
        }
        */
        
        //let msg: CosmosMsg = from_binary(&request.payload.into())?;

        let msg: CosmosMsg = CosmosMsg::Wasm(WasmMsg::Instantiate{
            msg: to_binary(&StakingInitMsg{
                denom_name: String::from("uluna")
            })?,
            funds: vec![],
            label: String::from("staking_contract"),
            code_id: 4998,
            admin: None,
        });

        if 1==0{

            return Err(StdError::generic_err(format!("{} {} ", request.status.to_string(), request.request_chain_id.to_string(), )))
        }

        messages.push(msg);
    }


    Ok(Response::new().add_messages(messages))
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    x_chain_registry: String,
    admin: AccountInfo,
    master: AccountInfo,
    slave: Option<AccountInfo>,
) -> StdResult<Response> {
    // TODO gate access
    // if config.admin.address != info.sender {
    //     return Err(ContractError::Unauthorized {});
    // }

    // TODO sanitize and validate user input values
    CONFIG.save(
        deps.storage,
        &Config {
            x_chain_registry,
            admin,
            master,
            slave,
        },
    )?;
    Ok(Response::new())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config: Config = CONFIG.load(deps.storage)?;

            Ok(to_binary(&config)?)
        }
        //QueryMsg::VerifyVAA { vaa } => to_binary(&query_parse_and_verify_vaa(deps, env, vaa)?),
    }
}

pub fn query_parse_and_verify_vaa(deps: Deps, env: Env, data: Binary) -> StdResult<ParsedVAA> {
    Ok(parse_vaa(deps, env.block.time.seconds(), &data)?)
}

fn parse_vaa(deps: Deps, block_time: u64, data: &Binary) -> StdResult<ParsedVAA> {
    let config: Config = CONFIG.load(deps.storage)?;
    let registry_addr = deps.api.addr_validate(&config.x_chain_registry)?;
    let res: RegistryConfigResponse = deps
        .querier
        .query_wasm_smart(registry_addr, &RegistryQueryMsg::Config {})?;

    let this_chain_info: ChainInfo = res.chain_info.into_iter().find(|x| x.wormhole_id==res.chain_id_here).ok_or_else(|| StdError::generic_err("registry missing this chain's wormhole info"))?;

    let vaa: ParsedVAA = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: this_chain_info.wormhole_core.into(),
        msg: to_binary(&WormholeQueryMsg::VerifyVAA {
            vaa: data.clone(),
            block_time,
        })?,
    }))?;
    Ok(vaa)
}
#[cfg(test)]
mod tests {}
