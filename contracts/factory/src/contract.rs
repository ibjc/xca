#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult, SubMsg, WasmMsg, Reply, Event, StdError, CosmosMsg};
// use cw2::set_contract_version;

use crate::state::{Config, CONFIG};
use xca::factory::{ ExecuteMsg, InstantiateMsg, QueryMsg, ConfigResponse};
use xca::wormhole::WormholeExecuteMsg;
use xca::registry::{QueryMsg as RegistryQueryMsg, ConfigResponse as RegistryConfigResponse, ChainInfo};

/*
// version info for migration info
const CONTRACT_NAME: &str = "crates.io:registry";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
*/

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    CONFIG.save(
        deps.storage,
        &Config {
            x_account_registry: deps.api.addr_validate(&msg.x_account_registry)?,
        },
    )?;

    Ok(Response::new())
}

#[entry_point]
pub fn reply(deps: DepsMut, env: Env, reply: Reply) -> StdResult<Response> {

    let mut events: Vec<Event> = reply.result.unwrap().events;
    events.retain(|event| event.ty == "wasm");

    let sequence = &events[0]
        .attributes
        .iter()
        .find(|attr| attr.key == "message.sequence")
        .ok_or_else(|| StdError::generic_err("no sequence"))?
        .value;



    return Err(StdError::generic_err("fail ftw"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::OpenMail {
            msg
        } => {

            let config: Config = CONFIG.load(deps.storage)?;

            let res: RegistryConfigResponse = deps
            .querier
            .query_wasm_smart(config.x_account_registry, &RegistryQueryMsg::Config {})?;
    
            let this_chain_info: ChainInfo = res.chain_info.into_iter().find(|x| x.wormhole_id==res.chain_id_here).ok_or_else(|| StdError::generic_err("registry missing this chain's wormhole info"))?;

            let msg_out: SubMsg = SubMsg::reply_on_error(CosmosMsg::Wasm(WasmMsg::Execute{
                contract_addr: this_chain_info.wormhole_core.into(),
                funds: vec![],
                msg: to_binary(&WormholeExecuteMsg::PostMessage{
                    message: to_binary(&(6969u128>>96))?,
                    nonce: 420u32,
                })?,
            }),
            1);

            Ok(Response::new().add_submessage(msg_out))
        }
        ExecuteMsg::CreateXAccounts { 
            chain_ids,
            initial_master
        } => {

            Ok(Response::new())
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => {
            let config: Config = CONFIG.load(deps.storage)?;

            Ok(to_binary(&ConfigResponse {
                x_account_registry: config.x_account_registry.to_string(),
            })?)
        },
        QueryMsg::CallLogs{} => {
            Ok(to_binary(&69u32)?)
        }
    }
}

#[cfg(test)]
mod tests {}
