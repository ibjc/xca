#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, WasmMsg,
};
use cw2::set_contract_version;
use xca::account::Config;
use xca::messages::{AccountInfo, WormholeMessage};
use xca::request::Request;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};
use crate::state::CONFIG;

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
) -> Result<Response, ContractError> {
    set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;
    // TODO gate access for who can instantiate
    // TODO validate addrs
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
    _env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Call {
            msg_type,       // e.g. ExecuteMsg, QueryMsg, InstatiateMsg, MigrateMsg. null => ExecuteMsg
            msg,            // base64-encoded stringified JSON
            destination,    // address registry required
            receive_caller, // optional general relayer usage
            is_response_expected, // give back Ok(x => VAA)
            execution_dependency, // wormhole_message.sequence here
        } => execute_call(
            deps,
            info,
            msg_type,
            msg,
            destination,
            receive_caller,
            is_response_expected,
            execution_dependency,
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
    info: MessageInfo,
    msg_type: Option<String>,
    msg: Binary,
    destination: AccountInfo,
    receive_caller: Option<String>,
    is_response_expected: Option<bool>,
    execution_dependency: Option<WormholeMessage>,
) -> Result<Response, ContractError> {
    // query registry
    // registry config has wormhole address

    // send call to other chain's xaccount
    // pub x_chain_registry: String,   // Updatable by admins
    // pub admin: AccountInfo,         // Can update Config. (chain, addr)
    // pub master: AccountInfo,        // Can accept VAA executions from these. (chain, addr)
    // pub slave: Option<AccountInfo>, //
    let config = CONFIG.load(deps.storage)?;
    let mut submessages = Vec::new();
    submessages.push(SubMsg::reply_on_success(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: wormhole_contract,
            msg,
            funds: vec![],
        }),
        POST_REPLY_ID,
    ));

    Ok(Response::new().add_submessages(submessages))
}

pub fn execute_broadcast_call(
    deps: DepsMut,
    info: MessageInfo,
    request: Request,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn execute_finish_call(
    deps: DepsMut,
    info: MessageInfo,
    vaas: Vec<Binary>,
) -> Result<Response, ContractError> {
    Ok(Response::new())
}

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    x_chain_registry: String,
    admin: AccountInfo,
    master: AccountInfo,
    slave: Option<AccountInfo>,
) -> Result<Response, ContractError> {
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
pub fn query(_deps: Deps, _env: Env, _msg: QueryMsg) -> StdResult<Binary> {
    unimplemented!()
}

#[cfg(test)]
mod tests {}
