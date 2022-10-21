#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Response,
    StdResult, SubMsg, WasmMsg, WasmQuery, StdError, Addr
};
use cw2::set_contract_version;
use xca::account::{Config, ExecuteMsg, InstantiateMsg, QueryMsg};
use xca::messages::{AccountInfo, ParsedVAA, Envelope, Request, RequestInfo};
use xca::registry::{ChainInfo, ConfigResponse as RegistryConfigResponse, QueryMsg as RegistryQueryMsg};
use xca::wormhole::{WormholeQueryMsg, GetAddressHexResponse};

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
        ExecuteMsg::BroadcastCall { request } => execute_broadcast_call(deps, info, request),
        ExecuteMsg::FinishCall { vaas } => execute_finish_call(deps, info, vaas),
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

    let destination_address_hex_response: GetAddressHexResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart{
        contract_addr: this_chain_info.wormhole_core.clone().into(),
        msg: to_binary(&WormholeQueryMsg::QueryAddressHex{
            address: deps.api.addr_validate(&outgoing_envelope.destination_address)?,
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

    //create xrequest
    let request: XRequest = XRequest{
        status: outgoing_envelope.id.status,
        request_chain_id: outgoing_envelope.id.x_account.chain_id,
        request_address: request_address_hex_response.hex.as_slice().into(),
        sender_chain_id: outgoing_envelope.sender.chain_id,
        sender_address: sender_address_hex_response.hex.as_slice().into(),
        emitter_chain_id: outgoing_envelope.emitter.chain_id,
        emitter_address: emitter_address_hex_response.hex.as_slice().into(),
        nonce: outgoing_envelope.nonce,
        destination_chain: outgoing_envelope.destination_chain,
        destination_address: outgoing_envelope.destination_address_hex_response.hex.as_slice().into(),
        is_response_expected: outgoing_envelope.is_response_expected,
        is_executable: outgoing_envelope.is_executable,
        caller: if Some(caller_hex) = caller_hex{
            caller_hex.as_slice().into()
        } else {
            [0u8;32]
        },
        response_of_chain_id: outgoing_envelope.response_of.chain_id,
        response_of_sequence: outgoing_envelope.response_of.sequence,
        request_status: RequestStatus::PENDING,


        
        response_chain_id:  outgoing_envelope.response_of.chain_id,
        response_of_sequence: outgoing_envelope.response_of.sequence,
        payload: 


    }


    Ok(Response::new())
}

pub fn execute_broadcast_call(
    deps: DepsMut,
    info: MessageInfo,
    request: Request,
) -> StdResult<Response>{
    Ok(Response::new())
}

pub fn execute_finish_call(
    deps: DepsMut,
    info: MessageInfo,
    vaas: Vec<Binary>,
) ->StdResult<Response> {
    Ok(Response::new())
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
